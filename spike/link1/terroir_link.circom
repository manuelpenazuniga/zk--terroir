pragma circom 2.1.0;

include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/mux1.circom";

// One level of a Poseidon(2) Merkle inclusion proof.
// isRight = 0  => `cur` is the LEFT input  -> hash(cur, sibling)
// isRight = 1  => `cur` is the RIGHT input -> hash(sibling, cur)
template MerkleLevel() {
    signal input cur;
    signal input sibling;
    signal input isRight;
    signal output out;

    // constrain isRight to be boolean
    isRight * (isRight - 1) === 0;

    component mux = MultiMux1(2);
    mux.c[0][0] <== cur;      mux.c[0][1] <== sibling;   // left  slot
    mux.c[1][0] <== sibling;  mux.c[1][1] <== cur;       // right slot
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

// MVP 1-eslabón: prueba membership de un commitment en la raíz del set acreditado
// (R_cert) y la correcta derivación del nullifierHash — sin revelar nada del eslabón.
template TerroirLink(levels) {
    // privados (el moat)
    signal input nullifier;
    signal input secret;
    signal input pathElements[levels];
    signal input pathIndices[levels];
    // públicos
    signal input root;          // R_cert: raíz del set de certificadores/atestaciones acreditadas
    signal input nullifierHash; // anti doble-cobro

    // commitment = Poseidon(nullifier, secret) = la hoja del árbol
    component commit = Poseidon(2);
    commit.inputs[0] <== nullifier;
    commit.inputs[1] <== secret;

    // nullifierHash = Poseidon(nullifier)
    component nh = Poseidon(1);
    nh.inputs[0] <== nullifier;
    nh.out === nullifierHash;

    // membership del commitment en R_cert
    component inc = MerkleInclusion(levels);
    inc.leaf <== commit.out;
    inc.root <== root;
    for (var i = 0; i < levels; i++) {
        inc.pathElements[i] <== pathElements[i];
        inc.pathIndices[i] <== pathIndices[i];
    }
}

component main { public [root, nullifierHash] } = TerroirLink(10);
