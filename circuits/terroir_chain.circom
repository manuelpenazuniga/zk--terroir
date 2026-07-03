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

// Decisión B (post-auditoría v3):
//   Eslabón 0 — cooperativa — fija lot_id, season_id, price_paid y lot_secret a una
//   atestación acreditada:
//     leaf_0 = Poseidon(certifier_pk_0, ROLE_COOP, lot_id, season_id, price_paid, lot_secret) in R_cert
//   Esto mata el double-cobro residual: season_id ya NO es libre (la atestación lo fija),
//   y como nullifier_hash = Poseidon(lot_secret, season_id) está constreñido por la misma
//   season_id de leaf_0, variar season_id rompe la membership en R_cert -> no hay segundo
//   proof válido con el mismo lot_secret.
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
// Ola 3 (role-tag): cada leaf_i incluye su constante de rol en la preimagen Poseidon.
//   ROLE_FINCA=1, ROLE_COOP=2, ROLE_TOSTADOR=3. Slot 0=COOP, slot 1=FINCA, slot 2=TOSTADOR.
//   Arities: leaf_0=Poseidon(6), leaf_1/leaf_2=Poseidon(4).
template TerroirChain(levels) {

    // --- constantes de rol (Ola 3, no-cero para evitar ambigüedad) ---
    var ROLE_FINCA = 1;
    var ROLE_COOP = 2;
    var ROLE_TOSTADOR = 3;

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

    // ----- Check LOW (audit): los 3 certificadores son distintos entre sí -----
    // Garantiza 3 entidades distintas detrás de los eslabones (no basta reusar el mismo
    // pk para simular multi-atestación). Son 3 IsEqual negadas y se chequean los tres
    // pares explícitamente abajo: pk1≠pk2, pk0≠pk1 y pk0≠pk2 (incluye a la cooperativa
    // del eslabón 0 vs los certificadores de cadena).
    component neq12 = IsEqual();
    neq12.in[0] <== certifier_pk[1];
    neq12.in[1] <== certifier_pk[2];
    neq12.out === 0;
    component neq01 = IsEqual();
    neq01.in[0] <== certifier_pk[0];
    neq01.in[1] <== certifier_pk[1];
    neq01.out === 0;
    component neq02 = IsEqual();
    neq02.in[0] <== certifier_pk[0];
    neq02.in[1] <== certifier_pk[2];
    neq02.out === 0;

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

    // ----- Ola 3 (role-tag): 3 memberships en R_cert con rol comprometido -----
    // slot 0 = COOP: leaf_0 = Poseidon(certifier_pk_0, ROLE_COOP, lot_id, season_id, price_paid, lot_secret)
    // slot 1 = FINCA: leaf_1 = Poseidon(certifier_pk_1, ROLE_FINCA, lot_id, attest_data_0)
    // slot 2 = TOSTADOR: leaf_2 = Poseidon(certifier_pk_2, ROLE_TOSTADOR, lot_id, attest_data_1)
    component leafH0 = Poseidon(6);
    leafH0.inputs[0] <== certifier_pk[0];
    leafH0.inputs[1] <== ROLE_COOP;
    leafH0.inputs[2] <== lot_id;
    leafH0.inputs[3] <== season_id;
    leafH0.inputs[4] <== price_paid;
    leafH0.inputs[5] <== lot_secret;

    component inc0 = MerkleInclusion(levels);
    inc0.leaf <== leafH0.out;
    inc0.root <== r_cert;
    for (var j = 0; j < levels; j++) {
        inc0.pathElements[j] <== pathElements[0][j];
        inc0.pathIndices[j] <== pathIndices[0][j];
    }

    component leafH[2];
    component inc[2];

    // slot 1 = FINCA
    leafH[0] = Poseidon(4);
    leafH[0].inputs[0] <== certifier_pk[1];
    leafH[0].inputs[1] <== ROLE_FINCA;
    leafH[0].inputs[2] <== lot_id;
    leafH[0].inputs[3] <== attest_data[0];
    inc[0] = MerkleInclusion(levels);
    inc[0].leaf <== leafH[0].out;
    inc[0].root <== r_cert;
    for (var j = 0; j < levels; j++) {
        inc[0].pathElements[j] <== pathElements[1][j];
        inc[0].pathIndices[j] <== pathIndices[1][j];
    }

    // slot 2 = TOSTADOR
    leafH[1] = Poseidon(4);
    leafH[1].inputs[0] <== certifier_pk[2];
    leafH[1].inputs[1] <== ROLE_TOSTADOR;
    leafH[1].inputs[2] <== lot_id;
    leafH[1].inputs[3] <== attest_data[1];
    inc[1] = MerkleInclusion(levels);
    inc[1].leaf <== leafH[1].out;
    inc[1].root <== r_cert;
    for (var j = 0; j < levels; j++) {
        inc[1].pathElements[j] <== pathElements[2][j];
        inc[1].pathIndices[j] <== pathIndices[2][j];
    }
}

component main { public [r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash] } = TerroirChain(10);