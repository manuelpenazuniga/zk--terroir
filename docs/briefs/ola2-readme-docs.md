# BRIEF Ola 1/2 — README público + limpieza de comentarios stale · WORKER (escribe)

> Antepón `docs/briefs/_wrapper.md`. **NO toques lógica** (ni contrato ni circuito): solo docs y
> comentarios. **No toca fondos.**

## Objetivo
Dejar la documentación pública coherente con el estado REAL del proyecto (jueces la leerán).

## Tareas
1. **Refina `README.md`** (Claude ya sembró una base en la raíz). Debe dejar claro y honesto:
   - **Qué es REAL:** verificación Groth16 **BN254 nativa on-chain** (host functions P25/P26),
     circuito de 3 eslabones sound (membership Merkle + hash-chain + range + nullifier), pago del
     premium en **SEP-41 (TUSDC de test)**, E2E en Testnet (happy/replay/tampered).
   - **Qué es MOCK/honesto:** el emisor de atestaciones (certificadoras reales no firman aún; en
     producción sería un oráculo de atestación PKI→credencial); TUSDC es token de test; 1 sola coop.
   - **El guiño de tecnología nueva de Stellar = BN254 + MSM nativos**, no Poseidon nativa
     (Poseidon vive SOLO en el circuito, circomlib). No sobrevendas Poseidon on-chain.
   - Flujo prueba→pago (1 diagrama simple) y **cómo correrlo** (generar prueba, deploy, claim, verificar QR).
   - Enlaza `docs/DECISIONS.md`, `docs/PLAN-DIA-3.md` y la página de verificación (`verify/`).
2. **Corrige 2 comentarios stale** (detectados en `docs/AUDIT-LOG.md` ronda 3, no funcionales):
   - `circuits/terroir_chain.circom` ~línea 135-136: el comentario dice que `pk_0` NO se chequea,
     pero el código SÍ lo chequea (3 `IsEqual`: pk0≠pk1, pk0≠pk2, pk1≠pk2). Corrige el comentario.
   - `contracts/terroir/src/lib.rs` (comentario cerca de la validación de floor): decía "H2 sigue
     abierto"; H2 se **cerró** en T1 v2/v3 (`price_paid` atado en `leaf_0`). Actualízalo.

## Restricciones
- No cambies código ejecutable, solo comentarios/markdown. `cargo build` y `snarkjs verify` deben
  seguir dando lo mismo (verifícalo).
- Mantén el README escaneable (no una novela). Español o inglés consistente con el repo (español).

## Criterios de aceptación
- README no promete como "real" nada que sea mock; el showcase está anclado en BN254/MSM nativos.
- Los 2 comentarios reflejan el código real.
- `cd contracts/terroir && cargo build` verde; `cd circuits && snarkjs groth16 verify verification_key.json public.json proof.json` = OK.

## Al terminar
**Haz commit**, resume cambios. Termina con: `VEREDICTO: LISTO — <una línea>`.
</content>
