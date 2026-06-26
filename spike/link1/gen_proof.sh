#!/usr/bin/env bash
# Groth16 (bn128) pipeline for terroir_link.circom (Merkle inclusion + nullifier).
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

echo "### input (Poseidon-consistent Merkle path)"
node gen_input.js

echo "### powers of tau (bn128, 2^13)"
snarkjs powersoftau new bn128 13 pot13_0000.ptau -v > /dev/null
snarkjs powersoftau contribute pot13_0000.ptau pot13_0001.ptau --name="t1" -e="terroir-link-1" > /dev/null
snarkjs powersoftau prepare phase2 pot13_0001.ptau pot13_final.ptau -v > /dev/null

echo "### groth16 setup + key contribution"
snarkjs groth16 setup terroir_link.r1cs pot13_final.ptau tl_0000.zkey > /dev/null
snarkjs zkey contribute tl_0000.zkey tl_0001.zkey --name="k1" -e="terroir-link-2" > /dev/null
snarkjs zkey export verificationkey tl_0001.zkey verification_key.json > /dev/null

echo "### witness + proof"
node terroir_link_js/generate_witness.js terroir_link_js/terroir_link.wasm input.json witness.wtns
snarkjs groth16 prove tl_0001.zkey witness.wtns proof.json public.json

echo "### off-chain verify"
snarkjs groth16 verify verification_key.json public.json proof.json

echo "### public.json:"
cat public.json
