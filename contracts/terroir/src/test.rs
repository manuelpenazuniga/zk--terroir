#![cfg(test)]
extern crate std;

// T3-final tests: crypto is exercised by the happy-path / double-spend tests
// (no `#[cfg(test)]` bypass in `verify()`). The baked VK is the real T1-v3 key.
// Proof + pub_signals below are real (serialized from circuits/proof.json +
// circuits/public.json, payout = real testnet account GBSQXMKJ…QTDL).
//
// Decisión A order (FROZEN): [r_cert, floor_price, lot_commit, premium_amount,
// payout_hi, payout_lo, nullifier_hash] (7 signals -> ic.len() == 8).
extern crate alloc;

use soroban_sdk::{
    address_payload::AddressPayload,
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr},
    testutils::{Address as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env, Vec,
};

use crate::{Proof, Terroir, TerroirClient};

// ---------------------------------------------------------------------------
// hex / field helpers (mirrors spike/contract/src/test.rs)
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

// ---------------------------------------------------------------------------
// Real proof / public signals (T1 v3, serialized.json) — payout = GBSQXMKJ…QTDL
// ---------------------------------------------------------------------------

// pubkey hex (32 bytes) of the payout account GBSQXMKJTJPXVRYM2VJIAQN47F64ALVSDZ7MDLDZ53OCF64ZIYQVQTDL
const PAYOUT_PUBKEY_HEX: &str = "650bb1499a5f7ac70cd5528041bcf97dc02eb21e7ec1ac79eedc22fb99462158";

// r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash
const PUB_R_CERT: &str = "28714846a51e2f9f0a1264b374618283303c9c0873db3ef9850d9efe6c1d0f5a";
const PUB_FLOOR: i128 = 150_000; // 150000 cents = $1500.00
const PUB_LOT_COMMIT: &str = "2ceda2ee11f38491b484858a98c200d48c97ce21fdf8e9217a62634de6da6563";
const PUB_PREMIUM: i128 = 37_500; // 37500 cents = $375.00
const PUB_PAYOUT_HI: &str = "00000000000000000000000000000000650bb1499a5f7ac70cd5528041bcf97d";
const PUB_PAYOUT_LO: &str = "00000000000000000000000000000000c02eb21e7ec1ac79eedc22fb99462158";
const PUB_NULLIFIER: &str = "01bbfda831bc496413713c87eb1d43e38482e2f28f7d1e9f92a2e85a38d9e437";

const PROOF_A: &str =
    "29505d0417ccffbaca13e4efc30c80a0fa00cf86849a3e4fc95e36e4e23624a8274a35118a07508050c407dc6edbf04c925db068789d9ba403a976bd4cb1f0df";
const PROOF_B: &str =
    "2a40238e2e359227975df26515dafd6cb8b7507693c077bf72390953797b1215016049f086181f5ddce7b125c2f99436165fc54ca639c26c7ee5fb81b179656414b6626cb0b7d6814abb2564e01c69b323b367c898666c99e559a77c6027de7b1f2d54a3ffa0f9b3ca3dad4f1ed7982f9b56f76c74459e0ba97813c2f8ed317e";
const PROOF_C: &str =
    "2439216f13b146ed3f51cb25e71f87cd4655712a383d8a31de4e441ae56ad07e05b4376c8d9a3bb1617993d5fa716c85276dc2e92b21820dea00a90f37b5b2ae";

/// Build the canonical Vec<Fr> matching `circuits/public.json` (Decisión A).
fn real_pub_signals(env: &Env) -> Vec<Fr> {
    let mut v = Vec::new(env);
    v.push_back(fr(env, PUB_R_CERT));
    v.push_back(fr(env, &format_fhex_i128(PUB_FLOOR)));
    v.push_back(fr(env, PUB_LOT_COMMIT));
    v.push_back(fr(env, &format_fhex_i128(PUB_PREMIUM)));
    v.push_back(fr(env, PUB_PAYOUT_HI));
    v.push_back(fr(env, PUB_PAYOUT_LO));
    v.push_back(fr(env, PUB_NULLIFIER));
    v
}

/// Encode a non-negative `i128` as 32-byte big-endian hex (left-padded) for Fr.
fn format_fhex_i128(v: i128) -> alloc::string::String {
    assert!(v >= 0, "fr hex: negative");
    let u = v as u128;
    let mut bytes = [0u8; 32];
    let be = u.to_be_bytes();
    bytes[16..32].copy_from_slice(&be[0..16]);
    let mut s = alloc::string::String::with_capacity(64);
    for b in bytes {
        use core::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

/// The real proof object (matches `circuits/proof.json`).
fn real_proof(env: &Env) -> Proof {
    Proof {
        a: g1(env, PROOF_A),
        b: g2(env, PROOF_B),
        c: g1(env, PROOF_C),
    }
}

/// Reconstruct the payout `Address` from the 32-byte payout payload baked
/// into the proof. Uses `hazmat-address` (Address::from_payload).
///
/// AUDIT-LOG T3 gotcha: a `G...` Account payout needs an asset trustline on
/// the SAC, which the local mock does not provision. We split the same 32
/// bytes as a Contract hash (C...) instead — contracts don't require a
/// trustline for SEP-41 transfers. `check_payout_binding` matches because the
/// 32-byte payload is unchanged (Decisión E is payload-agnostic). The on-
/// chain E2E keeps the real Account address (its trustline is established on
/// testnet).
fn real_payout(env: &Env) -> Address {
    let payload = BytesN::from_array(env, &decode_hex::<32>(PAYOUT_PUBKEY_HEX));
    Address::from_payload(env, AddressPayload::ContractIdHash(payload))
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
/// `init`, `set_certifier_root(PUB_R_CERT)`, and `set_floor(PUB_FLOOR)`.
/// `mock_all_auths` is on.
fn setup_real<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Terroir, ());
    let client = TerroirClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token = token_sac.address();

    // escrow: mint a large balance into the terroir contract.
    StellarAssetClient::new(&env, &token).mint(&contract_id, &i128::MAX);

    client.init(&admin, &token);

    client.set_certifier_root(
        &admin,
        &BytesN::from_array(&env, &decode_hex::<32>(PUB_R_CERT)),
    );
    client.set_floor(&admin, &PUB_FLOOR);

    Setup {
        env,
        client,
        token,
        contract_id,
    }
}

/// Returns a clone of `real_pub_signals` with index `idx` replaced by `new`.
fn with_signal_replaced(env: &Env, src: &Vec<Fr>, idx: u32, new: Fr) -> Vec<Fr> {
    let mut out = Vec::new(env);
    let mut i = 0;
    while i < src.len() {
        if i == idx {
            out.push_back(new.clone());
        } else {
            out.push_back(src.get(i).unwrap());
        }
        i += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// Tests: real crypto path (no bypass)
// ---------------------------------------------------------------------------

#[test]
fn test_happy_path_real() {
    let st = setup_real();
    let payout = real_payout(&st.env);
    let pub_signals = real_pub_signals(&st.env);
    let proof = real_proof(&st.env);

    // payout starts with zero TUSDC.
    assert_eq!(TokenClient::new(&st.env, &st.token).balance(&payout), 0);

    // freeze the ledger timestamp so lot_status is deterministic.
    st.env.ledger().set_timestamp(1_700_000_000);

    st.client.claim_premium(&proof, &pub_signals, &payout);

    // payout received EXACTLY premium_amount (Decisión G / I).
    assert_eq!(
        TokenClient::new(&st.env, &st.token).balance(&payout),
        PUB_PREMIUM
    );

    // lot registered with the claim timestamp.
    let lot = st.client.lot_status(&BytesN::from_array(
        &st.env,
        &decode_hex::<32>(PUB_LOT_COMMIT),
    ));
    assert_eq!(lot, Some(1_700_000_000u64));

    // escrow decreased by premium_amount.
    assert_eq!(
        TokenClient::new(&st.env, &st.token).balance(&st.contract_id),
        i128::MAX - PUB_PREMIUM
    );
}

#[test]
#[should_panic(expected = "nullifier already used")]
fn test_double_spend_real() {
    let st = setup_real();
    let payout = real_payout(&st.env);
    let pub_signals = real_pub_signals(&st.env);
    let proof = real_proof(&st.env);

    // first claim — verifies the real proof and seeds nullifier in storage.
    st.client
        .claim_premium(&proof.clone(), &pub_signals.clone(), &payout);

    // second claim with the same nullifier — panics BEFORE crypto.
    st.client.claim_premium(&proof, &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "root mismatch")]
fn test_bad_root_real() {
    let st = setup_real();
    let payout = real_payout(&st.env);
    let real = real_pub_signals(&st.env);
    // pub_signals[0]: tamper with r_cert (one byte changed). root-bind check
    // fires BEFORE crypto / nullifier / floor / payout.
    let wrong_r_cert = fr(
        &st.env,
        "28714846a51e2f9f0a1264b374618283303c9c0873db3ef9850d9efe6c1d0f5b",
    );
    let pub_signals = with_signal_replaced(&st.env, &real, 0, wrong_r_cert);
    let proof = real_proof(&st.env);

    st.client.claim_premium(&proof, &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "floor mismatch")]
fn test_bad_floor_real() {
    let st = setup_real();
    let payout = real_payout(&st.env);
    let real = real_pub_signals(&st.env);
    // pub_signals[1]: floor_price + 1 != stored PUB_FLOOR -> "floor mismatch"
    // fires BEFORE crypto (order: root, floor, amount, nullifier, crypto).
    let wrong_floor = fr(&st.env, &format_fhex_i128(PUB_FLOOR + 1));
    let pub_signals = with_signal_replaced(&st.env, &real, 1, wrong_floor);
    let proof = real_proof(&st.env);

    st.client.claim_premium(&proof, &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "amount zero")]
fn test_amount_zero_real() {
    let st = setup_real();
    let payout = real_payout(&st.env);
    let real = real_pub_signals(&st.env);
    // pub_signals[3]: zero premium -> "amount zero" before crypto.
    let pub_signals = with_signal_replaced(&st.env, &real, 3, fr(&st.env, &format_fhex_i128(0)));
    let proof = real_proof(&st.env);

    st.client.claim_premium(&proof, &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "bad proof")]
fn test_bad_proof_real() {
    // All upstream checks (root / floor / amount / nullifier-fresh) pass since
    // the proof points are untouched. We tamper pub_signals[2] (lot_commit):
    // vk_x no longer matches the real proof's vk_x -> `pairing_check` returns
    // false cleanly (no host panic on a malformed point) -> contract translates
    // `verify` returning false into "terroir: bad proof". `lot_commit` has no
    // pre-crypto check (only stored post-crypto), so this is the cleanest path
    // to exercise the crypto branch.
    let st = setup_real();
    let payout = real_payout(&st.env);
    let real = real_pub_signals(&st.env);
    let wrong_lot = fr(
        &st.env,
        "00000000000000000000000000000000000000000000000000000000000000ff",
    );
    let pub_signals = with_signal_replaced(&st.env, &real, 2, wrong_lot);
    let proof = real_proof(&st.env);

    st.client.claim_premium(&proof, &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_root_non_admin() {
    let st = setup_real();
    let attacker = Address::generate(&st.env);
    st.client.set_certifier_root(
        &attacker,
        &BytesN::from_array(&st.env, &decode_hex::<32>(PUB_R_CERT)),
    );
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_floor_non_admin() {
    let st = setup_real();
    let attacker = Address::generate(&st.env);
    st.client.set_floor(&attacker, &PUB_FLOOR);
}

#[test]
#[should_panic(expected = "floor negative")]
fn test_set_floor_negative() {
    // Admin legitimately authenticates, but a negative floor is rejected.
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Terroir, ());
    let client = TerroirClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    StellarAssetClient::new(&env, &token).mint(&contract_id, &i128::MAX);
    client.init(&admin, &token);
    client.set_floor(&admin, &-1);
}

#[test]
#[should_panic(expected = "floor not set")]
fn test_floor_not_set() {
    // init + set_certifier_root but NO set_floor -> claim must panic.
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Terroir, ());
    let client = TerroirClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    StellarAssetClient::new(&env, &token).mint(&contract_id, &i128::MAX);
    client.init(&admin, &token);
    client.set_certifier_root(
        &admin,
        &BytesN::from_array(&env, &decode_hex::<32>(PUB_R_CERT)),
    );
    // deliberately NOT calling set_floor

    let payout = real_payout(&env);
    let pub_signals = real_pub_signals(&env);
    client.claim_premium(&real_proof(&env), &pub_signals, &payout);
}

#[test]
#[should_panic(expected = "payout binding failed")]
fn test_payout_binding_real() {
    // Real proof + real pub_signals but a DIFFERENT payout address whose
    // payload != (payout_hi payout_lo) encoded in the proof. All upstream
    // checks pass; crypto ALSO passes (proof+pub_signals match) but the E
    // binding rejects. The funds cannot be redirected.
    let st = setup_real();
    let payout = Address::generate(&st.env); // not the baked one
    let pub_signals = real_pub_signals(&st.env);
    let proof = real_proof(&st.env);
    st.client.claim_premium(&proof, &pub_signals, &payout);
}
