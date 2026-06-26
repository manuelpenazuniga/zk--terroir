# Día 1 — Reporte del spike letal (BN254 Groth16 on-chain)

**Fecha:** 2026-06-26 · **Estado: ✅ PASA. El riesgo que hundía el proyecto está muerto.**

Objetivo (del plan): forkear el verificador, generar una prueba dummy con snarkjs,
serializarla, verificarla **on-chain en testnet** y **medir costo + clavar la serialización**
antes de escribir lógica de dominio.

---

## 0. Hallazgo crítico — corrige una suposición del spec

El `groth16_verifier` de `soroban-examples` **NO usa BN254**: usa **BLS12-381**
(`crypto::bls12_381`, coords de 381 bits). El spec asumía "forkear ese contrato para BN254".

**Resolución (la mejor posible):** `soroban-sdk 25.1.0` expone **ambas** curvas:
`crypto::bls12_381` **y** `crypto::bn254` (añadida en v25 — coincide con P26). Confirmado en
`~/.cargo/.../soroban-sdk-25.1.0/src/crypto/bn254.rs` + `_migrating/v25_bn254.rs`.

→ La pila circom/circomlib/**BabyJubjub/Poseidon sobre BN254** es válida tal cual.
El trabajo NO es copiar el ejemplo verbatim, es **retargetearlo `bls12_381` → `bn254`**
(swap de módulo; la lógica del pairing es idéntica). Ya hecho en `spike/contract/`.

> BabyJubjub y las constantes Poseidon de circomlib son específicas del campo escalar de
> BN254 — NO existen sobre BLS12-381. Si Soroban solo tuviera BLS12-381, el plan EdDSA se
> rompía. Tiene BN254 → plan intacto.

---

## 1. Toolchain verificado

| Tool | Versión |
|---|---|
| rust / cargo | 1.96.0 |
| wasm32-unknown-unknown / wasm32v1-none | ✅ |
| node / npm | v24.17.0 / 11.13.0 |
| stellar CLI | 27.0.0 |
| circom | 2.2.3 (compilado de fuente) |
| snarkjs | 0.7.6 |
| soroban-sdk | 25.1.0 (BN254 nativo) |

---

## 2. Formato de serialización (snarkjs bn128 → Soroban `crypto::bn254`)

Layout Ethereum / EIP-197 uncompressed, **big-endian**:

| Objeto | Bytes | Layout |
|---|---|---|
| `Fr` (public input) | 32 | `be(value)`, mod r. **CLI lo toma como decimal** (`u256`), p.ej. `"33"` |
| `Bn254G1Affine` (A, C, IC[i], α) | 64 | `be(X) ‖ be(Y)`; ∞ = 64 ceros; sin subgroup check |
| `Bn254G2Affine` (B, β, γ, δ) | 128 | `be(X) ‖ be(Y)`, cada Fp2 = **`be(c1) ‖ be(c0)`** |

⚠️ **EL gotcha (silencioso):** snarkjs guarda cada Fp2 como `[c0, c1]` (real, imag), pero
Soroban/Ethereum quiere **`c1 ‖ c0`** (imaginario primero) → el serializador **invierte**
el par interno en TODO punto G2 (pi_b, vk_beta_2, vk_gamma_2, vk_delta_2). G1 NO se invierte.
Top-2 bits del byte alto de cada coord siempre 0 (modulus < 2^254) → flag bits OK.

Implementado en `circuit/serialize.js`. Verificado: la prueba da `true` on-chain.
El CLI acepta los structs como JSON con hex (sin `0x`) o vía `--<arg>-file-path`.

---

## 3. Costo medido (BN254, 1 public input)

Metering local (`env.cost_estimate().budget()`) = lo que fija el fee en red:

```
Cpu:  30,502,966 / 100,000,000   (30.5 %)
Mem:     204,383 /  41,943,040   ( 0.5 %)
```

Drivers:
- `Bn254Pairing`               17,528,691  (4 pairings — FIJO, no escala con inputs)
- `Bn254G2CheckPointInSubgroup` 11,751,020  (4 puntos G2 — FIJO)
- `Bn254G1Mul`                   1,150,435  **por public input** (el MSM)

**Proyección circuito real (5–7 public inputs):** +~6×1.15M ≈ **~37M CPU total**, cómodo
bajo 100M. BN254 es **~25% más barato** que el baseline BLS12-381 (~40.97M) del ejemplo.

**Fee real on-chain:** 0.00266 XLM (26,603 stroops).

---

## 4. Artefactos on-chain (testnet)

- Identidad: `terroir` = `GAKMZTTT53DPPFQUJWJ7EIRDY34YSUB63BY7T2OU7O4PYFE7OLDCJJ5J`
- Contrato verificador: `CCK6X7HZUC57YALZEW5NDH54TDM6PIR42Z76GR3E43IQ5YXGOHYNSK6W`
- Tx verify (correcto → true): `80acbe1b5af1496f41904be9bb693204bd97c078a4c8f05d8d0d5d72b0acd033`
- Soundness: input 22 → `false` ✓ (rechaza prueba con public input falso)

---

## 5. Cómo reproducir

```bash
# 1. generar prueba BN254
cd spike/circuit && ./gen_proof.sh
# 2. serializar al layout BN254 + emitir fixture Rust
node serialize.js ../contract/src/fixture.rs
# 3. test local (imprime budget)
cd ../contract && cargo test -- --nocapture
# 4. build + deploy + verify on-chain
stellar contract build
stellar contract deploy --wasm target/wasm32v1-none/release/terroir_bn254_verifier_spike.wasm --source terroir --network testnet
stellar contract invoke --id <ID> --source terroir --network testnet -- \
  verify_proof --vk-file-path ../circuit/vk_arg.json --proof-file-path ../circuit/proof_arg.json --pub_signals '["33"]'
```

## 6. Siguiente (Día 1 tarde → Día 2)

Con el spike verde, lo que queda NO tiene riesgo de rewrite cripto:
1. Circuito de 1 eslabón en circom: EdDSA-BabyJubjub + membership Poseidon + nullifier.
2. Extender a 3 eslabones + range proof `price ≥ floor`.
3. `claim_premium`: verify_proof + check `R_cert` + nullifier + transfer USDC (SEP-41).
