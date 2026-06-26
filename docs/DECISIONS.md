# Decisiones de arquitectura — ZK-Terroir

## D-001 (2026-06-26) · Curva = BN254, NO BLS12-381

**Contexto:** el agente arquitecto propuso cambiar a BLS12-381 alegando que "no hay
verificador Groth16 de Circom para BN254 shipped" y que por eso EdDSA-BabyJubjub no servía.

**Hecho que decide:** el spike del Día 1 **refutó esa premisa empíricamente**. `soroban-sdk
25.1.0` trae `crypto::bn254` nativo (P26). Generé una prueba Circom/snarkjs **bn128**, la
serialicé y la **verifiqué on-chain en testnet** (tx `80acbe1b…acd033`, contrato
`CCK6X7HZ…SK6W`): correcto→`true`, falso→`false`. El retarget del ejemplo BLS12-381→BN254 tomó ~1h.

**Decisión (confirmada por el usuario):** **quedarse en BN254 + circomlib.** Razones:
- Spike BN254 ya verificado on-chain (cero riesgo de rewrite cripto).
- circomlib (Poseidon, Merkle, comparadores) es **nativo de BN254** → todo funciona out-of-the-box.
- Cambiar a BLS12-381 obligaría a abandonar circomlib (constantes Poseidon son BN254-específicas),
  usar poseidon2/poseidon255, y apostar al reuse de `NethermindEth/stellar-private-payments` —
  cuyo layout real difiere de lo descrito (usa Poseidon2, no `poseidon255`; sin `cli/circom2soroban`
  ni `libs/lean-imt` visibles) → **no es drop-in**.

## D-002 (2026-06-26) · Modelo de confianza = membership de Merkle (NO EdDSA-BabyJubjub)

**Adoptado del agente arquitecto (mejora válida e independiente de la curva).** En vez de que
cada certificador firme con EdDSA-BabyJubjub in-circuit (más caro y frágil), usamos el patrón
commitment + membership de Merkle (estilo privacy-pools), **implementado con circomlib sobre BN254**:

- Cada certificador acreditado publica una **raíz de Merkle (Poseidon, circomlib)** de sus
  atestaciones por eslabón (publicar la raíz desde su cuenta = su "firma").
- El circuito prueba por eslabón: **membership** de la atestación en la raíz del certificador +
  certificador ∈ set acreditado (`R_cert`, patrón ASP) + hash-chain de custodia +
  range proof `price ≥ floor` (circomlib `GreaterEqThan`) + `nullifierHash = Poseidon(nullifier)`.
- Esto **invalida** la sección EdDSA del spec original (`zk-terroir.md` §3.2 / techs-specs §3.2).

**Estructura de commitment (tornado/privacy-pools):** `commitment = Poseidon(nullifier, secret)`
(hoja del árbol), `nullifierHash = Poseidon(nullifier)`. Públicos: `root`, `nullifierHash`.
