#![no_std]
//! ZK-Terroir `terroir` contract — Soroban (T3-final, PLAN-DIA-2 §8.2).
//!
//! Reuses the Groth16/BN254 verification pattern proven in the Day-1 spike
//! (`spike/contract/src/lib.rs`): `env.crypto().bn254()` host functions
//! (`g1_mul`, `g1_add`, `pairing_check`). The verifying key is BAKED from the
//! T1 Ola-3 circuit (role-tag, leaf arities 6/4/4), serialized via
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

// VK real del circuito de 3 eslabones T1 Ola-3 (role-tag, arities 6/4/4).
// Seriada con circuits/serialize.js: G1 = be32(x)||be32(y), G2 = Fp2(x)||Fp2(y) con
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
const VK_ALPHA: &str = "2697d44eb8a797ae4b750b90d92ee5fbbd1ada8dfe6878cb38aad5130414ee800cfd707c32cf439c472e924a31ffaf770336066f071a4445e8fa18f65baa5894";
const VK_BETA: &str = "279c6100e0c293e7bce2488d868a67d9269c7371e68f4e25fad5655bdcf84b10086d3fdd2fb31f1eda1b23dd42ce6e1bd21975d0bfd2d75c0c5579ce882128bf183e86cdad29f7145a064aeb804ec1af20177e83a0bfb50be3e46770629f072b1e148cd719687f161ba97d558d8b4df3fcbdc4478560a338429cc837ff2f2da9";
const VK_GAMMA: &str = "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa";
const VK_DELTA: &str = "0a963efb3318dfacd1fa1adc03248166e10dcc20d529f567ce98196fd68828e509dd11771e6d0c973a8b7ed696cf694e81c92ea6022864876e61ce227d0e0e6126522bbdbb08cb717f8b71b5707cdb71c677b8b14d5de6f98e7fcf45e1440e7007f2ce155145ce7bf4bfcba16a39b556f6ae0f480ab2f779568122d8fbc0872d";
const VK_IC0: &str = "155551d896ed813908afd6ff29293dfb87254c3626675b43f51c13439ede32af0d8ece81350e0e9e9f408c3cfe68dd94f43900cbb4c24d4d50e2232cb962e783";
const VK_IC1: &str = "26653b97aefd2ebafd9816ab186fc7d68d52ba5908a349fbb1f1e8acdf02f3522710622c403580aa511d3cc16f98b6ccdc4cc8f2b7f08aacde6c5878216478d7";
const VK_IC2: &str = "271648728acf99c10c85ebb760b268968e655d2daa2a00387b3b746ae993e88c0c650e77e96894e5f5be94573df4206642a1b6a36484922511b4f71152e427e5";
const VK_IC3: &str = "1991e61c44cf3e74ec165e297699b4cefa0c935988bc58bff0436db776711903231fd5b04b716b2cc6b4e04f10d00d1445aee6212045df51ad2c71073be935f0";
const VK_IC4: &str = "21feae4e43c830076f9a40d917f84b914e7f834b5b1adb52d8f6f6b35b8e2e7d1c8ecbd4dc893eb87faff6387f1773426cc1a8ad369bec06f8b662aef5cc5b60";
const VK_IC5: &str = "0679e3fc28aa10871ff790bf234cd6d990e65b14473a7bff4b6b30de0ae078a9077f2ff556948cfbc5f50ed16eaad8e084fe8c06d1954dd053079314f1dfd1e1";
const VK_IC6: &str = "0a1fcd6001f4f99509ccbd1af86de739809a6e5ac7cc72d3925e65082fc06c750174c41773f676d94acf212e87046c3c63e9926e9d9625376ad8adf6f25768b6";
const VK_IC7: &str = "12667b46825464d48a0fad3c7ee26dd028661c27a235f242d682276f191d9df528990507f1e0a459ca6d4b69cc5ffbaf16b9205fd547e7c128e9fc243c57609b";

#[cfg(test)]
mod test;
