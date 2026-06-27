#![cfg(test)]
extern crate std;

use soroban_sdk::{
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr},
    testutils::{Address as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env, Vec, U256,
};

use crate::{groth16_verify, vk, Proof, Terroir, TerroirClient};

// ---------------------------------------------------------------------------
// hex / field helpers (mirror spike/contract/src/test.rs)
// ---------------------------------------------------------------------------

fn decode_hex<const N: usize>(s: &str) -> [u8; N] {
    assert_eq!(s.len(), N * 2, "hex length mismatch");
    let bytes = s.as_bytes();
    let mut out = [0u8; N];
    let mut i = 0;
    while i < N {
        let hi = (bytes[2 * i] as char).to_digit(16).unwrap() as u8;
        let lo = (bytes[2 * i + 1] as char).to_digit(16).unwrap() as u8;
        out[i] = (hi << 4) | lo;
        i += 1;
    }
    out
}

fn g1(env: &Env, s: &str) -> Bn254G1Affine {
    Bn254G1Affine::from_array(env, &decode_hex::<64>(s))
}
fn g2(env: &Env, s: &str) -> Bn254G2Affine {
    Bn254G2Affine::from_array(env, &decode_hex::<128>(s))
}
fn fr(env: &Env, s: &str) -> Fr {
    Fr::from_bytes(BytesN::from_array(env, &decode_hex::<32>(s)))
}

/// 32-byte big-endian → Fr (value must be < BN254 scalar field order r).
fn fr_from_be32(env: &Env, bytes: &[u8; 32]) -> Fr {
    Fr::from_bytes(BytesN::from_array(env, bytes))
}
/// u128 → Fr (cents, fits easily in the BN254 scalar field).
fn fr_from_u128(env: &Env, v: u128) -> Fr {
    Fr::from_u256(U256::from_u128(env, v))
}

/// Build a placeholder proof (spike a*b=c fixture). The points are valid BN254
/// so the real pairing code path does not panic; the proof won't verify under
/// the placeholder VK, which is fine — `verify()` is bypassed under cfg(test).
fn placeholder_proof(env: &Env) -> Proof {
    Proof {
        a: g1(
            env,
            "2addba30bbc7a855c72199597ac9122aeb1f48d2886c97c9ddcbb9c9470f41df27c93a4b5449b14154ae3b6ed8e18c5233144a0af0fb961db3c8aa5466c0bf42",
        ),
        b: g2(
            env,
            "21271286f67c65ec80d03700de2c599befdb9a39103fdbefbaff90280fc012ed1f3fae548aabcfd32be7a5fd68ad5b3a7d55f8a0bd3c518e21fae36956ffaf0d29f4da60406e8c45d01507de2fd1c371e8f5c6582a5a74da04a65451cf46143c22b4f11fe9071577ab08084fc1ff9adbaa142b06e35ea26ed36cefacf4834e63",
        ),
        c: g1(
            env,
            "07637f6e2b995f56acf0caec61adbec66da503251015f43958339892ef0d20ce2ba364a3eaf53ebb364deb60d08243587232927d59d2c0dac20c2a4353d6ea5d",
        ),
    }
}

/// Split a 32-byte address payload into (hi, lo) 16-byte halves carried as Fr.
fn addr_to_hi_lo(env: &Env, addr: &Address) -> (Fr, Fr) {
    use soroban_sdk::address_payload::AddressPayload;
    let bytes = match addr.to_payload() {
        Some(AddressPayload::AccountIdPublicKeyEd25519(b)) => b,
        Some(AddressPayload::ContractIdHash(b)) => b,
        None => panic!("unsupported address type"),
    };
    let arr = bytes.to_array();
    let mut hi = [0u8; 32];
    let mut lo = [0u8; 32];
    hi[16..32].copy_from_slice(&arr[0..16]);
    lo[16..32].copy_from_slice(&arr[16..32]);
    (fr_from_be32(env, &hi), fr_from_be32(env, &lo))
}

// ---------------------------------------------------------------------------
// Fixture: a valid-looking set of 7 public signals in Decisión A order.
//   [r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash]
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Signals {
    r_cert: [u8; 32],
    lot_commit: [u8; 32],
    nullifier: [u8; 32],
    floor_price: u128,
    premium_amount: u128,
}

fn default_signals() -> Signals {
    let mut r_cert = [0u8; 32];
    r_cert[31] = 0x42; // < BN254 scalar field order r
    let mut lot_commit = [0u8; 32];
    lot_commit[31] = 0x07;
    let mut nullifier = [0u8; 32];
    nullifier[31] = 0x99;
    Signals {
        r_cert,
        lot_commit,
        nullifier,
        floor_price: 1_000,
        premium_amount: 1_000_000, // 10000.00 USDC at 2 decimals
    }
}

fn build_pub_signals(env: &Env, s: &Signals, payout: &Address) -> Vec<Fr> {
    let (payout_hi, payout_lo) = addr_to_hi_lo(env, payout);
    let mut v = Vec::new(env);
    v.push_back(fr_from_be32(env, &s.r_cert));
    v.push_back(fr_from_u128(env, s.floor_price));
    v.push_back(fr_from_be32(env, &s.lot_commit));
    v.push_back(fr_from_u128(env, s.premium_amount));
    v.push_back(payout_hi);
    v.push_back(payout_lo);
    v.push_back(fr_from_be32(env, &s.nullifier));
    v
}

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

struct Setup<'a> {
    env: Env,
    client: TerroirClient<'a>,
    token: Address,
    contract_id: Address,
}

/// Register the terroir contract + a mock SAC token (SEP-41), mint the escrow,
/// `init`, and `set_certifier_root` with `r_cert`. `mock_all_auths` is on.
fn setup<'a>(r_cert: &[u8; 32]) -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Terroir, ());
    let client = TerroirClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token = token_sac.address();

    // escrow: mint a large TUSDC balance into the terroir contract.
    StellarAssetClient::new(&env, &token).mint(&contract_id, &i128::MAX);

    client.init(&admin, &token);

    let mut rb = *r_cert;
    // guarantee r_cert < BN254 scalar field order (clear top 2 bits).
    rb[0] &= 0x3F;
    client.set_certifier_root(&admin, &BytesN::from_array(&env, &rb));

    Setup {
        env,
        client,
        token,
        contract_id,
    }
}

// ---------------------------------------------------------------------------
// Tests: non-crypto logic (verify() bypassed under cfg(test))
// ---------------------------------------------------------------------------

#[test]
fn test_vk_placeholder_shape() {
    let env = Env::default();
    let vk = vk(&env);
    // Decisión A: 7 public signals → ic.len() == nPublic + 1 == 8.
    assert_eq!(vk.ic.len(), 8);
}

#[test]
fn test_happy_path() {
    let s = default_signals();
    let st = setup(&s.r_cert);

    // r_cert stored by setup is the top-2-bits-cleared version; mirror it.
    let mut r_cert_stored = s.r_cert;
    r_cert_stored[0] &= 0x3F;

    let payout = Address::generate(&st.env);
    let premium_i128 = s.premium_amount as i128;

    let balance_before = TokenClient::new(&st.env, &st.token).balance(&payout);
    assert_eq!(balance_before, 0);

    // freeze the ledger timestamp so lot_status is deterministic.
    st.env.ledger().set_timestamp(1_700_000_000);

    let pub_signals = build_pub_signals(
        &st.env,
        &{
            let mut s = s;
            s.r_cert = r_cert_stored;
            s
        },
        &payout,
    );
    let proof = placeholder_proof(&st.env);

    st.client.claim_premium(&proof, &pub_signals, &payout);

    // payout received exactly premium_amount (Decisión G / I).
    let balance_after = TokenClient::new(&st.env, &st.token).balance(&payout);
    assert_eq!(balance_after, premium_i128);

    // lot registered with the claim timestamp.
    let lot = st
        .client
        .lot_status(&BytesN::from_array(&st.env, &s.lot_commit));
    assert_eq!(lot, Some(1_700_000_000u64));

    // escrow decreased by premium_amount.
    let escrow = TokenClient::new(&st.env, &st.token).balance(&st.contract_id);
    assert_eq!(escrow, i128::MAX - premium_i128);
}

#[test]
#[should_panic(expected = "nullifier already used")]
fn test_double_spend() {
    // First claim succeeds (seeds the nullifier in persistent storage), then
    // the second claim with the SAME nullifier must panic. The whole body runs
    // under #[should_panic]; the first claim must NOT panic, only the second.
    let s = default_signals();
    let st = setup(&s.r_cert);
    let mut r_cert_stored = s.r_cert;
    r_cert_stored[0] &= 0x3F;

    let payout = Address::generate(&st.env);
    let pub_signals = build_pub_signals(
        &st.env,
        &{
            let mut s = s;
            s.r_cert = r_cert_stored;
            s
        },
        &payout,
    );
    let proof = placeholder_proof(&st.env);

    // first claim — succeeds, seeds the nullifier in persistent storage
    st.client
        .claim_premium(&proof.clone(), &pub_signals.clone(), &payout);

    // second claim with the same nullifier — must panic
    st.client.claim_premium(&proof, &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "root mismatch")]
fn test_bad_root() {
    let s = default_signals();
    let st = setup(&s.r_cert);

    // build signals with a DIFFERENT r_cert than the one stored.
    let mut wrong_r_cert = s.r_cert;
    wrong_r_cert[31] = s.r_cert[31].wrapping_add(1);
    // (top-2-bits clearing in setup does not affect mismatch here.)

    let payout = Address::generate(&st.env);
    let pub_signals = build_pub_signals(
        &st.env,
        &{
            let mut s = s;
            s.r_cert = wrong_r_cert;
            s
        },
        &payout,
    );
    let proof = placeholder_proof(&st.env);

    st.client.claim_premium(&proof, &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "amount zero")]
fn test_amount_zero() {
    let s = default_signals();
    let st = setup(&s.r_cert);
    let mut r_cert_stored = s.r_cert;
    r_cert_stored[0] &= 0x3F;

    let payout = Address::generate(&st.env);
    let pub_signals = build_pub_signals(
        &st.env,
        &{
            let mut s = s;
            s.r_cert = r_cert_stored;
            s.premium_amount = 0;
            s
        },
        &payout,
    );
    let proof = placeholder_proof(&st.env);

    st.client.claim_premium(&proof, &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_root_non_admin() {
    let s = default_signals();
    let st = setup(&s.r_cert);
    let attacker = Address::generate(&st.env);
    // attacker authenticates as themselves (mock_all_auths) but is not admin.
    st.client
        .set_certifier_root(&attacker, &BytesN::from_array(&st.env, &s.r_cert));
}

// ---------------------------------------------------------------------------
// Crypto test (DEFERRED). TODO(T1): bake the real 3-link VK + a real proof,
// drop the #[ignore], and assert groth16_verify == true end-to-end.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "TODO(T1): replace placeholder VK + spike proof with the real 3-link verification_key.json + proof.json"]
fn test_groth16_with_real_vk() {
    let env = Env::default();
    let vk = vk(&env);
    // Shape contract: 7 public signals → ic.len() == 8 (Decisión A).
    assert_eq!(vk.ic.len(), 8);

    // Smoke-test the real BN254 pairing code path (the same one `claim_premium`
    // uses in the wasm build). The placeholder VK reuses the spike's a*b=c
    // points with ic padded to 8; with public input 33 (0x21) and the rest
    // zero, vk_x collapses to ic[0] + 33*ic[1] and the spike proof verifies.
    // This proves the wiring is correct — NOT the 3-link circuit (TODO(T1)).
    let proof = placeholder_proof(&env);
    let mut pub_signals = Vec::new(&env);
    pub_signals.push_back(fr(
        &env,
        "0000000000000000000000000000000000000000000000000000000000000021",
    ));
    pub_signals.push_back(fr(
        &env,
        "0000000000000000000000000000000000000000000000000000000000000000",
    ));
    pub_signals.push_back(fr(
        &env,
        "0000000000000000000000000000000000000000000000000000000000000000",
    ));
    pub_signals.push_back(fr(
        &env,
        "0000000000000000000000000000000000000000000000000000000000000000",
    ));
    pub_signals.push_back(fr(
        &env,
        "0000000000000000000000000000000000000000000000000000000000000000",
    ));
    pub_signals.push_back(fr(
        &env,
        "0000000000000000000000000000000000000000000000000000000000000000",
    ));
    pub_signals.push_back(fr(
        &env,
        "0000000000000000000000000000000000000000000000000000000000000000",
    ));
    let ok = groth16_verify(&env, &vk, &proof, &pub_signals);
    assert!(ok);
}
