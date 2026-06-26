#![cfg(test)]
extern crate std;

use soroban_sdk::{
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr},
    BytesN, Env, Vec,
};

use crate::fixture;
use crate::{Groth16Verifier, Groth16VerifierClient, Proof, VerificationKey};

/// Decode a hex string (no 0x prefix) of exactly 2*N chars into [u8; N].
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

fn load_vk(env: &Env) -> VerificationKey {
    let mut ic = Vec::new(env);
    for &s in fixture::VK_IC {
        ic.push_back(g1(env, s));
    }
    VerificationKey {
        alpha: g1(env, fixture::VK_ALPHA),
        beta: g2(env, fixture::VK_BETA),
        gamma: g2(env, fixture::VK_GAMMA),
        delta: g2(env, fixture::VK_DELTA),
        ic,
    }
}

fn load_proof(env: &Env) -> Proof {
    Proof {
        a: g1(env, fixture::PROOF_A),
        b: g2(env, fixture::PROOF_B),
        c: g1(env, fixture::PROOF_C),
    }
}

#[test]
fn test_bn254_groth16_verifies() {
    let env = Env::default();
    let client = Groth16VerifierClient::new(&env, &env.register(Groth16Verifier {}, ()));

    let vk = load_vk(&env);
    let proof = load_proof(&env);

    // Correct public input (33) -> verifies.
    let mut signals = Vec::new(&env);
    for &s in fixture::PUB_SIGNALS {
        signals.push_back(fr(&env, s));
    }
    let res = client.verify_proof(&vk, &proof, &signals);
    assert_eq!(res, true);

    // Authoritative BN254 cost breakdown.
    env.cost_estimate().budget().print();

    // Wrong public input (22 = 0x16) -> rejects.
    let mut bad = Vec::new(&env);
    bad.push_back(fr(
        &env,
        "0000000000000000000000000000000000000000000000000000000000000016",
    ));
    let res2 = client.verify_proof(&vk, &proof, &bad);
    assert_eq!(res2, false);
}
