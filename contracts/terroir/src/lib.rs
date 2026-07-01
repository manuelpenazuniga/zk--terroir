#![no_std]
//! ZK-Terroir `terroir` contract — Soroban (T3-final, PLAN-DIA-2 §8.2).
//!
//! Reuses the Groth16/BN254 verification pattern proven in the Day-1 spike
//! (`spike/contract/src/lib.rs`): `env.crypto().bn254()` host functions
//! (`g1_mul`, `g1_add`, `pairing_check`). The verifying key is BAKED from the
//! T1-v3 circuit of 3 eslabones (AUDIT-LOG ronda 3 PASA), serialized via
//! `circuits/serialize.js` (swap G2 c1||c0, EIP-197 layout).
//!
//! Public-signal order is FROZEN by Decisión A (PLAN-DIA-2 §2):
//! `[r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo,
//! nullifier_hash]` (7 signals -> VK.ic.len() == 8). Do NOT reorder.
//!
//! Storage (Decisión H): `admin`/`token`/`certifier_root`/`floor_price` in
//! instance storage; `nullifiers`/`lots` as persistent entries with TTL bump.
//! Order (Decisión I): checks -> effects -> interaction (SEP-41 transfer last).
//!
//! Floor binding: `claim_premium` exige `pub_signals[1] == floor_almacenado`
//! (anti inflación de premium). Junto con T1 v3 (commit `price_paid` in
//! `leaf_0`), `premium = price_paid - floor` queda con ambos extremos fijos.

use soroban_sdk::{
    address_payload::AddressPayload,
    contract, contractimpl, contracttype,
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr},
    symbol_short,
    token::TokenClient,
    vec, Address, BytesN, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Types (reused from spike/contract/src/lib.rs)
// ---------------------------------------------------------------------------

/// Groth16 proof passed by the caller. Crosses the contract boundary.
#[derive(Clone)]
#[contracttype]
pub struct Proof {
    pub a: Bn254G1Affine,
    pub b: Bn254G2Affine,
    pub c: Bn254G1Affine,
}

/// Internal verifying key. NOT a `#[contracttype]`: it is built by [`vk`] and
/// consumed by [`groth16_verify`] within a single call, never serialized.
#[derive(Clone)]
pub struct VerificationKey {
    pub alpha: Bn254G1Affine,
    pub beta: Bn254G2Affine,
    pub gamma: Bn254G2Affine,
    pub delta: Bn254G2Affine,
    pub ic: Vec<Bn254G1Affine>,
}

/// Persistent storage keys. Instance globals use `Symbol` keys; per-entry
/// persistent data uses this enum (Decisión H).
#[contracttype]
enum DataKey {
    Nullifier(BytesN<32>),
    Lot(BytesN<32>),
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of public signals (Decisión A: 7).
const N_PUB: u32 = 7;

/// TTL bump for persistent entries (Decisión H): replay protection survives.
/// `extend_ttl` extends only when the remaining TTL is below `threshold`.
const TTL_BUMP_THRESHOLD: u32 = 10_000;
const TTL_BUMP_EXTEND_TO: u32 = 10_000;

const ADMIN: Symbol = symbol_short!("admin");
const TOKEN: Symbol = symbol_short!("token");
const ROOT: Symbol = symbol_short!("root");
const FLOOR: Symbol = symbol_short!("floor");

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct Terroir;

#[contractimpl]
impl Terroir {
    /// One-time initialization. Stores the admin and the SEP-41 token used as
    /// escrow asset (Decisión G). Idempotent guard: panics if already called.
    pub fn init(env: Env, admin: Address, token: Address) {
        let storage = env.storage().instance();
        if storage.has(&ADMIN) {
            panic!("terroir: already initialized");
        }
        storage.set(&ADMIN, &admin);
        storage.set(&TOKEN, &token);
    }

    /// Update the certifier Merkle root `R_cert`. Admin-only (Decisión H).
    pub fn set_certifier_root(env: Env, admin: Address, r_cert: BytesN<32>) {
        admin.require_auth();
        let storage = env.storage().instance();
        let stored: Address = storage
            .get(&ADMIN)
            .unwrap_or_else(|| panic!("terroir: not initialized"));
        if admin != stored {
            panic!("terroir: unauthorized");
        }
        storage.set(&ROOT, &r_cert);
    }

    /// Set the floor price (cents, non-negative `i128`). Admin-only (Decisión H).
    /// `claim_premium` exige `pub_signals[1] == floor_almacenado` (anti inflación
    /// de premium). Must be called before any claim (panics otherwise).
    pub fn set_floor(env: Env, admin: Address, floor_price: i128) {
        admin.require_auth();
        if floor_price < 0 {
            panic!("terroir: floor negative");
        }
        let storage = env.storage().instance();
        let stored: Address = storage
            .get(&ADMIN)
            .unwrap_or_else(|| panic!("terroir: not initialized"));
        if admin != stored {
            panic!("terroir: unauthorized");
        }
        storage.set(&FLOOR, &floor_price);
    }

    /// Claim a premium. Verifies a Groth16 proof over the 7 public signals
    /// (Decisión A order), enforces root + floor + nullifier + payout binding,
    /// then transfers `premium_amount` SEP-41 from this contract to `payout`.
    ///
    /// Order (Decisión I): checks -> effects -> interaction (transfer last).
    pub fn claim_premium(env: Env, proof: Proof, pub_signals: Vec<Fr>, payout: Address) {
        // --- checks ---
        if pub_signals.len() != N_PUB {
            panic!("terroir: expected 7 public signals");
        }
        let r_cert_sig = pub_signals.get(0).unwrap();
        let floor_sig = pub_signals.get(1).unwrap();
        let lot_commit_sig = pub_signals.get(2).unwrap();
        let premium_sig = pub_signals.get(3).unwrap();
        let payout_hi_sig = pub_signals.get(4).unwrap();
        let payout_lo_sig = pub_signals.get(5).unwrap();
        let nullifier_sig = pub_signals.get(6).unwrap();

        // (1) root binding: pub_signals[0] == stored R_cert (Decisión A).
        let instance = env.storage().instance();
        let root: BytesN<32> = instance
            .get(&ROOT)
            .unwrap_or_else(|| panic!("terroir: certifier root not set"));
        if r_cert_sig.to_bytes() != root {
            panic!("terroir: root mismatch");
        }

        // (1b) floor binding: pub_signals[1] == stored floor_price (Decisión A).
        // Pins the floor the prover claims to the admin's published value.
        // Combined with T1 v3 (commit price_paid en leaf_0), premium =
        // price_paid - floor queda con ambos extremos fijados.
        let stored_floor: i128 = instance
            .get(&FLOOR)
            .unwrap_or_else(|| panic!("terroir: floor not set"));
        match fr_to_nonneg_i128(&floor_sig) {
            Some(v) if v == stored_floor => {}
            _ => panic!("terroir: floor mismatch"),
        }

        // amount > 0 (Decisión A: premium_amount = pub_signals[3], i128 cents).
        let premium_u128 = premium_sig
            .as_u256()
            .to_u128()
            .unwrap_or_else(|| panic!("terroir: premium_amount overflow"));
        if premium_u128 == 0 {
            panic!("terroir: amount zero");
        }
        let premium_i128: i128 = if premium_u128 > i128::MAX as u128 {
            panic!("terroir: premium_amount overflow");
        } else {
            premium_u128 as i128
        };

        // (2) nullifier anti-replay check (Decisión A: pub_signals[6]).
        let nullifier_bytes = nullifier_sig.to_bytes();
        let nullifier_key = DataKey::Nullifier(nullifier_bytes.clone());
        let persistent = env.storage().persistent();
        if persistent.has(&nullifier_key) {
            panic!("terroir: nullifier already used");
        }

        // (3) Groth16 verification (BN254, Decisión F, VK horneada en vk()).
        if !verify(&env, &proof, &pub_signals) {
            panic!("terroir: bad proof");
        }

        // (4) payout binding (Decisión E, strong): the 32-byte payload of the
        // `payout` Address, split into hi/lo 16-byte halves, must equal
        // pub_signals[4] and pub_signals[5]. Ligatures the destination of the
        // funds to the proof without Poseidon on-chain.
        let payout_bytes = address_to_bytes(&payout);
        if !check_payout_binding(&payout_hi_sig, &payout_lo_sig, &payout_bytes) {
            panic!("terroir: payout binding failed");
        }

        // --- effects ---
        // insert nullifier (persistent + TTL bump; replay must survive).
        persistent.set(&nullifier_key, &true);
        persistent.extend_ttl(&nullifier_key, TTL_BUMP_THRESHOLD, TTL_BUMP_EXTEND_TO);

        // register lot_commit with claim timestamp (Decisión A: pub_signals[2]).
        let lot_bytes = lot_commit_sig.to_bytes();
        let lot_key = DataKey::Lot(lot_bytes.clone());
        let timestamp = env.ledger().timestamp();
        persistent.set(&lot_key, &timestamp);
        persistent.extend_ttl(&lot_key, TTL_BUMP_THRESHOLD, TTL_BUMP_EXTEND_TO);

        // --- interaction (last; panic here reverts the whole tx, atómico) ---
        let token_addr: Address = instance
            .get(&TOKEN)
            .unwrap_or_else(|| panic!("terroir: token not set"));
        let token = TokenClient::new(&env, &token_addr);
        let me = env.current_contract_address();
        token.transfer(&me, payout.clone(), &premium_i128);
    }

    /// Read endpoint (stretch): returns the claim timestamp for a `lot_commit`,
    /// or `None` if it has never been claimed. Persistent storage.
    pub fn lot_status(env: Env, lot_commit: BytesN<32>) -> Option<u64> {
        env.storage().persistent().get(&DataKey::Lot(lot_commit))
    }
}

// ---------------------------------------------------------------------------
// Groth16 / BN254 verification (reused from spike/contract/src/lib.rs)
// ---------------------------------------------------------------------------

/// Real BN254 Groth16 pairing check. Returns `false` (never panics) when the
/// proof or VK are malformed/wrong — `claim_premium` translates that into
/// `panic!("terroir: bad proof")`.
fn groth16_verify(env: &Env, vk: &VerificationKey, proof: &Proof, pub_signals: &Vec<Fr>) -> bool {
    let bn = env.crypto().bn254();

    // vk_x = ic[0] + sum(pub_signals[i] * ic[i+1])
    if pub_signals.len() + 1 != vk.ic.len() {
        return false;
    }
    let mut vk_x = vk.ic.get(0).unwrap();
    for (s, v) in pub_signals.iter().zip(vk.ic.iter().skip(1)) {
        let prod = bn.g1_mul(&v, &s);
        vk_x = bn.g1_add(&vk_x, &prod);
    }

    // e(-A, B) * e(alpha, beta) * e(vk_x, gamma) * e(C, delta) == 1
    let neg_a = -&proof.a;
    let vp1 = vec![env, neg_a, vk.alpha.clone(), vk_x, proof.c.clone()];
    let vp2 = vec![
        env,
        proof.b.clone(),
        vk.beta.clone(),
        vk.gamma.clone(),
        vk.delta.clone(),
    ];
    bn.pairing_check(vp1, vp2)
}

/// Wrapper called by [`Terroir::claim_premium`]. Routes to the real BN254
/// pairing check (Decisión F) with the T1-v3 verifying key baked into [`vk`].
/// The same path runs in the wasm build and under `cargo test`, so the crypto
/// is exercised by the happy-path / double-spend tests (no `#[cfg(test)]`
/// bypass). Negative tests (bad-root / bad-floor / amount-zero) panic BEFORE
/// this fn (order checks -> effects -> interaction). No cambiar el orden de
/// señales (Decisión A).
fn verify(env: &Env, proof: &Proof, pub_signals: &Vec<Fr>) -> bool {
    let vk = vk(env);
    groth16_verify(env, &vk, proof, pub_signals)
}

// VK real del circuito de 3 eslabones T1 v3 (AUDIT-LOG ronda 3 PASA). Seriada
// con circuits/serialize.js: G1 = be32(x)||be32(y), G2 = Fp2(x)||Fp2(y) con
// Fp2(c) = be32(c1)||be32(c0) (swap c1||c0, EIP-197 layout). 7 publicos ->
// ic.len() == 8 (Decision A). NO verificar pruebas de otro circuito.
fn vk(env: &Env) -> VerificationKey {
    let mut ic = Vec::new(env);
    ic.push_back(g1(env, VK_IC0));
    ic.push_back(g1(env, VK_IC1));
    ic.push_back(g1(env, VK_IC2));
    ic.push_back(g1(env, VK_IC3));
    ic.push_back(g1(env, VK_IC4));
    ic.push_back(g1(env, VK_IC5));
    ic.push_back(g1(env, VK_IC6));
    ic.push_back(g1(env, VK_IC7));
    VerificationKey {
        alpha: g1(env, VK_ALPHA),
        beta: g2(env, VK_BETA),
        gamma: g2(env, VK_GAMMA),
        delta: g2(env, VK_DELTA),
        ic,
    }
}

// ---------------------------------------------------------------------------
// Payout binding helpers (Decisión E)
// ---------------------------------------------------------------------------

/// Fr -> non-negative `i128`. Returns `None` if the value doesn't fit (>= 2^127)
/// — used to compare a public signal (floor) to a stored `i128`.
fn fr_to_nonneg_i128(fr: &Fr) -> Option<i128> {
    let u = fr.as_u256().to_u128()?;
    if u > i128::MAX as u128 {
        return None;
    }
    Some(u as i128)
}

/// Extract the 32-byte payload of an [`Address`] (Ed25519 pubkey for G...,
/// contract hash for C...). Requires the `hazmat-address` SDK feature.
fn address_to_bytes(addr: &Address) -> BytesN<32> {
    match addr.to_payload() {
        Some(AddressPayload::AccountIdPublicKeyEd25519(b)) => b,
        Some(AddressPayload::ContractIdHash(b)) => b,
        None => panic!("terroir: unsupported address type"),
    }
}

/// Verify `payout_hi`/`payout_lo` (each a 16-byte value carried as an `Fr`)
/// reconstruct the 32-byte `addr` payload. Each half must fit in 16 bytes,
/// i.e. the high 16 bytes of the 32-byte big-endian `Fr` repr must be zero.
fn check_payout_binding(hi: &Fr, lo: &Fr, addr: &BytesN<32>) -> bool {
    let hi_arr = hi.to_bytes().to_array();
    let lo_arr = lo.to_bytes().to_array();
    let addr_arr = addr.to_array();
    let mut i = 0;
    while i < 16 {
        // each half must fit in 16 bytes (high 16 bytes zero)
        if hi_arr[i] != 0 || lo_arr[i] != 0 {
            return false;
        }
        // hi[16..32] || lo[16..32] == addr[0..32]
        if hi_arr[16 + i] != addr_arr[i] {
            return false;
        }
        if lo_arr[16 + i] != addr_arr[16 + i] {
            return false;
        }
        i += 1;
    }
    true
}

// ---------------------------------------------------------------------------
// Hex -> BN254 point helpers (used by the baked VK)
// ---------------------------------------------------------------------------

fn decode_hex<const N: usize>(s: &str) -> [u8; N] {
    assert!(s.len() == N * 2, "terroir: hex length mismatch");
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

fn g1(env: &Env, s: &'static str) -> Bn254G1Affine {
    Bn254G1Affine::from_array(env, &decode_hex::<64>(s))
}

fn g2(env: &Env, s: &'static str) -> Bn254G2Affine {
    Bn254G2Affine::from_array(env, &decode_hex::<128>(s))
}

// Real BN254 VK constants (T1 v3, AUDIT-LOG ronda 3). Seriada con circuits/serialize.js
// (swap G2 c1||c0). NO usar para verificar pruebas de otro circuito.
const VK_ALPHA: &str = "248b8d2929640612c3b091a78b17dfc38ce1d4358795877d626f35da1faf8595001c2c2220b4ecf84dcc8042ef164440d365603f33a649443c702d1f7057c68e";
const VK_BETA: &str = "011399a20df3c97093bc93467f6581613b20a58448d73c3cf4e5638aedefc3e72787d2b7bd128d028c95101d1075b55d8230cbb276955bbaf6c8839c1509d20d117dc9d86bdaa7d828c17425c6a23b9c9c0dfba5545643fc6156dda893e0238e2568facb06c95104e9f41619a6366ba58a30221379dea7377ace03f0b2262f46";
const VK_GAMMA: &str = "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
const VK_DELTA: &str = "1d3610d3db69c45b5e39540c87b5ebc5e2188d97d5563ac64dfa4dee16cec7f903d29f89df9783fdb40170840bf71855a76054131eae468e86a1476838964be406415899a6a09d79cecc0bf968684d1df0cac738405f0e9d892c7f819d0efc7826db11cb6793227677311dbd59afd18d43a51be134e4f01111641dbf9695c575";
const VK_IC0: &str = "07cfa6cef5bf51f96427e7a2b5308a5e10ea334c3e0d63ef4cc80fb4f2e212690378817634aa2ac56b1eaceac3ba874f613aaa84f9f414e7fc47b1d5c8578e53";
const VK_IC1: &str = "1f17dd91c21f61cf5b1384dd2c70b52f0ca0d4b4c057c2a70d612f1e3352fc8e14629756ed967152d85ba4bf4168b033ed21797a7378fdd3a811dafb785ce830";
const VK_IC2: &str = "18f15004380580ef0dd0686d118ba6d8e52cf8beef0d84673d3fc42b573099ac09fcb19bb831fb9441d12763976e070bf3399e312e1137325c64649548660054";
const VK_IC3: &str = "29e71e38963469dab4cef5fd36e30b1e17fe3aa041b72a51bbc82123e60233c414a3f579c02a86a8f9289a45390bb8d9ad096ff86c31b8a48a98b3a80e416b27";
const VK_IC4: &str = "2683a539c43bc55b83983877866f3bf441fcc9cb2e9905b31229b7a6bb77d5e116e69ff8e1a844d944a5cc9cc715c375ca918b1019b523f16e9933268419afb2";
const VK_IC5: &str = "14432416a8f651f9c1ee8a1eabecd486c8ff2d99550c98dfba5f0c5ee664b0381d6d5e617fea7b2c307c0ab0207e7e738158ce605c4d6c94749afad3186f7e68";
const VK_IC6: &str = "01f5a307fe208e81b31157c431039db1e0ffc94523f12bafa189b83c65d5b77b2c8084ca8885cd85030d5bebec8be22f882a38e95c64b0a53924d9ea9b019d0e";
const VK_IC7: &str = "256b5a4271cc3edeeebc172fe87f6d64ab2ee568d30f0b16ae32edbbfe599cad12f4602a18d9b602a2689e3bd052429bb2aa10510cd33c68953a2ba4a3a829e9";

#[cfg(test)]
mod test;
