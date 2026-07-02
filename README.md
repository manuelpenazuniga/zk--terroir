# ZK-Terroir

**Procedencia justa de café, demostrable sin revelar tu cadena de suministro.**

Una cooperativa puede probar —con **zero-knowledge**— que un lote pasó por una cadena de
certificadores acreditados y que pagó por encima de un **precio piso**, y cobrar automáticamente un
**premium en USDC** on-chain, **sin exponer** quiénes son sus proveedores, precios exactos, ni rutas.
El consumidor escanea un QR y verifica la procedencia contra la cadena, sin ver ningún dato privado.

> Estado: MVP de hackathon. Verificación Groth16 **BN254 nativa on-chain** en **Stellar Testnet**,
> con un circuito de 3 eslabones auditado y pago real de premium en SEP-41. Ver "Qué es real / qué es mock".

---

## Cómo funciona (flujo prueba → pago)

```
  Certificadores acreditados            Cooperativa (prover, en el navegador)         Contrato Soroban (Testnet)
  publican una raíz Merkle  ─────►  genera una prueba Groth16 (snarkjs WASM):   ─────►  claim_premium(proof, señales, payout):
  de atestaciones (R_cert)          · membership de 3 hojas ∈ R_cert                     1) señales[0] == R_cert almacenada
                                     · hash-chain del lote                                2) señales[1] == floor almacenado
                                     · range: price_paid ≥ floor                          3) nullifier no usado (anti doble-cobro)
                                     · nullifier = Poseidon(lot_secret, season)           4) verifica Groth16 (BN254 nativo P26)
                                                                                          5) payout binding (hi/lo == dirección)
                                                                                          6) transfiere premium en TUSDC (SEP-41)
                                                                                          7) registra lot_commit (para el QR)
```
El **premium = price_paid − floor**, con ambos extremos **fijados criptográficamente** (el
`price_paid` va comprometido dentro de una hoja acreditada; el `floor` está pineado en el contrato),
así nadie puede inflar el pago ni cobrar dos veces el mismo lote.

Verificación pública: `lot_status(lot_commit)` devuelve el timestamp del claim → el QR del consumidor
consulta ese endpoint de solo-lectura (ver `verify/`).

---

## Qué es real / qué es mock (honesto para el jurado)

| Pieza | Estado |
|---|---|
| Verificación Groth16 **BN254 nativa on-chain** (host functions P25/P26, `pairing_check` + MSM) | ✅ **REAL** en Testnet |
| Circuito de 3 eslabones (membership Merkle + hash-chain + range + nullifier), **auditado sound** | ✅ **REAL** (`circuits/terroir_chain.circom`) |
| Pago del premium en **SEP-41** (TUSDC de test) desde escrow del contrato | ✅ **REAL** (E2E: happy / replay-bloqueado / prueba-manipulada) |
| Anti doble-cobro (nullifier persistente) y anti-inflación (floor pineado) | ✅ **REAL**, auditado |
| Emisor de atestaciones (certificadoras) | 🟡 **MOCK honesto**: en producción, un oráculo reempaqueta PKI real (X.509/PGP) en credenciales; hoy el emisor es simulado |
| Token USDC | 🟡 **TUSDC de test** (SAC en Testnet), no USDC de mainnet |
| Alcance del demo | 🟡 **1 sola cooperativa / 1 lote** end-to-end; multi-coop y multi-región es trabajo futuro |
| Orden de custodia finca→coop→tostador | 🟡 **stretch**: hoy se prueba que 3 certificadores distintos atestan el mismo lote; el orden estricto es Día 3 stretch |

**El "guiño de tecnología nueva de Stellar" es BN254 + MSM nativos (P25 22-ene-2026, P26 6-may-2026)**,
que es genuino y load-bearing. Poseidon vive **solo dentro del circuito** (circomlib); el contrato
nunca recomputa Poseidon: trata raíces/nullifiers como field elements opacos que el SNARK ya validó.

---

## Estructura del repo

```
circuits/         circuito Circom (terroir_chain.circom) + setup snarkjs + infra JS (árbol R_cert, witness)
contracts/terroir Soroban (soroban-sdk 25.1.0): claim_premium, set_certifier_root, set_floor, lot_status
scripts/          setup_token.sh (TUSDC SAC), deploy.sh
deployments/      testnet.json (direcciones + tx del E2E)
verify/           verificador público de solo-lectura (QR / lot_status) — bash + stellar CLI, sin claves de escritura
spike/            spike Día 1: verificación BN254 genérica on-chain (base validada)
docs/             plan, decisiones, audit-log, internal/ (routing y orquestación multiagente)
```

---

## Cómo correrlo

```bash
# 1) Circuito: generar prueba y verificar off-chain
cd circuits && ./gen_proof.sh
snarkjs groth16 verify verification_key.json public.json proof.json     # -> OK

# 2) Contrato: build + tests (incluye la suite con prueba real)
cd ../contracts/terroir && cargo test && stellar contract build

# 3) Deploy + token en Testnet (direcciones quedan en deployments/testnet.json)
cd ../.. && ./scripts/setup_token.sh && ./scripts/deploy.sh

# 4) Verificación pública por QR / lot_status
#    ver verify/README.md
```
Direcciones y hashes de tx del último E2E: `deployments/testnet.json`.

---

## Documentación

- `docs/DECISIONS.md` — decisiones de arquitectura (D-001 curva BN254; D-002 confianza = membership Merkle, no EdDSA).
- `docs/PLAN-DIA-2.md` / `docs/PLAN-DIA-3.md` — plan por día (decisiones congeladas, olas de trabajo).
- `docs/AUDIT-LOG.md` — bitácora de auditoría (soundness del circuito, seguridad del contrato).
- `docs/internal/orchestration-zk-terroir.md` — cómo se coordina el desarrollo multiagente (opencode/agy/codex).

> Nota: `zk-terroir.md` y `techs-specs-zk-terroir.md` son el spec original; la sección de firmas
> **EdDSA-BabyJubjub** fue **reemplazada** por membership de Merkle (ver D-002). Se conservan con
> una nota de errata para trazabilidad.
</content>
