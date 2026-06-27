pragma circom 2.1.0;

// Cadena de 3 eslabones (Día 2, T1).
// Reusa MerkleLevel / MerkleInclusion de spike/link1/terroir_link.circom
// (copiados in-situ para NO tocar el spike ni crear doble `component main`).
include "../spike/node_modules/circomlib/circuits/poseidon.circom";
include "../spike/node_modules/circomlib/circuits/mux1.circom";
include "../spike/node_modules/circomlib/circuits/comparators.circom";
include "../spike/node_modules/circomlib/circuits/bitify.circom";

// --- Reutilizado de spike/link1/terroir_link.circom (sin modificaciones) ---
template MerkleLevel() {
    signal input cur;
    signal input sibling;
    signal input isRight;
    signal output out;

    isRight * (isRight - 1) === 0;

    component mux = MultiMux1(2);
    mux.c[0][0] <== cur;      mux.c[0][1] <== sibling;
    mux.c[1][0] <== sibling;  mux.c[1][1] <== cur;
    mux.s <== isRight;

    component h = Poseidon(2);
    h.inputs[0] <== mux.out[0];
    h.inputs[1] <== mux.out[1];
    out <== h.out;
}

template MerkleInclusion(levels) {
    signal input leaf;
    signal input root;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    component lvl[levels];
    signal cur[levels + 1];
    cur[0] <== leaf;
    for (var i = 0; i < levels; i++) {
        lvl[i] = MerkleLevel();
        lvl[i].cur <== cur[i];
        lvl[i].sibling <== pathElements[i];
        lvl[i].isRight <== pathIndices[i];
        cur[i + 1] <== lvl[i].out;
    }
    root === cur[levels];
}
// ---------------------------------------------------------------------------

// Decisión B (post-auditoría):
//   Eslabón 0 — cooperativa — liga price_paid, lot_id y lot_secret a una atestación acreditada:
//     leaf_0 = Poseidon(certifier_pk_0, lot_id, price_paid, lot_secret) in R_cert
//     (mata premium arbitrario y el doble-cobro: lot_secret se vuelve único-verificable por lote).
//   Eslabones 1 y 2: meten lot_id en la hoja -> todos atestan el MISMO lote:
//     leaf_i = Poseidon(certifier_pk_i, lot_id, attest_data_i) in R_cert   (i=1,2)
// Decisión 3 (custodia): la hash-chain del spike (chain_{i+1}=Poseidon(chain_i, leaf_i))
//   se ELIMINA para el MVP. Con lot_id metido en CADA hoja (punto 2), las 3 atestaciones ya
//   están atadas al mismo lote; el nullifier_hash (C) cubre el replay-protection; no hay
//   ordinal Nina dependencia entre eslabones que requiera la cadena. Exponer chain[3]
//   exigiría o revelar attestation_data (rompe privacidad) o añadir un 8º público (rompe
//   Decisión A). Quitar la cadena también evita la señal chain[3] muerta del audit.
//   Stretch (Día 3, si se requiere ordenar certificaciones): revivir la cadena y atar
//   chain[3] a un commitment público añadido conVK nueva (NO HOY).
// Decisión C: lot_commit = Poseidon(lot_id, season_id); nullifier_hash = Poseidon(lot_secret, season_id).
// Decisión D: range price_paid >= floor_price (GreaterEqThan(64) con Num2Bits en ambos).
// Decisión E: payout_hi/payout_lo públicos ligados (anti-malleability + rango 128 bits).
// Decisión A: orden público EXACTO
//   [r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash]
template TerroirChain(levels) {

    // --- inputs públicos (Decisión A, orden congelado) ---
    signal input r_cert;
    signal input floor_price;
    signal input lot_commit;
    signal input premium_amount;
    signal input payout_hi;
    signal input payout_lo;
    signal input nullifier_hash;

    // --- inputs privados ---
    signal input lot_id;
    signal input season_id;
    signal input lot_secret;
    signal input price_paid;

    // 3 eslabones: certifier_pk[3] (uno por eslabón). attest_data[2] alimenta
    // los eslabones 1,2 (el eslabón 0 = cooperativa NO lleva attest_data: su
    // payload son (lot_id, price_paid, lot_secret) — ver leafH0 abajo).
    signal input certifier_pk[3];
    signal input attest_data[2];
    signal input pathElements[3][levels];
    signal input pathIndices[3][levels];

    // ----- Decisión C: lot_commit = Poseidon(lot_id, season_id) -----
    component lc = Poseidon(2);
    lc.inputs[0] <== lot_id;
    lc.inputs[1] <== season_id;
    lot_commit === lc.out;

    // ----- Decisión C: nullifier_hash = Poseidon(lot_secret, season_id) -----
    component nh = Poseidon(2);
    nh.inputs[0] <== lot_secret;
    nh.inputs[1] <== season_id;
    nullifier_hash === nh.out;

    // ----- Decisión D: range price_paid, floor_price < 2^64 -----
    component nbPrice = Num2Bits(64);
    nbPrice.in <== price_paid;
    component nbFloor = Num2Bits(64);
    nbFloor.in <== floor_price;

    // premium = price_paid - floor_price (no negativo por el range de abajo).
    signal premium;
    premium <== price_paid - floor_price;

    // premium (público) ligado al calculado y de rango 64 bits.
    premium_amount === premium;
    component nbPremium = Num2Bits(64);
    nbPremium.in <== premium_amount;

    // ----- Decisión D: GreaterEqThan(64) sobre price_paid >= floor_price -----
    component ge = GreaterEqThan(64);
    ge.in[0] <== price_paid;
    ge.in[1] <== floor_price;
    ge.out === 1;

    // ----- Decisión E: binding payout hi/lo (16B = 128b) -----
    // Rango explícito anti-mallas (no field-wraparound).
    component nbHi = Num2Bits(128);
    nbHi.in <== payout_hi;
    component nbLo = Num2Bits(128);
    nbLo.in <== payout_lo;
    // Constraint anti-malleabilidad (Decisión E, indicado por arquitecto).
    signal h2;
    h2 <== payout_hi * payout_hi;
    signal l2;
    l2 <== payout_lo * payout_lo;

    // ----- Decisión B (post-auditoría): 3 memberships en R_cert -----
    // Eslabón 0 (cooperativa): leaf_0 = Poseidon(certifier_pk_0, lot_id, price_paid, lot_secret)
    //   -> liga price_paid, lot_id, lot_secret a una atestación acreditada (no hay más
    //      libertad para elegimos premium arbitrario o reusar lot_secret).
    // Eslabones 1,2: leaf_i = Poseidon(certifier_pk_i, lot_id, attest_data_i)
    //   -> todos atestan el MISMO lot_id, sin cadena.
    component leafH0 = Poseidon(4);
    leafH0.inputs[0] <== certifier_pk[0];
    leafH0.inputs[1] <== lot_id;
    leafH0.inputs[2] <== price_paid;
    leafH0.inputs[3] <== lot_secret;

    component inc0 = MerkleInclusion(levels);
    inc0.leaf <== leafH0.out;
    inc0.root <== r_cert;
    for (var j = 0; j < levels; j++) {
        inc0.pathElements[j] <== pathElements[0][j];
        inc0.pathIndices[j] <== pathIndices[0][j];
    }

    component leafH[2];
    component inc[2];
    for (var i = 0; i < 2; i++) {
        leafH[i] = Poseidon(3);
        leafH[i].inputs[0] <== certifier_pk[i + 1];
        leafH[i].inputs[1] <== lot_id;
        leafH[i].inputs[2] <== attest_data[i];

        inc[i] = MerkleInclusion(levels);
        inc[i].leaf <== leafH[i].out;
        inc[i].root <== r_cert;
        for (var j = 0; j < levels; j++) {
            inc[i].pathElements[j] <== pathElements[i + 1][j];
            inc[i].pathIndices[j] <== pathIndices[i + 1][j];
        }
    }
}

component main { public [r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash] } = TerroirChain(10);