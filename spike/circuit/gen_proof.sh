#!/usr/bin/env bash
# Full Groth16 (BN254/bn128) proof generation for the dummy multiplier circuit.
# Produces proof.json, public.json, verification_key.json — the three files the
# Soroban verifier consumes (after serialization to the BN254 byte layout).
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

echo "### 1. compile circuit (bn128 default prime)"
circom multiplier2.circom --r1cs --wasm --sym -o .

echo "### 2. powers of tau (bn128, 2^12)"
snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="spike-1" -v -e="zk-terroir-spike-entropy-1"
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

echo "### 3. groth16 setup + contribute"
snarkjs groth16 setup multiplier2.r1cs pot12_final.ptau mult_0000.zkey
snarkjs zkey contribute mult_0000.zkey mult_0001.zkey --name="spike-key-1" -v -e="zk-terroir-spike-entropy-2"
snarkjs zkey export verificationkey mult_0001.zkey verification_key.json

echo "### 4. witness + proof"
node multiplier2_js/generate_witness.js multiplier2_js/multiplier2.wasm input.json witness.wtns
snarkjs groth16 prove mult_0001.zkey witness.wtns proof.json public.json

echo "### 5. off-chain verify (sanity)"
snarkjs groth16 verify verification_key.json public.json proof.json

echo "### DONE: proof.json / public.json / verification_key.json"
