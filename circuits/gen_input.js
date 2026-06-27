#!/usr/bin/env node
// Genera input.json válido para circuits/terroir_chain.circom (Día 2, T1).
// Construye R_cert: árbol Merkle Poseidon(BN254) prof.10 con MISMAS constantes que circomlib,
// inserta 3 hojas leaf_i = Poseidon(certifier_pk_i, attest_data_i) en índices distintos y
// emite sus Merkle paths. Verifica todas las derivaciones públicas (Decisión A).
const fs = require('fs');
const path = require('path');
const { buildPoseidon } = require('../spike/node_modules/circomlibjs');

const LEVELS = 10;
const NLEAVES = 1 << LEVELS; // 1024
const P = (x) => x.toString();

(async () => {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;
  const o = (x) => F.toObject(x);
  const pose = (arr) => o(poseidon(arr.map(BigInt)));

  // ----- privados del lote / nullifier -----
  const lot_id    = 7777777777777777n;
  const season_id = 20262027n;
  const lot_secret = 9999999999999999000000000000000000n;

  const lot_commit     = pose([lot_id, season_id]);           // público (Decisión C)
  const nullifier_hash = pose([lot_secret, season_id]);      // público (Decisión C)

  // ----- range / premium -----
  // Montos en centavos. price_paid >= floor_price; premium_amount == price_paid - floor_price.
  const floor_price    = 1_500_00n;   // 1500.00 USDC
  const price_paid     = 1_875_00n;   // 1875.00 USDC
  const premium_amount = price_paid - floor_price;           // 375.00 USDC (público)

  // ----- payout hi/lo: 32-byte pubkey ed25519 partida en 2x16B (decoy de test) -----
  // 16B = 128 bits -> encaja en BN254 <p.
  const pub32 = Buffer.from(
    '3c0b8a02e3f16b9c4d7e5a3b0c0d6e1f4a2b3c4d5e6f7081920a3b4c5d6e7f81',  // 32 bytes (64 hex)
    'hex'
  );
  if (pub32.length !== 32) throw new Error('pubkey debe ser 32 bytes');
  const payout_hi = BigInt('0x' + pub32.slice(0, 16).toString('hex'));  // 128 bits BE
  const payout_lo = BigInt('0x' + pub32.slice(16, 32).toString('hex'));

  // ----- 3 certifiers + leaves del eslabón (post-auditoría) -----
  // Decisión B post-audit:
  //  - eslabón 0 (cooperativa): leaf_0 = Poseidon(pk_0, lot_id, price_paid, lot_secret)
  //    -> liga price_paid, lot_id, lot_secret a una atestación acreditada.
  //  - eslabones 1,2: leaf_i = Poseidon(pk_i, lot_id, attest_data_i)
  //    -> todos atestan el MISMO lot_id (no hay hash-chain; cadena eliminada para MVP).
  const certifier_pk = [11n, 22n, 33n];
  const attest_data  = [101n, 202n]; // sólo 2 (eslabones 1 y 2)
  const leaves = [
    pose([certifier_pk[0], lot_id, price_paid, lot_secret]),
    pose([certifier_pk[1], lot_id, attest_data[0]]),
    pose([certifier_pk[2], lot_id, attest_data[1]]),
  ];

  // ----- Merkle tree (Poseidon(2), BN254), filled with zeros -----
  // Estructura por niveles: leaves first, then computed parents.
  const idxs = [0, 1, 2]; // posiciones donde van las 3 hojas reales
  let level = new Array(NLEAVES).fill(0n);
  for (let k = 0; k < idxs.length; k++) level[idxs[k]] = leaves[k];

  // Devuelve pathElements/pathIndices para un índice dado Desde la hoja hasta la raíz.
  function merklePath(index) {
    const pathElements = [];
    const pathIndices = [];
    let cur = level.slice();
    let ix = index;
    for (let d = 0; d < LEVELS; d++) {
      const sibIx = ix ^ 1;
      pathElements.push(P(cur[sibIx]));
      pathIndices.push(ix & 1); // 0 -> cur es LEFT, 1 -> cur es RIGHT (igual que MerkleLevel)
      // sube un nivel
      const next = new Array(cur.length >> 1);
      for (let j = 0; j < next.length; j++) {
        next[j] = pose([cur[2 * j], cur[2 * j + 1]]);
      }
      cur = next;
      ix >>= 1;
    }
    return { pathElements, pathIndices };
  }

  const r_cert = (() => {
    let cur = level.slice();
    while (cur.length > 1) {
      const next = new Array(cur.length >> 1);
      for (let j = 0; j < next.length; j++) next[j] = pose([cur[2 * j], cur[2 * j + 1]]);
      cur = next;
    }
    return cur[0];
  })();

  // sanity de paths: recomprobar raíz in-código para LOS 3.
  for (let i = 0; i < 3; i++) {
    const { pathElements, pathIndices } = merklePath(idxs[i]);
    let c = leaves[i];
    for (let d = 0; d < LEVELS; d++) {
      const sib = BigInt(pathElements[d]);
      c = pathIndices[d] === 0 ? pose([c, sib]) : pose([sib, c]);
    }
    if (P(c) !== P(r_cert)) throw new Error(`path ${i} NO recalcula la raíz`);
  }

  const paths = idxs.map(merklePath);

  const input = {
    // públicos (Decisión A, orden EXACTO)
    r_cert: P(r_cert),
    floor_price: P(floor_price),
    lot_commit: P(lot_commit),
    premium_amount: P(premium_amount),
    payout_hi: P(payout_hi),
    payout_lo: P(payout_lo),
    nullifier_hash: P(nullifier_hash),

    // privados
    lot_id: P(lot_id),
    season_id: P(season_id),
    lot_secret: P(lot_secret),
    price_paid: P(price_paid),

    certifier_pk: certifier_pk.map(P),
    attest_data: attest_data.map(P), // 2 entradas (eslabones 1,2)
    pathElements: paths.map((p) => p.pathElements),
    pathIndices: paths.map((p) => p.pathIndices),
  };

  fs.writeFileSync(path.join(__dirname, 'input.json'), JSON.stringify(input, null, 2));

  console.log('r_cert         :', P(r_cert));
  console.log('lot_commit      :', P(lot_commit));
  console.log('nullifier_hash  :', P(nullifier_hash));
  console.log('premium_amount  :', P(premium_amount));
  console.log('leaves          :', leaves.map(P));
  console.log('wrote input.json (public order Decisión A)');
})();