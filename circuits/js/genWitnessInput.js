#!/usr/bin/env node
// T2 — genWitnessInput.js
// Emite input.json válido para circuits/terroir_chain.circom (T1).
//
// Diseño:
//  - Usa la MISMA factoría Poseidon que buildTree.js (mismas constantes circomlib).
//  - Genera las 3 hojas con la forma EXACTA del circuito (Decisión B post-audit).
//  - Para cada hoja emite pathElements/pathIndices (LEVELS=10) con la convención
//    de MerkleLevel: pathIndices[d]=0 => cur LEFT  (h = Poseidon(cur, sibling)),
//                                          1 => cur RIGHT (h = Poseidon(sibling, cur)).
//  - Privados: lot_id, season_id, lot_secret, price_paid.
//  - Públicos (orden Decisión A):
//      r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash
//  - payout_hi/payout_lo: una pubkey ed25519 de 32B partida en 2×16B BE (128 bits c/u,
//    encajan en BN254). Constraint anti-malleability: h2 = payout_hi^2, l2 = payout_lo^2
//    en el circuito; ambos son Num2Bits(128), así que ningún wraparound de campo.
//
// Salida: circuits/input.json (mismo path que el consumidor actual gen_proof.sh).
const fs   = require('fs');
const path = require('path');
const { initPoseidon } = require('./poseidonFactory.js');

const CIRCUITS_DIR = path.resolve(__dirname, '..', '..', 'circuits');
const OUT_INPUT    = path.join(CIRCUITS_DIR, 'input.json');
const RCERT_JSON   = path.join(__dirname, 'r_cert.json');

(async () => {
  const { pose, P, LEVELS, NLEAVES } = await initPoseidon();

  // ---------------- carga r_cert.json (semilla on-chain) ----------------
  // ATAR la raíz pública al valor sembrado on-chain: si los fixtures difieren
  // entre buildTree y genWitnessInput, este check FALLA explícitamente.
  if (!fs.existsSync(RCERT_JSON)) {
    throw new Error('r_cert.json no existe; ejecuta primero: node circuits/js/buildTree.js');
  }
  const rcertDoc = JSON.parse(fs.readFileSync(RCERT_JSON, 'utf8'));
  const r_cert_loaded = BigInt(rcertDoc.r_cert);

  // ---------------- datos del lote / nullifier (mismos que buildTree) ----------------
  // Mantener coherencia con buildTree: si buildTree usó fixtures, genWitnessInput
  // debe usarlas también (mismo lot_id/price_paid/... o la hoja 0 NO matchea R_cert).
  let FIX = null;
  try { FIX = require(path.join(__dirname, 'fixtures.js')); } catch (_) { /* default */ }

  const lot_id     = (FIX && FIX.lot_id     != null) ? BigInt(FIX.lot_id)     : 7777777777777777n;
  const season_id  = (FIX && FIX.season_id  != null) ? BigInt(FIX.season_id)  : 20262027n;
  const lot_secret = (FIX && FIX.lot_secret != null) ? BigInt(FIX.lot_secret) : 9999999999999999000000000000000000n;
  const floor_price= (FIX && FIX.floor_price!= null) ? BigInt(FIX.floor_price): 1_500_00n;
  const price_paid = (FIX && FIX.price_paid != null) ? BigInt(FIX.price_paid) : 1_875_00n;

  if (price_paid < floor_price) throw new Error('price_paid < floor_price (rompe Decisión D)');
  const premium_amount = price_paid - floor_price;

  const lot_commit     = pose([lot_id, season_id]);       // público (Decisión C)
  const nullifier_hash = pose([lot_secret, season_id]);   // público (Decisión C)

  // ---------------- payout hi/lo: ed25519 pubkey 32B -> 2×16B BE ----------------
  // pubkey de test: 32B hex. La elegimos reproducible (semilla) para que el
  // output.json sea estable entre ejecuciones.
  const pub32 = Buffer.from(
    (FIX && FIX.payout_pub_hex)
      ? FIX.payout_pub_hex
      : '3c0b8a02e3f16b9c4d7e5a3b0c0d6e1f4a2b3c4d5e6f7081920a3b4c5d6e7f81',
    'hex'
  );
  if (pub32.length !== 32) throw new Error('payout_pub_hex debe ser 32 bytes');
  // 16B = 128 bits => BigInt sin signo cabe holgadamente en BN254.
  const payout_hi = BigInt('0x' + pub32.slice(0, 16).toString('hex'));
  const payout_lo = BigInt('0x' + pub32.slice(16, 32).toString('hex'));

  // ---------------- 3 certifiers + leaves (forma EXACTA del circuito) ----------------
  const certifier_pk = (FIX && FIX.certifier_pk) ? FIX.certifier_pk.map(BigInt) : [11n, 22n, 33n];
  const attest_data  = (FIX && FIX.attest_data)  ? FIX.attest_data.map(BigInt)  : [101n, 202n];

  const leaves = [
    pose([certifier_pk[0], lot_id, price_paid, lot_secret]),
    pose([certifier_pk[1], lot_id, attest_data[0]]),
    pose([certifier_pk[2], lot_id, attest_data[1]]),
  ];

  // ---------------- reconstruye el árbol bottom-up ----------------
  // MISMAS reglas que buildTree.js: hoja 0/1/2 en los índices 0/1/2, resto en 0n.
  let level = new Array(NLEAVES).fill(0n);
  const idxs = [0, 1, 2];
  for (let k = 0; k < idxs.length; k++) level[idxs[k]] = leaves[k];

  // Devuelve (pathElements, pathIndices) para un índice dado.
  // Convención MerkleLevel: isRight=0 => cur LEFT  => hash(cur, sibling)
  //                         isRight=1 => cur RIGHT => hash(sibling, cur)
  function merklePath(index) {
    const pathElements = [];
    const pathIndices  = [];
    let cur = level.slice();
    let ix  = index;
    for (let d = 0; d < LEVELS; d++) {
      const sibIx = ix ^ 1;
      pathElements.push(P(cur[sibIx]));
      pathIndices.push(ix & 1);
      const next = new Array(cur.length >> 1);
      for (let j = 0; j < next.length; j++) next[j] = pose([cur[2 * j], cur[2 * j + 1]]);
      cur = next;
      ix >>= 1;
    }
    return { pathElements, pathIndices };
  }

  // ---------------- raíz (bottom-up) ----------------
  let cur = level.slice();
  while (cur.length > 1) {
    const next = new Array(cur.length >> 1);
    for (let j = 0; j < next.length; j++) next[j] = pose([cur[2 * j], cur[2 * j + 1]]);
    cur = next;
  }
  const r_cert = cur[0];

  // Sanity #1: la raíz de genWitnessInput DEBE coincidir con la de buildTree.
  // Si los fixtures cambian entre llamadas, este check atrapa el mismatch
  // antes de generar un witness inválido.
  if (P(r_cert) !== P(r_cert_loaded)) {
    throw new Error(`r_cert mismatch: genWitnessInput=${P(r_cert)} buildTree=${P(r_cert_loaded)}. ¿fixtures.js cambió entre llamadas?`);
  }

  // Sanity #2: cada path debe re-derivar la raíz in-código (anti-tampering).
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

  // ---------------- emite input.json (orden Decisión A EXACTO) ----------------
  const input = {
    // públicos (Decisión A)
    r_cert:         P(r_cert),
    floor_price:    P(floor_price),
    lot_commit:     P(lot_commit),
    premium_amount: P(premium_amount),
    payout_hi:      P(payout_hi),
    payout_lo:      P(payout_lo),
    nullifier_hash: P(nullifier_hash),

    // privados
    lot_id:     P(lot_id),
    season_id:  P(season_id),
    lot_secret: P(lot_secret),
    price_paid: P(price_paid),

    certifier_pk: certifier_pk.map(P),
    attest_data:  attest_data.map(P),  // 2 entradas (eslabones 1,2)
    pathElements: paths.map((p) => p.pathElements),
    pathIndices:  paths.map((p) => p.pathIndices),
  };
  fs.writeFileSync(OUT_INPUT, JSON.stringify(input, null, 2));

  console.log('r_cert         :', P(r_cert));
  console.log('lot_commit     :', P(lot_commit));
  console.log('nullifier_hash :', P(nullifier_hash));
  console.log('premium_amount :', P(premium_amount));
  console.log('leaves         :', leaves.map(P));
  console.log('wrote input.json (public order Decisión A)');
})().catch((e) => { console.error(e); process.exit(1); });
