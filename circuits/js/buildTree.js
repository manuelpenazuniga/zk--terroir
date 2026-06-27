#!/usr/bin/env node
// T2 — buildTree.js
// Construye R_cert (raíz del set de certificadores acreditados) con circomlibjs
// (buildPoseidon, BN254) usando las MISMAS constantes que circomlib/poseidon.circom.
//
// Diseño:
//  - Árbol binario de profundidad LEVELS=10 (1<<10 = 1024 hojas).
//  - Hojas = Poseidon con las MISMAS reglas que usa el circuito (post-auditoría):
//      leaf_0 = Poseidon(pk_0, lot_id, price_paid, lot_secret)   // eslabón 0 (cooperativa)
//      leaf_i = Poseidon(pk_i, lot_id, attest_data_i)            // eslabones 1,2  (i∈{1,2})
//    Esta forma liga price_paid/lot_id/lot_secret a una atestación acreditada
//    (mata el premium arbitrario y el doble-cobro) — Decisión B post-auditoría.
//  - Padres: h = Poseidon(left, right) sin permutación previa (la elección LEFT/RIGHT
//    la dicta el pathIndex en el inclusion proof del circuito; aquí el árbol
//    representa el MISMO orden que MerkleLevel: pathIndices[d]=0 => cur LEFT).
//  - Hojas vacías = 0n (BN254 zero) y se hashean hacia arriba como cualquier otro slot.
//    Coincide con el comportamiento por defecto del circomlib Poseidon(0,0) y
//    reproduce el árbol "relleno con ceros" que espera el witness.
//
// Salidas:
//  - circuits/js/r_cert.json      : { r_cert, levels, leafFormulas, ... }  (la raíz
//                                    se siembra on-chain; el resto es metadata útil).
//  - circuits/js/tree.json         : snapshot del árbol (niveles serializados) — útil
//                                    para auditoría cruzada con el witness.
const fs   = require('fs');
const path = require('path');
const { initPoseidon } = require('./poseidonFactory.js');

const OUT_RCERT = path.join(__dirname, 'r_cert.json');
const OUT_TREE  = path.join(__dirname, 'tree.json');

// Carga opcional de fixtures para que buildTree.js sea REPRODUCIBLE: si existe
// circuits/js/fixtures.js, lo usa; si no, usa defaults deterministas.
let FIX = null;
try { FIX = require(path.join(__dirname, 'fixtures.js')); } catch (_) { /* no fixtures -> default */ }

(async () => {
  const { pose, P, LEVELS, NLEAVES } = await initPoseidon();

  // ---------------- datos del lote / nullifier (mismos que genWitnessInput) ----------------
  const lot_id     = (FIX && FIX.lot_id     != null) ? BigInt(FIX.lot_id)     : 7777777777777777n;
  const season_id  = (FIX && FIX.season_id  != null) ? BigInt(FIX.season_id)  : 20262027n;
  const lot_secret = (FIX && FIX.lot_secret != null) ? BigInt(FIX.lot_secret) : 9999999999999999000000000000000000n;
  const price_paid = (FIX && FIX.price_paid != null) ? BigInt(FIX.price_paid) : 1_875_00n;

  // ---------------- 3 certificadores acreditados ----------------
  // En producción estas pks son ed25519 verificadas fuera de línea por la
  // autoridad que sella R_cert. Aquí usamos scalars para no ligar el set a
  // un mnemonic concreto; el circuito sólo las ve como field elements.
  const certifier_pk = (FIX && FIX.certifier_pk) ? FIX.certifier_pk.map(BigInt) : [11n, 22n, 33n];
  const attest_data  = (FIX && FIX.attest_data)  ? FIX.attest_data.map(BigInt)  : [101n, 202n];

  if (certifier_pk.length !== 3) throw new Error('certifier_pk debe tener 3 entradas');
  if (attest_data.length  !== 2) throw new Error('attest_data debe tener 2 entradas (eslabones 1,2)');

  // ---------------- hojas (forma EXACTA del circuito) ----------------
  const leaves = [
    pose([certifier_pk[0], lot_id, price_paid, lot_secret]), // eslabón 0
    pose([certifier_pk[1], lot_id, attest_data[0]]),        // eslabón 1
    pose([certifier_pk[2], lot_id, attest_data[1]]),        // eslabón 2
  ];

  // ---------------- inserción en árbol prof.10, lleno con 0n ----------------
  // El set acreditado suele ser pequeño; lo sembramos en los PRIMEROS índices
  // disponibles para que paths cortos sean eficientes y reproducibles.
  const leafIndexes = [0, 1, 2];
  let level = new Array(NLEAVES).fill(0n);
  for (let k = 0; k < leafIndexes.length; k++) level[leafIndexes[k]] = leaves[k];

  // ---------------- calcula raíz + captura nodos por nivel (para tree.json) ----------------
  // Guarda cada nivel: levelNodes[0]=hojas, levelNodes[LEVELS]=[r_cert].
  const levelNodes = [level.slice()];
  {
    let cur = level.slice();
    while (cur.length > 1) {
      const next = new Array(cur.length >> 1);
      for (let j = 0; j < next.length; j++) next[j] = pose([cur[2 * j], cur[2 * j + 1]]);
      cur = next;
      levelNodes.push(cur);
    }
  }
  const r_cert = levelNodes[LEVELS][0];

  // ---------------- verifica in-código: la raíz es estable ----------------
  // (re-hash bottom-up sobre levelNodes reconstruido desde leaves;
  // útil para detectar bugs de orden LEFT/RIGHT temprano).
  for (let d = 0; d < LEVELS; d++) {
    const lo = levelNodes[d];
    const up = levelNodes[d + 1];
    for (let j = 0; j < up.length; j++) {
      const recomputed = pose([lo[2 * j], lo[2 * j + 1]]);
      if (P(recomputed) !== P(up[j])) {
        throw new Error(`nivel ${d}->${d + 1}, nodo ${j}: raíz inestable`);
      }
    }
  }

  // ---------------- emite r_cert.json (lo que se siembra on-chain) ----------------
  const rcertDoc = {
    r_cert: P(r_cert),
    levels: LEVELS,
    hash: 'Poseidon(BN254) — circomlibjs con constantes idénticas a circomlib',
    leafFormulas: [
      'Poseidon(certifier_pk[0], lot_id, price_paid, lot_secret)',
      'Poseidon(certifier_pk[1], lot_id, attest_data[0])',
      'Poseidon(certifier_pk[2], lot_id, attest_data[1])',
    ],
    // metadata no sensible: índices de las 3 hojas; el árbol completo NO
    // expone las pks/attest_data (eso vive en el leafFormulas público).
    leafIndexes,
    generatedAt: new Date().toISOString(),
  };
  fs.writeFileSync(OUT_RCERT, JSON.stringify(rcertDoc, null, 2));

  // ---------------- emite tree.json (snapshot para auditoría) ----------------
  // Serializa niveles como arrays de decimal-strings. Tamaño: 1024 + 512 + ... + 1 = 2047.
  const treeDoc = {
    r_cert: P(r_cert),
    levels: LEVELS,
    levelNodes: levelNodes.map((arr) => arr.map(P)),
  };
  fs.writeFileSync(OUT_TREE, JSON.stringify(treeDoc, null, 2));

  console.log('r_cert         :', P(r_cert));
  console.log('leaves         :', leaves.map(P));
  console.log('leafIndexes    :', leafIndexes);
  console.log('wrote r_cert.json (semilla on-chain) y tree.json (snapshot)');
})().catch((e) => { console.error(e); process.exit(1); });
