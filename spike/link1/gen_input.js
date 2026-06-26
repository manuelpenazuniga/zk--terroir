#!/usr/bin/env node
// Build a Poseidon Merkle path (circomlibjs == circomlib circuit constants, BN254)
// and emit input.json for terroir_link.circom (depth 10).
const fs = require('fs');
const { buildPoseidon } = require('circomlibjs');

(async () => {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;
  const o = (x) => F.toObject(x); // field elem -> BigInt
  const LEVELS = 10;

  // The link's private witness.
  const nullifier = 123456789012345678901234567890n;
  const secret = 111122223333444455556666777788889999n;

  const commitment = o(poseidon([nullifier, secret]));
  const nullifierHash = o(poseidon([nullifier]));

  // Place commitment at a fixed position; deterministic distinct siblings.
  // pathIndices alternate so we exercise both left/right ordering.
  let cur = commitment;
  const pathElements = [];
  const pathIndices = [];
  for (let i = 0; i < LEVELS; i++) {
    const sib = o(poseidon([BigInt(i + 1)])); // a real field element sibling
    const isRight = i % 2; // 0,1,0,1...
    pathElements.push(sib.toString());
    pathIndices.push(isRight);
    cur = isRight ? o(poseidon([sib, cur])) : o(poseidon([cur, sib]));
  }
  const root = cur;

  const input = {
    nullifier: nullifier.toString(),
    secret: secret.toString(),
    pathElements,
    pathIndices,
    root: root.toString(),
    nullifierHash: nullifierHash.toString(),
  };
  fs.writeFileSync(__dirname + '/input.json', JSON.stringify(input, null, 2));
  console.log('commitment   :', commitment.toString());
  console.log('root (R_cert):', root.toString());
  console.log('nullifierHash:', nullifierHash.toString());
  console.log('wrote input.json (public order: [root, nullifierHash])');
})();
