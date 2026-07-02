# Plan Día 3 — Cerrar Día 2 + verificación pública (QR) + README para jueces

> **Cómo se ejecuta este plan:** lo sigue el **agente orquestador** según
> `docs/internal/orchestration-zk-terroir.md` (runbook mecánico). Claude ya congeló el diseño y
> pre-escribió los briefs en `docs/briefs/`. El orquestador NO diseña: ejecuta olas, verifica,
> audita, y aplica la regla de veredicto (§6 del runbook). Ante fondos en duda → **STOP → usuario**.

---

## 0. Estado real al empezar (verificado 2026-07-01)

| Pieza | Estado | Dónde |
|---|---|---|
| Día 1 — spike BN254 on-chain | ✅ verificado (tx `80acbe1b…`) | `spike/`, `docs/DECISIONS.md` D-001 |
| T1 v3 — circuito 3 eslabones **SOUND** | ✅ auditado PASA (commit `be996a2`) | `circuits/terroir_chain.circom`, `docs/AUDIT-LOG.md` ronda 3 |
| T2 — infra JS (árbol R_cert, witness) | ✅ PASA | `circuits/js/` |
| T3 esqueleto + floor pin | ✅ PASA | `contracts/terroir/src/lib.rs` |
| T5 — token TUSDC (SAC testnet) | ✅ PASA | `scripts/`, `deployments/testnet.json` |
| **T3-final** (VK horneada, bypass `cfg(test)` fuera, tests real-proof, deploy, E2E) | ⚠️ **HECHO en working tree, SIN COMMITEAR** | `git status` (lib.rs, test.rs, `*_real.json`, testnet.json modificados) |
| **Auditoría de cierre Día 2 (GPT-5.5) + commit** | ❌ **PENDIENTE** → **es la Ola 0** | — |

**Evidencia del E2E ya corrido** (`deployments/testnet.json`, working tree):
`terroir_contract = CBHFN7QUJJMA2RXMPVNYCFSCVZDQSOSVIRNVHJKPYHTE4DNHWX5ATJQQ`;
`e2e.happy_path=pass`, `replay_blocked=pass`, `tampered_proof_blocked=pass`,
`lot_status_registered=1782661915`; tx de happy/init/set_root/set_floor registradas.

> **Implicación:** el hito Definition-of-Done del Día 2 (§1 de `PLAN-DIA-2.md`) **parece cumplido**,
> pero **no está cerrado**: falta el pase adversarial de cierre y el commit. Ese es el gate de la Ola 0.

### 0.1 Progreso de ejecución (actualizado 2026-07-01)

| Ola | Estado | Resultado |
|---|---|---|
| **Ola 0** — cerrar Día 2 (GATE) | ✅ **CERRADA** | Triple audit PASA (checklist-Claude + agy/Gemini 3.1 Pro High + codex/GPT-5.5*); VK recomputada byte-a-byte por los 3; `cargo test` 11/11. Commit `505dc49`, tag **`dia2-cerrado`**. |
| **Ola 1** — T3D-verify ∥ T7-docs | ✅ **HECHA** | `verify/` (verificador read-only `lot_status`+QR, smoke-test Testnet OK) + README refinado + comentario stale del circom corregido. |
| **Ola 2** — audit dual + merge | ✅ **CERRADA** | Dual PASA (checklist-Claude + agy/Gemini 9/9). Mergeadas a `main` (`879a20c`). |
| **Ola 3** — stretch (endurecer custodia) | ⏳ **diseño/brief** | Toca circuito sound → gate humano. Brief en `docs/briefs/ola3-harden-custody.md`. |

\* codex emitió `NO-PASA` **solo** por no poder correr `cargo test` en su sandbox read-only (falso
negativo de entorno, no del código); 10/11 OK + VK recomputada. Efectivo-PASA aprobado por usuario.

**DoD Día 3 (§1) puntos 1–3: ✅ cumplidos.** Punto 4 (stretch) = Ola 3, opcional y con aprobación.

---

## 1. Meta del Día 3 (Definition of Done)

1. **Día 2 cerrado**: T3-final auditado (triple, con GPT-5.5) y **commiteado + tag** en `main`.
2. **Verificación pública (el QR)**: dado un `lot_commit`, cualquiera puede comprobar on-chain
   (sin secretos) que ese lote fue certificado y el premium pagado, vía `lot_status(lot_commit)`.
   Entregable mínimo: un verificador (CLI o página estática) que consulta el contrato en Testnet.
3. **README público** (el que leen los jueces): qué es real / qué es mock, flujo prueba→pago,
   showcase BN254/MSM nativos (P25/P26), cómo correrlo.
4. (**Stretch**) endurecer custodia: `region_root` / orden finca→coop→tostador / doble-membership.

---

## 2. DAG de olas

```
Ola 0 (secuencial, GATE):   cerrar Día 2  ── audit cierre GPT-5.5 + Gemini ─► commit + tag
                                 │  (bloquea todo lo demás: main debe quedar limpio y auditado)
                                 ▼
Ola 1 (paralelo):    T3D-verify (página/CLI QR)  ∥  T7-docs (README + fix 2 comentarios stale)
                                 │
                                 ▼
Ola 2 (secuencial):  auditoría dual de la Ola 1 (no toca fondos → checklist-Claude + Gemini) ─► merge
                                 │
                                 ▼
Ola 3 (STRETCH, solo si sobra tiempo y con diseño aprobado): endurecer circuito/custodia
```
**Barrera dura:** la Ola 1 **no arranca** hasta que la Ola 0 mergeó (`main` limpio). Trabajar sobre
un T3-final sin commitear = riesgo de perder trabajo y de auditar un blanco móvil.

---

## 3. Tareas (con modelo, brief y compuerta)

### Ola 0 — Cierre Día 2 (GATE) · **audita, no implementa**
**Objetivo:** confirmar que el T3-final sin commitear es correcto y seguro, y **commitearlo**.
**No hay implementación nueva**; es verificación + auditoría + commit.

1. **Verifica local** (orquestador, Paso 5 del runbook):
   `cd contracts/terroir && cargo test` (deben pasar los `*_real`: happy, double_spend, bad_root,
   bad_floor, amount_zero, bad_proof, payout_binding) `&& stellar contract build && cargo clippy --all-targets`.
2. **Auditoría de cierre — TRIPLE (toca fondos):**
   - Gemini 3.1 Pro High (agy) con `docs/briefs/ola0-close-dia2.md`, `--add-dir …/contracts/terroir/src`.
   - **GPT-5.5 (codex)** con el mismo brief — **pide OK al usuario antes** (cuota).
   - Checklist-Claude: el propio brief lleva la sección "Checklist" (aplícala ítem por ítem).
3. **Veredicto** (§6 runbook). Foco adversarial: ¿el bypass `cfg(test)` está de verdad fuera?
   ¿la VK horneada corresponde al circuito T1 v3? ¿algún hallazgo nuevo de drenaje/doble-cobro?
4. **PASA → commit + tag:**
   ```bash
   git add -A && git commit    # mensaje: "Día 2 CERRADO: T3-final (VK horneada + E2E testnet) — auditado ✅"
   git tag dia2-cerrado
   ```
   **NO-PASA / ALTA en fondos → STOP → usuario.**
- **Brief:** `docs/briefs/ola0-close-dia2.md`

### Ola 1 · T3D-verify — Verificación pública (QR / lot_status) · Gemini 3.5 Flash (High) (agy) → escala MiniMax M3
**Deliverable:** `verify/` con un verificador de solo-lectura (página estática o script Node) que:
dado un `lot_commit` (hex), llama `lot_status(lot_commit)` del contrato en Testnet
(`terroir_contract` de `deployments/testnet.json`) y muestra "Certificado ✓ + premium pagado
(timestamp)" o "No encontrado". Un QR codifica la URL/params del `lot_commit`.
**No toca fondos** (solo lectura on-chain). **Aceptación:** con el `lot_commit` del E2E
(`lot_status_registered` existe) devuelve el timestamp; con uno inventado, "No encontrado".
- **Brief:** `docs/briefs/ola1-public-verify.md`

### Ola 1 · T7-docs — README público + limpieza de comentarios · Gemini 3.5 Flash (Medium) (agy)
**Deliverable:** refinar `README.md` (Claude ya sembró una base) y **corregir 2 comentarios stale**
detectados en auditoría: `circuits/terroir_chain.circom:135-136` (dice que pk0 no se chequea, pero SÍ)
y `contracts/terroir/src/lib.rs` (comentario que decía "H2 sigue abierto" — ya cerrado en T1 v2/v3).
**No toca lógica.** **Aceptación:** README coherente con el estado real (no promete lo mock como real);
comentarios corregidos; `cargo build` sigue verde.
- **Brief:** `docs/briefs/ola2-readme-docs.md`

### Ola 3 · STRETCH — Endurecer custodia (region_root / orden / doble-membership)
⚠️ **TOCA EL CIRCUITO SOUND.** No hay brief pre-escrito y el orquestador **NO** lo lanza solo.
Requiere que **Claude/usuario diseñe y apruebe** el brief (re-hornear VK, re-auditar triple, re-deploy).
Candidatos (de `PLAN-DIA-2 §2` stretch): `region_root`; orden finca→coop→tostador (recupera H1);
`pk_1 != pk_2` fuerte / doble-membership. **Gate:** diseño aprobado → brief → triple audit → re-deploy.

---

## 4. Compuertas que aplica el orquestador (resumen)

- **Ola 0:** triple audit (Gemini + GPT-5.5 + checklist) sobre el T3-final; PASA → commit + tag.
  Cualquier ALTA de fondos → STOP.
- **Ola 1 (no fondos):** dual (checklist-Claude + Gemini) sobre cada entregable antes de merge.
- **Ola 3 (fondos):** no se ejecuta sin diseño+brief aprobado por humano.
- **Global:** verifica siempre tú (`cargo test`, `snarkjs verify`, `stellar`), nunca el claim del agente.

---

## 5. Handoffs al usuario

1. Fin de Ola 0: reporta el veredicto triple + los tx del E2E; pide OK para el tag `dia2-cerrado`.
2. Fin de Ola 1/2: muestra la página/CLI de verificación funcionando contra Testnet y el README.
3. Ola 3 solo bajo pedido explícito con diseño aprobado.
</content>
