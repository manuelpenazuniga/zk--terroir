#!/usr/bin/env bash
# Pipeline Groth16 (bn128) para circuits/terroir_chain.circom — cadena de 3 eslabones (Día 2, T1).
# Entrega: verification_key.json + proof.json + public.json.
# Criterio de aceptación: `snarkjs groth16 verify verification_key.json public.json proof.json` => OK.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

snarkjs() {
  npx snarkjs "$@"
}

CIRC=terroir_chain
PTAU_BITS=15

echo "### 0. compile (circom 2.x, bn128, sin warnings)"
circom ${CIRC}.circom --r1cs --wasm --sym -o .
# sanity de señales públicas: deben ser 7 (Decisión A)
PUB=$(snarkjs r1cs print ${CIRC}.r1cs 2>/dev/null | grep -c "public" || true)
echo "  r1cs+sym+wasm listos"

echo "### 1. input (árbol R_cert Poseidon prof.10 + 3 hojas)"
node gen_input.js

echo "### 2. powers of tau (bn128, 2^${PTAU_BITS})"
snarkjs powersoftau new bn128 ${PTAU_BITS} pot${PTAU_BITS}_0000.ptau -v > /dev/null
snarkjs powersoftau contribute pot${PTAU_BITS}_0000.ptau pot${PTAU_BITS}_0001.ptau \
  --name="t1" -e="terroir-chain-1" > /dev/null
snarkjs powersoftau prepare phase2 pot${PTAU_BITS}_0001.ptau pot${PTAU_BITS}_final.ptau > /dev/null

echo "### 3. groth16 setup + key contribution"
snarkjs groth16 setup ${CIRC}.r1cs pot${PTAU_BITS}_final.ptau ${CIRC}_0000.zkey > /dev/null
snarkjs zkey contribute ${CIRC}_0000.zkey ${CIRC}_0001.zkey --name="k1" -e="terroir-chain-2" > /dev/null
snarkjs zkey export verificationkey ${CIRC}_0001.zkey verification_key.json > /dev/null

echo "### 4. witness + proof"
node ${CIRC}_js/generate_witness.js ${CIRC}_js/${CIRC}.wasm input.json witness.wtns
snarkjs groth16 prove ${CIRC}_0001.zkey witness.wtns proof.json public.json

echo "### 5. off-chain verify (debe imprimir OK)"
snarkjs groth16 verify verification_key.json public.json proof.json

echo "### 6. public.json (orden Decisión A):"
cat public.json
echo
echo "### 7. n inputs públicos en verification_key:"
node -e "console.log('IC.len =', require('./verification_key.json').IC.length)"