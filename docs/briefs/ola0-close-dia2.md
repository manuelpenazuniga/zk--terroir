# BRIEF Ola 0 — Auditoría de cierre del T3-final (Día 2) · READ-ONLY

> Antepón `docs/briefs/_wrapper.md`. **Este es un brief de AUDITORÍA: NO modifiques ningún archivo,
> solo LEE y REPORTA.** Fraseo pre-merge (no "pentest"): *revisa MI código antes de commitear, por
> bugs de correctness, conservación de fondos y soundness; sé adversarial sobre quién podría ganar
> dinero rompiendo una regla.*

## Qué se está cerrando
El **T3-final** está implementado en el working tree pero **sin commitear**. Cambios vs el último
commit (`be996a2`): VK real horneada en el contrato, eliminación del bypass `#[cfg(test)]` en
`verify()`, suite de tests con **prueba real** (`*_real`), y un E2E ya corrido en Testnet
(`deployments/testnet.json`). Tu trabajo: confirmar que es **correcto y no saqueable** antes del commit.

## Archivos a leer (foco)
- `contracts/terroir/src/lib.rs` — `claim_premium`, `verify`/`groth16_verify`/`vk`, constantes `VK_*`,
  `check_payout_binding`, `fr_to_nonneg_i128`, orden checks→effects→interaction.
- `contracts/terroir/src/test.rs` — la suite `*_real` (usa `circuits/proof.json`/`public.json`).
- `circuits/verification_key.json` — la VK del circuito T1 v3 (comparar con la horneada).
- `circuits/terroir_chain.circom` — solo para confirmar que las 7 señales públicas coinciden.

## Checklist de auditoría (reporta OK/PROBLEMA con archivo:línea por cada punto)
1. **Bypass fuera de verdad:** `verify()` (lib.rs ~274) llama al `groth16_verify` real **sin**
   `#[cfg(test)]` que devuelva `true`. Bajo `cargo test` se ejercita cripto real (no un stub).
2. **VK horneada == circuito:** las constantes `VK_ALPHA/BETA/GAMMA/DELTA/IC0..IC7` (lib.rs ~379-390)
   corresponden a `circuits/verification_key.json` **serializado con `circuits/serialize.js`**
   (G1 = be32(x)‖be32(y); G2 con swap `c1‖c0`, layout EIP-197). `ic.len() == 8` (7 públicos + 1).
   Si no puedes recomputar los bytes, marca `// TODO(audit)` y dilo — no lo des por bueno a ciegas.
3. **Orden de señales (Decisión A):** el parseo en `claim_premium` (lib.rs ~141-147) mapea índices
   0..6 exactamente a `[r_cert, floor, lot_commit, premium, payout_hi, payout_lo, nullifier]`.
4. **Root binding:** `pub_signals[0] == ROOT` almacenado; `set_certifier_root` es admin-only.
5. **Floor binding (anti-inflación):** `pub_signals[1] == FLOOR` almacenado (lib.rs ~162-168);
   `set_floor` admin-only y rechaza negativos; sin floor seteado → panic (no paga).
6. **Premium:** `premium_amount` (idx 3) > 0, cabe en i128 sin overflow; el transfer usa ese monto.
7. **Nullifier anti-replay:** `pub_signals[6]` en storage **persistente** (no `temporary`) + bump TTL;
   se **inserta después** de verificar y **antes** del transfer; doble uso → panic.
8. **Payout binding:** `check_payout_binding` reconstruye los 32 bytes del `payout` desde hi/lo
   (16+16, mitades altas en cero). Un atacante NO puede redirigir el premium a otra dirección.
9. **CEI (Decisión I):** orden root→floor→amount→nullifier(check)→verify→payout→(effects: insert
   nullifier + lot)→**transfer al final**. Un panic en el transfer revierte todo (atómico).
10. **Tests negativos reales presentes y verdes:** `test_double_spend_real` (nullifier), `bad_root`,
    `bad_floor`, `amount_zero`, `bad_proof` (prueba manipulada), `payout_binding`. Que fallen por la
    razón correcta (`#[should_panic(expected=…)]`).
11. **Ataque adversarial (piénsalo con saña):** ¿hay ALGÚN camino en que un prover con 3 hojas
    válidas se auto-pague más de `price_paid - floor`, o cobre el mismo lote dos veces variando algún
    input privado libre (p.ej. `season_id`, ya cerrado en T1 v3 — confírmalo), o dispare el transfer
    sin pasar el pairing? Reporta cualquier grado de libertad no constreñido.

## Salida obligatoria
Por cada punto: `OK` o `PROBLEMA (archivo:línea) — descripción`. Termina EXACTAMENTE con:
`VEREDICTO: PASA` **o** `VEREDICTO: NO-PASA — <razón en una línea>`.
</content>
