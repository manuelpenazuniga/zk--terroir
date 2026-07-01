# Tech-Specs — ZK-Terroir (validación de ingeniería senior ZK/Stellar)

**Rol:** Senior Blockchain Engineer (ZK + Stellar) · **Fecha:** 2026-06-26 · **Para:** de-riskear el build de 3 días
**Mandato:** ninguna tecnología en el camino crítico puede estar sin lanzar, incompleta o ser impracticable. Cada pieza lleva estado real (mainnet/probado) + fallback.

> **⚠️ ERRATA (2026-07-01):** las secciones §0.2, §3.2, §3.3, §4 describen firmas de certificadora con
> **EdDSA-BabyJubjub-Poseidon**. Ese enfoque fue **reemplazado** por **membership de Merkle** (D-002,
> `docs/DECISIONS.md`): el circuito no verifica firmas EdDSA in-circuit; prueba *membership* de cada
> atestación en la raíz Merkle del certificador (más barato y ya implementado en
> `circuits/terroir_chain.circom`). Lo demás del doc (Poseidon solo in-circuit, BN254 nativo como
> showcase, serialización G2 `c1‖c0`) **sigue vigente** y fue validado en el spike/Día 2. Estado real:
> `README.md`, `docs/AUDIT-LOG.md`, `docs/PLAN-DIA-3.md`.

---

## 0. Veredicto de viabilidad (TL;DR)

🟢 **CONSTRUIBLE sin riesgo de rewrite**, con **dos correcciones de diseño** que aplico abajo. Todo el camino crítico corre sobre tecnología **viva en mainnet hoy** (jun-2026) y sobre código de referencia que ya funciona. El único riesgo real (Poseidon interop) se neutraliza con una regla de arquitectura, no con tecnología nueva.

**Las dos correcciones (detalle en §2 y §3):**
1. **La Poseidon va SOLO dentro del circuito (circomlib), nunca recomputada on-chain.** El contrato trata raíces/nullifiers/commitments como **field elements opacos** (public inputs del SNARK). El "showcase" on-chain es **BN254 nativo (pairing)**, no Poseidon nativo.
2. **Las firmas de certificadoras son EdDSA-BabyJubjub-Poseidon (circomlib), no ECDSA/secp256k1.** Verificar ECDSA-secp256k1 dentro de un circuito es carísimo; EdDSA-BabyJubjub es nativo de circom y barato.

---

## 1. Estado real de cada tecnología (la tabla que responde tu miedo)

| Tecnología | Rol en Terroir | Estado real (evidencia) | Riesgo de rewrite | Fallback seguro |
|---|---|---|---|---|
| **BN254 host functions (pairing)** | verificar el Groth16 on-chain | 🟢 **Mainnet desde P25 (22-ene-2026)**; ampliado en P26 (6-may-2026). CAP-0074. Espeja EIP-196/197. | **Nulo** | — (es la base) |
| **`groth16_verifier` (soroban-examples)** | contrato verificador base | 🟢 **Existe y funciona** (Circom2 + snarkjs, demo a·b=c). | **Nulo** | fork directo |
| **Circom 2 + snarkjs (Groth16, BN254/alt_bn128)** | circuito + pruebas en navegador | 🟢 Maduro, años en producción. Curva = la de Stellar. | **Nulo** | — |
| **circomlib Poseidon (in-circuit)** | Merkle, nullifier, commitments | 🟢 Battle-tested (iden3). | **Nulo** *si se mantiene in-circuit* | ver regla §2 |
| **circomlib EdDSA-BabyJubjub** | firma de certificadoras en circuito | 🟢 Estándar (`eddsaposeidon.circom`). | **Nulo** | — |
| **SEP-41 token (USDC/EURC)** | pago del premium | 🟢 Interfaz estándar viva. | **Nulo** | testnet USDC |
| **Poseidon host function (nativa)** | *opcional* showcase on-chain | 🟡 Viva (CAP-0075) **pero params ≠ circomlib** | **Alto si la pones en camino crítico** | **NO usarla on-chain**; ver §2 |
| **Stellar Private Payments PoC** | patrón de referencia (Circom+Poseidon Merkle+Groth16) | 🟢 Existe (Nethermind). Prueba que el patrón corre en Stellar. | **Nulo** | estudiar/forkear |

**Conclusión:** todo el camino crítico es verde. Lo único amarillo (Poseidon nativa on-chain) **se saca del camino crítico** por diseño — no se "descarta a mitad de desarrollo", se decide hoy no depender de ella.

---

## 2. El riesgo #1 y su neutralización: Poseidon interop

**El problema (real, confirmado):** la host function nativa de Stellar (CAP-0075) expone la **permutación** Poseidon/Poseidon2 con parámetros configurables (estado `t`, S-box `d`, rondas, constantes, MDS). **circomlib usa SUS propias constantes/MDS.** Si computas un commitment con circomlib en el circuito y luego el contrato lo recomputa con la host function nativa esperando igualdad, **probablemente NO coincidan**. Ese mismatch es invisible hasta integración → el clásico rewrite tardío.

**La regla de arquitectura (cero-riesgo):**
> **Todo Poseidon se computa DENTRO del circuito (circomlib). El contrato NUNCA recomputa un Poseidon.** Raíces, nullifiers y commitments llegan al contrato como **public inputs opacos** (field elements) que el SNARK ya validó. El contrato solo: (a) verifica el Groth16 con **BN254 nativo**, (b) compara `R_cert` con la raíz almacenada (igualdad de field elements), (c) chequea el nullifier en un `Map` (igualdad), (d) paga.

Con esta regla, la compatibilidad de parámetros Poseidon **es irrelevante para la corrección** porque solo existe en un lugar (el circuito). El verificador Groth16 no necesita Poseidon: necesita **pairing BN254**, que es nativo y vive en mainnet.

**¿Y el "guiño Poseidon nativo" de la convocatoria?** Se reencuadra honestamente: el showcase de "tech nueva de Stellar" en Terroir es **BN254 nativo (pairing para verificar + MSM que el propio verificador usa para los public inputs)** — que es genuino y load-bearing. Poseidon nativa on-chain queda como **stretch opcional** (un endpoint de conveniencia que hashea datos de auditoría, NO en el camino de la prueba), y solo si alguien alinea params; **no se promete ni se depende de ella**.

---

## 3. Arquitectura técnica validada (la receta que no falla)

### 3.1 Stack congelado (pinear versiones evita sorpresas)
- **Circuito:** Circom 2.x + circomlib (Poseidon, `eddsaposeidon`, `smt`/Merkle).
- **Pruebas:** snarkjs (Groth16, curva bn128). Proving en navegador (WASM), como el PoC de Private Payments.
- **Contrato:** fork de `stellar/soroban-examples/groth16_verifier` + lógica de escrow/pago (SEP-41).
- **Red:** testnet (host functions P25/P26 disponibles; usar SDK última versión).

### 3.2 Firmas de certificadora — corrección concreta
El doc original dice `Sig_certifier(...)` genérico. **Especificación:** cada certificadora (en el MVP, un emisor mock) tiene una clave **BabyJubjub**; firma con **EdDSA-Poseidon**. El circuito verifica con `EdDSAPoseidonVerifier` (circomlib). Esto es barato y nativo de circom.
**Aplicabilidad real (gap honesto, NO bloqueante):** las certificadoras reales (Fairtrade, USDA) hoy no firman en BabyJubjub. En producción eso se cierra con un **oráculo de atestación** (un puente que reempaqueta su PKI X.509/PGP en una credencial BabyJubjub firmada). Para el hackathon es **mock honesto** y se dice en el README. No requiere tecnología inexistente; requiere onboarding de datos.

### 3.3 El circuito, recortado a lo que cabe en 3 días
- **Profundidad fija:** cadena de **3 eslabones**, árbol de certificadores de **profundidad ≤ 10** (1024 certificadores; sobra).
- **Predicados:** 3× (verificar firma EdDSA + membership del PK del certificador) + hash-chain (3 niveles) + 1 range proof (`price ≥ floor`, via `LessThan`/`GreaterEqThan` de circomlib) + nullifier (`Poseidon(lot_secret, season_id)`).
- **Public inputs:** `R_cert, floor_price, region_root, lot_commit, premium_amount, payout_addr, nullifier`. Todo lo demás privado.
- **Costo on-chain:** una verificación Groth16 (constante). Medir el Día 1 con prueba dummy (es el spike de riesgo).

### 3.4 Gotcha práctico #1 (no bloqueante, pero cuesta horas): serialización
El paso fiddly probado en los tutoriales: **convertir vk/proof/public-inputs de snarkjs al layout de bytes (hex canónico, endianness, codificación de puntos G1/G2) que espera el contrato.** No es un blocker; es trabajo de plomería. Reservar medio día. El `groth16_verifier` ya trae el formato esperado → copiar su convención exacta.

---

## 4. Correcciones al `zk-terroir.md` original

| Sección | Dice | Corregir a |
|---|---|---|
| §3.3 "Poseidon nativo (P25): *todos* los commitments... usan la host function `poseidon`, no un hash en WASM" | implica que el contrato recomputa Poseidon nativo | **El Poseidon vive en el circuito (circomlib); el contrato no lo recomputa.** Showcase on-chain = **BN254 nativo (pairing+MSM)**. Poseidon nativa = stretch opcional fuera del camino crítico. |
| §3.1 / §3.2 `Sig_certifier` genérico | ambiguo | Especificar **EdDSA-BabyJubjub-Poseidon (circomlib)**. ECDSA-secp256k1 in-circuit = evitar. |
| §3.1 "hash chain" | ok conceptual | Aclarar que la hash-chain también es **circomlib Poseidon in-circuit**. |
| §7 tabla, fila "garbage-in" | ok | Añadir el **oráculo de atestación** como el puente real PKI→BabyJubjub (no es tech faltante, es onboarding). |
| §0/§3 "guiño al protocolo" vía Poseidon | sobre-vende | Reanclar el guiño en **BN254/MSM nativos** (genuinos y vivos). |

> Nota: son correcciones de **encuadre técnico**, no de producto. La tesis, el dolor y la narrativa del original siguen intactos y fuertes.

---

## 5. Mejoras para subir el fit con el hackathon

1. **Forkea, no escribas cripto.** Base = `groth16_verifier` + patrones del **Private Payments PoC** (mismo trío Circom+Poseidon-Merkle+Groth16). Reduce riesgo y le dice al jurado "usé el starter code oficial bien".
2. **Haz visible el pago cross-border.** El momento que el jurado recuerda: prueba válida → **USDC a una wallet de cooperativa** en ~5s. Es el caso insignia de Stellar; ponlo en primer plano del video.
3. **Explicita BN254/MSM nativos en el README** ("la verificación corre sobre las host functions de P25/P26") — es el guiño correcto y verificable.
4. **QR read-only barato.** El consumidor escanea → lee `lot_status`. No construyas un explorer; un endpoint simple.
5. **Un solo vertical, un solo lote.** Café, 3 eslabones, 1 premium. La generalización (minerales, farma) se **cuenta** en §9, no se construye.

---

## 6. Simplificaciones (anti over-engineering, tu requisito explícito)

- ❌ **Fuera:** grafo/DAG general de N eslabones → ✅ cadena fija de 3.
- ❌ **Fuera:** acumulación de Merkle root on-chain con Poseidon nativo → ✅ raíces como public inputs opacos.
- ❌ **Fuera:** integración con PKI real de certificadoras → ✅ emisor mock BabyJubjub, documentado.
- ❌ **Fuera:** multi-certificación por eslabón → ✅ 1 cert por eslabón en el MVP.

---

## 7. Plan de implementación de cero-riesgo (orden que garantiza no chocar pared)

1. **Día 1 (mañana) — Spike letal primero.** Fork `groth16_verifier`, despliega en testnet, verifica una **prueba dummy** snarkjs on-chain y **mide costo + clava la serialización**. Si esto funciona (y funciona: es código probado), el 90% del riesgo murió.
2. **Día 1 (tarde) — Circuito de 1 eslabón.** EdDSA + membership + nullifier. Prueba real verifica on-chain.
3. **Día 2 — Cadena de 3 + range proof + pago.** `claim_premium` paga USDC testnet vía SEP-41.
4. **Día 3 — UX (proving en navegador) + QR + README honesto + video.**

**Regla de oro:** cada día termina con algo que **verifica on-chain**. Nada se deja "para integrar al final".

---

## 8. Dependencias congeladas

```
circom        2.1.x
circomlib     (Poseidon, eddsaposeidon, comparators, smt)
snarkjs       0.7.x  (Groth16, curve bn128)
soroban-sdk   última (P26-compatible)
base contrato stellar/soroban-examples → groth16_verifier
referencia    NethermindEth/stellar-private-payments
red           Stellar testnet (P25+P26 activos)
```

**Garantía:** ninguna de estas piezas está "por lanzarse". Todas están en mainnet o son librerías con años de uso. Si el spike del Día 1 pasa, no hay tecnología que pueda obligarte a un rewrite.
