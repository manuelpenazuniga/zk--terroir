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
// Real proof / public signals (Ola 3 role-tag, serialized.json) — payout = zkq-t0 (GCWZZEAF…)
// ---------------------------------------------------------------------------

// pubkey hex (32 bytes) of the payout account zkq-t0 (E2E Ola 3, testnet.json)
const PAYOUT_PUBKEY_HEX: &str = "ad9c90050d1ba96aceaf4a50df90ac41f03a305b28748c486672276b981deb35";

// r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash
// Ola 3 (role-tag) — regenerated from new circuit with leaf arities 6/4/4
const PUB_R_CERT: &str = "0228324e1057f1953a4914298d4f7ddba09104e41841f06a7666a787337dd17d";
const PUB_FLOOR: i128 = 150_000; // 150000 cents = $1500.00
const PUB_LOT_COMMIT: &str = "2ceda2ee11f38491b484858a98c200d48c97ce21fdf8e9217a62634de6da6563";
const PUB_PREMIUM: i128 = 37_500; // 37500 cents = $375.00
const PUB_PAYOUT_HI: &str = "00000000000000000000000000000000ad9c90050d1ba96aceaf4a50df90ac41";
const PUB_PAYOUT_LO: &str = "00000000000000000000000000000000f03a305b28748c486672276b981deb35";
const PUB_NULLIFIER: &str = "01bbfda831bc496413713c87eb1d43e38482e2f28f7d1e9f92a2e85a38d9e437";

const PROOF_A: &str =
    "165f8dc4032fd38c8db0c9ae856921800d75023919582ca6e39a2d4286f6208722d26f4b95507095f08e2eb89f26b1b4a0f4168017097f8de621ac95a648faec";
const PROOF_B: &str =
    "25c48c64b523571771b0a10069de4ba7c2b3772dc9d50f64de5ed9362fb48c55220a0c816e042087669b7b805ac2e1c4765cc5fcfb3fd2c11d02503595b3453706281b4bd65030b18fb494d2a26472e3fb7bb672a53980f45688dd4c37e6533c04e2f7ef476d22338c2731585aa05cd7b1c7e7f6eb7e844023eeb542a406483f";
const PROOF_C: &str =
    "11430dbc9598ea63a076303e9224280e8ac4d6b88b9073820c09bd89f56274670aafb21f8afd21781a9d360b7d3f08992be83c42b6998ea30ce7dc60213fe137";

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
        "0228324e1057f1953a4914298d4f7ddba09104e41841f06a7666a787337dd17e",
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
