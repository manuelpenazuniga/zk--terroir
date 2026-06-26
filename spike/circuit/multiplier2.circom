pragma circom 2.1.0;

// Minimal dummy circuit for the Day-1 on-chain spike.
// Proves knowledge of a, b (private) such that a*b == c (public output).
// Default prime is bn128 (= BN254 / alt_bn128) -> matches Soroban's
// native `crypto::bn254` pairing verifier.
template Multiplier2() {
    signal input a;
    signal input b;
    signal output c;
    c <== a * b;
}

component main = Multiplier2();
