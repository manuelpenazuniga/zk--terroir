// T2 — poseidonFactory.js
// Pequeño wrapper sobre circomlibjs.buildPoseidon para que buildTree.js y
// genWitnessInput.js compartan las MISMAS constantes (round keys, MDS) sin
// duplicar el `require` ni reinicializar wasm.
//
// `circomlibjs` está publicado como ESM y `circomlibjs` en spike/node_modules
// viene como CJS. Apuntamos al path local para evitar versiones distintas.
const path = require('path');
const CIRCOMLIBJS = require(path.join(__dirname, '..', '..', 'spike', 'node_modules', 'circomlibjs'));

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

module.exports = { initPoseidon, CIRCOMLIBJS_PATH: path.join(__dirname, '..', '..', 'spike', 'node_modules', 'circomlibjs') };
