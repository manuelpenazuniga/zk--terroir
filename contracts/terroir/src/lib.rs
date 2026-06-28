#![no_std]
//! ZK-Terroir `terroir` contract — Soroban skeleton (T3-esqueleto, PLAN-DIA-2 §8.1).
//!
//! Reuses the Groth16/BN254 verification pattern proven in the Day-1 spike
//! (`spike/contract/src/lib.rs`): `env.crypto().bn254()` host functions
//! (`g1_mul`, `g1_add`, `pairing_check`). The verifying key is a PLACEHOLDER
//! until T1 bakes the real `verification_key.json` of the 3-link circuit
//! (see [`vk`] / TODO(T1)).
//!
//! Public-signal order is FROZEN by Decisión A (PLAN-DIA-2 §2):
//! `[r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo,
//! nullifier_hash]` (7 signals → VK.ic.len() == 8). Do NOT reorder.
//!
//! Storage (Decisión H): `admin`/`token`/`certifier_root`/`floor_price` in
//! instance storage; `nullifiers`/`lots` as persistent entries with TTL bump.
//! Order (Decisión I): checks → effects → interaction (SEP-41 transfer last).
//!
//! Floor binding: `claim_premium` exige `pub_signals[1] == floor_almacenado`
//! (anti inflación de premium). NOTE: esto NO corrige AUDIT-LOG H2 (price_paid
//! flota en el circuito → premium = price_paid - floor inflable); H2 requiere
//! el fix del circuito T1 (commit price_paid en leaf_0). El check de contrato
//! es necesario, no suficiente.
//!
//! VK: PLACEHOLDER hasta T3-final (ver [`vk`] / TODO(T3-final)). Requiere T1
//! re-auditado ✅ (AUDIT-LOG H1/H2/H3); PLAN-DIA-2 §8.2.

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
    /// Order (Decisión I): checks → effects → interaction (transfer last).
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
        // NOTE: NO corrige AUDIT-LOG H2 (price_paid flota en el circuito →
        // premium = price_paid - floor sigue inflable); H2 necesita el fix T1
        // (commit price_paid en leaf_0). Check de contrato: necesario, no
        // suficiente.
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

        // (3) Groth16 verification (BN254, reused from spike).
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

/// Real BN254 Groth16 pairing check. Always compiled (used by [`verify`] in
/// the wasm build and by the `#[ignore]` crypto test). Returns `false` (never
/// panics) when the proof or VK are malformed/wrong.
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

/// Wrapper called by [`Terroir::claim_premium`]. Under `cfg(test)` the real
/// pairing check is bypassed so the non-crypto logic (root / floor / nullifier
/// / payout / transfer / lot) is exercisable with a placeholder VK; the crypto
/// path is covered by the `#[ignore]` `test_groth16_with_real_vk` test. The
/// bypass is absent from the wasm build (`cfg(not(test))`).
///
/// TODO(T3-final): ELIMINAR el bypass `#[cfg(test)]` de abajo para que el
/// build wasm y el de tests compartan el MISMO path de pairing real. Requiere
/// T1 re-auditado ✅ (AUDIT-LOG H1/H2/H3) y la VK real horneada en [`vk`].
/// En ese momento el happy-path / doble-cobro cambian a prueba+pub_signals
/// reales (serializa circuits/proof.json + public.json); bad-root / floor-
/// mismatch / amount-zero siguen pasando porque sus checks disparan ANTES de
/// esta fn (orden checks → ... → crypto). No cambiar el orden de señales
/// (Decisión A).
fn verify(env: &Env, proof: &Proof, pub_signals: &Vec<Fr>) -> bool {
    #[cfg(test)]
    {
        // TODO(T3-final): borrar este bypass; rutear a groth16_verify(vk()).
        let _ = (env, proof, pub_signals);
        true
    }
    #[cfg(not(test))]
    {
        let vk = vk(env);
        groth16_verify(env, &vk, proof, pub_signals)
    }
}

// TODO(T3-final): pegar verification_key.json del circuito de 3 eslabones
// (serializado con circuits/serialize.js, swap G2 c1‖c0) reemplazando los
// VK_*_PLACEHOLDER de abajo. Requiere T1 re-auditado ✅ (AUDIT-LOG H1/H2/H3);
// PLAN-DIA-2 §8.2. El placeholder usa puntos BN254 válidos del fixture del
// spike (a*b=c) con `ic` padded a 8 entradas (== nPublic+1 para 7 señales,
// Decisión A). NO verifica ninguna prueba real del circuito de 3 eslabones.
fn vk(env: &Env) -> VerificationKey {
    let mut ic = Vec::new(env);
    ic.push_back(g1(env, VK_IC0_PLACEHOLDER));
    ic.push_back(g1(env, VK_IC1_PLACEHOLDER));
    ic.push_back(g1(env, VK_IC1_PLACEHOLDER));
    ic.push_back(g1(env, VK_IC1_PLACEHOLDER));
    ic.push_back(g1(env, VK_IC1_PLACEHOLDER));
    ic.push_back(g1(env, VK_IC1_PLACEHOLDER));
    ic.push_back(g1(env, VK_IC1_PLACEHOLDER));
    ic.push_back(g1(env, VK_IC1_PLACEHOLDER));
    VerificationKey {
        alpha: g1(env, VK_ALPHA_PLACEHOLDER),
        beta: g2(env, VK_BETA_PLACEHOLDER),
        gamma: g2(env, VK_GAMMA_PLACEHOLDER),
        delta: g2(env, VK_DELTA_PLACEHOLDER),
        ic,
    }
}

// ---------------------------------------------------------------------------
// Payout binding helpers (Decisión E)
// ---------------------------------------------------------------------------

/// Fr → non-negative `i128`. Returns `None` if the value doesn't fit (>= 2^127)
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
// Hex → BN254 point helpers (used by the placeholder VK)
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

// Placeholder BN254 points (spike fixture, a*b=c). TODO(T3-final): replace
// with the real 3-link circuit VK serialized via circuits/serialize.js (swap
// G2 c1‖c0). Requiere T1 re-auditado ✅.
const VK_ALPHA_PLACEHOLDER: &str = "2c804bdc1f03bb45b8cf602491bf04a7ff878b58464fadd4eda4064b2f27bf82286437a0d09cfe3e7e4c74ed9ef5a6ef2a0b2cdfc82b95dda7ba365bd5f60d7e";
const VK_BETA_PLACEHOLDER: &str = "2aaafb97938f9bb81436a827a0cb7ce39035c54689dc18aacaecdade7b1c524e228333fbb43ddbbfaf3c313fc4b4943d58fe587d7301157caaf4a60d0c2bc8b929087a74d646d971bfba7bcc64ae77f3ad7be5945ea533f1c9ebbb940b23785f0cfbcd57bb05e7fecedd62cf13288bddeaca4e872ad81b9a21d07dc3560021bf";
const VK_GAMMA_PLACEHOLDER: &str = "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
const VK_DELTA_PLACEHOLDER: &str = "253eaaa423bd5f4590da530addfb4225ae5bb0d0a2f116081b3e941bc6afb43b080c3cb99362aa593bc01d7e96f2eeccc72d561705f9651c074a889c9589d9411040924a2916c683bf03c0a0f96b3e1e0f88f4536ec11b306bb2b65d42255dde2b608cf090e8ec7a295720bb79af4647c2e907539a50d02d1f843a252fe1efeb";
const VK_IC0_PLACEHOLDER: &str = "1bb12b2426f29bf906ddcb4451d5bf52aa1dd417aa95d796b961df5521f39a77168e2625b0034e271b6d0f5bcb33d57347d85ee994ae57f80019d16b6a30ec81";
const VK_IC1_PLACEHOLDER: &str = "2b5cec58446f697970fd3e14e072e2d5e5ae8f6229ba70a992de6be81e75611c18ec7edebbe1ac0fde6014dfb09573833390be9bc6dd9b52ffc2d283bd0bf63d";

#[cfg(test)]
mod test;
