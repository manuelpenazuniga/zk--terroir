// T2 — poseidonFactory.js
// Pequeño wrapper sobre circomlibjs.buildPoseidon para que buildTree.js y
// genWitnessInput.js compartan las MISMAS constantes (round keys, MDS) sin
// duplicar el `require` ni reinicializar wasm.
//
// circomlibjs se resuelve desde circuits/node_modules (pin EXACTO 0.1.7 en package.json,
// `npm ci` = paso 0). Mismo par circomlib 2.0.5 / circomlibjs 0.1.7 que produjo la VK.
const path = require('path');
const CIRCOMLIBJS = require('circomlibjs');

/**
 * Inicializa Poseidon(BN254) con las constantes idénticas a circomlib
 * (poseidon.circom). Devuelve un objeto { pose, o, F, LEVELS, P, NLEAVES }.
 */
async function initPoseidon() {
  const { buildPoseidon } = CIRCOMLIBJS;
  const poseidon = await buildPoseidon();
  const F = poseidon.F;
  const o  = (x) => F.toObject(x);
  const pose = (arr) => o(poseidon(arr.map(BigInt)));
  const P = (x) => (typeof x === 'bigint' ? x.toString() : x);
  const LEVELS  = 10;
  const NLEAVES = 1 << LEVELS;
  return { pose, o, F, P, LEVELS, NLEAVES };
}

module.exports = { initPoseidon, CIRCOMLIBJS_PATH: require.resolve('circomlibjs') };
