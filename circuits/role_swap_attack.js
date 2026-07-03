#!/usr/bin/env node
// Ola 3 — test adversarial de rol: construir un witness con una hoja cuyo rol
// NO coincide con el slot (p.ej. finca en el slot de tostador, o 2 finca).
// La membership role-tagged en R_cert debe impedir la prueba: el leaf hash no
// matchea la raíz, el witness es RECHAZADO por snarkjs.
//
// Casos:
//  A) slot 2 (TOSTADOR) usa ROLE_FINCA en lugar de ROLE_TOSTADOR -> debe fallar.
//  B) slot 0 (COOP) usa ROLE_TOSTADOR en lugar de ROLE_COOP -> debe fallar.
// Control: roles correctos -> witness válido (gen_input.js normal).
//
// Uso: node circuits/role_swap_attack.js
//   Retorna código 0 si todos los casos adversos fallan (como se espera)
//   y el control pasa. Retorna código 1 si algún caso adverso pasa (BUG).
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const { buildPoseidon } = require('circomlibjs');

const CIRCUITS_DIR = __dirname;
const WASM = path.join(CIRCUITS_DIR, 'terroir_chain_js', 'terroir_chain.wasm');
const NLEAVES = 1 << 10;

const ROLE_FINCA = 1n;
const ROLE_COOP = 2n;
const ROLE_TOSTADOR = 3n;

(async () => {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;
  const o = (x) => F.toObject(x);
  const pose = (a) => o(poseidon(a.map(BigInt)));
  const P = (x) => (typeof x === 'bigint' ? x.toString() : x);

  const inp = JSON.parse(fs.readFileSync(path.join(CIRCUITS_DIR, 'input.json'), 'utf8'));
  const lot_id = BigInt(inp.lot_id);
  const season_id = BigInt(inp.season_id);
  const lot_secret = BigInt(inp.lot_secret);
  const price_paid = BigInt(inp.price_paid);
  const floor_price = BigInt(inp.floor_price);
  const certifier_pk = inp.certifier_pk.map(BigInt);
  const attest_data = inp.attest_data.map(BigInt);

  let failures = 0;
  let passedWhenShouldFail = 0;

  // Construye witness y verifica constrains con snarkjs. Retorna true si OK.
  function tryWitness(inputData, label) {
    const tmpInput = path.join(CIRCUITS_DIR, `input_role_attack_${label}.json`);
    const tmpWitness = path.join(CIRCUITS_DIR, `witness_role_attack_${label}.wtns`);
    fs.writeFileSync(tmpInput, JSON.stringify(inputData, null, 2));
    try {
      execSync(
        `node ${path.join(CIRCUITS_DIR, 'terroir_chain_js', 'generate_witness.js')} ${WASM} ${tmpInput} ${tmpWitness}`,
        { cwd: CIRCUITS_DIR, stdio: 'pipe' }
      );
      return true;
    } catch (e) {
      return false;
    }
  }

  // Helper: construye r_cert con las hojas dadas y emite paths válidos.
  function buildTreeWithLeaves(leaves) {
    let level = new Array(NLEAVES).fill(0n);
    const idxs = [0, 1, 2];
    for (let k = 0; k < idxs.length; k++) level[idxs[k]] = leaves[k];
    // calcula raíz
    let cur = level.slice();
    while (cur.length > 1) {
      const next = new Array(cur.length >> 1);
      for (let j = 0; j < next.length; j++) next[j] = pose([cur[2 * j], cur[2 * j + 1]]);
      cur = next;
    }
    const r_cert = cur[0];
    // paths para los 3 índices
    const paths = idxs.map((index) => {
      const pathElements = [];
      const pathIndices = [];
      let lvl = level.slice();
      let ix = index;
      for (let d = 0; d < 10; d++) {
        const sibIx = ix ^ 1;
        pathElements.push(P(lvl[sibIx]));
        pathIndices.push(ix & 1);
        const next = new Array(lvl.length >> 1);
        for (let j = 0; j < next.length; j++) next[j] = pose([lvl[2 * j], lvl[2 * j + 1]]);
        lvl = next;
        ix >>= 1;
      }
      return { pathElements, pathIndices };
    });
    return { r_cert: P(r_cert), paths };
  }

  // --- Control: roles correctos ---
  console.log('=== CONTROL: roles correctos (debe ser válido) ===');
  const controlLeaves = [
    pose([certifier_pk[0], ROLE_COOP, lot_id, season_id, price_paid, lot_secret]),
    pose([certifier_pk[1], ROLE_FINCA, lot_id, attest_data[0]]),
    pose([certifier_pk[2], ROLE_TOSTADOR, lot_id, attest_data[1]]),
  ];
  const { r_cert: r_cert_ctrl, paths: paths_ctrl } = buildTreeWithLeaves(controlLeaves);
  const ctrlInput = {
    r_cert: r_cert_ctrl,
    floor_price: P(floor_price),
    lot_commit: P(pose([lot_id, season_id])),
    premium_amount: P(price_paid - floor_price),
    payout_hi: inp.payout_hi,
    payout_lo: inp.payout_lo,
    nullifier_hash: P(pose([lot_secret, season_id])),
    lot_id: P(lot_id),
    season_id: P(season_id),
    lot_secret: P(lot_secret),
    price_paid: P(price_paid),
    certifier_pk: certifier_pk.map(P),
    attest_data: attest_data.map(P),
    pathElements: paths_ctrl.map((p) => p.pathElements),
    pathIndices: paths_ctrl.map((p) => p.pathIndices),
  };
  const ctrlOk = tryWitness(ctrlInput, 'ctrl');
  if (ctrlOk) {
    console.log('  CONTROL OK: witness válido (roles correctos).');
  } else {
    console.log('  CONTROL FALLO: witness rechazado con roles correctos (BUG).');
    failures++;
  }

  // --- Caso A: slot 2 (TOSTADOR) usa ROLE_FINCA en lugar de ROLE_TOSTADOR ---
  console.log('');
  console.log('=== ATAQUE A: slot 2 (TOSTADOR) con ROLE_FINCA ===');
  const leavesA = [
    pose([certifier_pk[0], ROLE_COOP, lot_id, season_id, price_paid, lot_secret]),
    pose([certifier_pk[1], ROLE_FINCA, lot_id, attest_data[0]]),
    pose([certifier_pk[2], ROLE_FINCA, lot_id, attest_data[1]]), // <-- rol equivocado
  ];
  const { r_cert: rA, paths: pA } = buildTreeWithLeaves(leavesA);
  const inputA = { ...ctrlInput,
    r_cert: rA,
    pathElements: pA.map((p) => p.pathElements),
    pathIndices: pA.map((p) => p.pathIndices),
  };
  const okA = tryWitness(inputA, 'attackA');
  if (okA) {
    console.log('  PROBLEMA: witness ACEPTADO con rol equivocado en slot 2 (BUG grave).');
    passedWhenShouldFail++;
  } else {
    console.log('  OK: witness RECHAZADO (rol equivocado en slot 2).');
  }

  // --- Caso B: slot 0 (COOP) usa ROLE_TOSTADOR ---
  console.log('');
  console.log('=== ATAQUE B: slot 0 (COOP) con ROLE_TOSTADOR ===');
  const leavesB = [
    pose([certifier_pk[0], ROLE_TOSTADOR, lot_id, season_id, price_paid, lot_secret]), // <-- rol equivocado
    pose([certifier_pk[1], ROLE_FINCA, lot_id, attest_data[0]]),
    pose([certifier_pk[2], ROLE_TOSTADOR, lot_id, attest_data[1]]),
  ];
  const { r_cert: rB, paths: pB } = buildTreeWithLeaves(leavesB);
  const inputB = { ...ctrlInput,
    r_cert: rB,
    pathElements: pB.map((p) => p.pathElements),
    pathIndices: pB.map((p) => p.pathIndices),
  };
  const okB = tryWitness(inputB, 'attackB');
  if (okB) {
    console.log('  PROBLEMA: witness ACEPTADO con rol equivocado en slot 0 (BUG grave).');
    passedWhenShouldFail++;
  } else {
    console.log('  OK: witness RECHAZADO (rol equivocado en slot 0).');
  }

  // --- Caso C: 2 finca (slot 1 + slot 2 ambos ROLE_FINCA) ---
  console.log('');
  console.log('=== ATAQUE C: 2x FINCA (slot 1 y slot 2 ambos ROLE_FINCA) ===');
  const leavesC = [
    pose([certifier_pk[0], ROLE_COOP, lot_id, season_id, price_paid, lot_secret]),
    pose([certifier_pk[1], ROLE_FINCA, lot_id, attest_data[0]]),
    pose([certifier_pk[2], ROLE_FINCA, lot_id, attest_data[1]]), // <-- rol duplicado
  ];
  const { r_cert: rC, paths: pC } = buildTreeWithLeaves(leavesC);
  const inputC = { ...ctrlInput,
    r_cert: rC,
    pathElements: pC.map((p) => p.pathElements),
    pathIndices: pC.map((p) => p.pathIndices),
  };
  const okC = tryWitness(inputC, 'attackC');
  if (okC) {
    console.log('  PROBLEMA: witness ACEPTADO con rol duplicado (2x FINCA) (BUG grave).');
    passedWhenShouldFail++;
  } else {
    console.log('  OK: witness RECHAZADO (rol duplicado).');
  }

  // --- Caso D: rol omitido (slot 1 = TOSTADOR, slot 2 = TOSTADOR, sin FINCA) ---
  console.log('');
  console.log('=== ATAQUE D: sin FINCA (slot 1 y 2 ambos TOSTADOR) ===');
  const leavesD = [
    pose([certifier_pk[0], ROLE_COOP, lot_id, season_id, price_paid, lot_secret]),
    pose([certifier_pk[1], ROLE_TOSTADOR, lot_id, attest_data[0]]),    // <-- omite FINCA
    pose([certifier_pk[2], ROLE_TOSTADOR, lot_id, attest_data[1]]),
  ];
  const { r_cert: rD, paths: pD } = buildTreeWithLeaves(leavesD);
  const inputD = { ...ctrlInput,
    r_cert: rD,
    pathElements: pD.map((p) => p.pathElements),
    pathIndices: pD.map((p) => p.pathIndices),
  };
  const okD = tryWitness(inputD, 'attackD');
  if (okD) {
    console.log('  PROBLEMA: witness ACEPTADO sin rol FINCA (BUG grave).');
    passedWhenShouldFail++;
  } else {
    console.log('  OK: witness RECHAZADO (FINCA omitido).');
  }

  // --- Veredicto ---
  console.log('');
  console.log('========================================');
  console.log(`fallos en control           : ${failures}`);
  console.log(`ataques aceptados (BUG)     : ${passedWhenShouldFail}`);
  if (failures === 0 && passedWhenShouldFail === 0) {
    console.log('VEREDICTO: ROLE-TAG PASA — todos los ataques rechazados, control OK.');
    process.exit(0);
  } else {
    console.log('VEREDICTO: NO-PASA — hay bugs en el aislamiento de roles.');
    process.exit(1);
  }
})().catch((e) => { console.error(e); process.exit(1); });
