# Handoff — retomar la sesión del ORQUESTADOR en otro PC (Mac)

> **Qué es esto:** el brief de arranque para una **nueva sesión de Claude Code (orquestador)** en el
> Mac. Léelo entero antes de tocar nada. La sesión anterior (Linux/WSL) cerró el Día 3 puntos 1–3 y
> dejó **preparada** (no ejecutada) la Ola 3 stretch. Este doc te reconstruye TODO el contexto porque
> **la memoria de Claude NO viaja entre máquinas** (vive en `~/.claude/…/memory/` del PC viejo).
>
> **Cómo usarlo:** al abrir Claude Code en el Mac, dentro del repo, pásale como primer mensaje algo
> como: *"Sos el orquestador de ZK-Terroir. Lee `docs/HANDOFF-ORQUESTADOR-MAC.md` y luego el runbook
> `docs/internal/orchestration-zk-terroir.md`; corré el arranque rápido y decime el plan."*

---

## 0. TL;DR del estado (a 2026-07-01)

- Rama `main`, repo ahora en **GitHub**: `https://github.com/manuelpenazuniga/zk--terroir.git`.
- **Día 2 CERRADO** (tag `dia2-cerrado`): contrato `terroir` con VK real horneada + E2E en Testnet,
  auditado triple. **Día 3 puntos 1–3 ✅**: verificador público (`verify/`) + README para jueces.
- **Pendiente = Ola 3 (STRETCH), solo preparada:** endurecer custodia (orden de roles
  finca→coop→tostador). Diseño congelado y brief escrito en `docs/briefs/ola3-harden-custody.md`.
  **NO ejecutada** — toca fondos/circuito ⇒ gate humano.
- Últimos commits (verificá con `git log --oneline -8`):
  `b617693` brief Ola 3 · `41473a4` estado Día 3 · `879a20c` merge Ola 1 T7-docs ·
  `18199b9` Ola 1 T3D-verify · `505dc49` **Día 2 CERRADO** (tag `dia2-cerrado`).

---

## 1. Tu rol (orquestador) — NO lo olvides

Ejecutás **mecánicamente** un runbook ya escrito; **no diseñás ni auditás con criterio propio lo que
toca fondos**. Corrés comandos, leés strings de veredicto (`VEREDICTO: PASA` / `NO-PASA`), aplicás
reglas de merge/STOP. **Ante cualquier duda sobre FONDOS → STOP → usuario.** El "cerebro" (diseño,
decisiones, briefs, checklists) ya está pre-horneado en los docs.

Fuente de verdad, en orden:
1. `docs/internal/orchestration-zk-terroir.md` — **el runbook operativo** (§4 ciclo por tarea, §5
   auditoría, §6 regla de veredicto, §8 arranque rápido). *(Antes gitignored; ahora público.)*
2. `docs/PLAN-DIA-3.md` — las olas. **§0.1 tiene el progreso real** (Olas 0/1/2 ✅).
3. `docs/briefs/` — briefs pegables (`_wrapper.md` = contexto común a anteponer SIEMPRE;
   `ola3-harden-custody.md` = lo próximo).
4. `docs/AUDIT-LOG.md` — rondas 1–3 de auditoría del circuito/contrato (qué se cerró y por qué).
5. `docs/DECISIONS.md` + `docs/PLAN-DIA-2.md §2` — decisiones A–I **congeladas**.

---

## 2. Arranque rápido en el Mac (setup + verificación)

El Mac arranca en frío: **clona y reinstala el toolchain**. Shell del Mac suele ser **zsh** (el PC
viejo era bash) → citá paths con espacios y no asumas sintaxis bash-only. Verificá cada binario con
`--version` (nunca confíes en "exit 0" de un instalador).

```bash
# 1) Clonar
git clone https://github.com/manuelpenazuniga/zk--terroir.git
cd zk--terroir     # OJO: el repo se llama zk--terroir (doble guion) en GitHub

# 2) Identidad git (para commits). El PC viejo usaba:
git config user.name  "Manuel"
git config user.email "vale.lirah@gmail.com"     # ajústalo si querés otro autor

# 3) gh CLI autenticado como manuelpenazuniga (para push):
gh auth status        # debe mostrar 'manuelpenazuniga' como Active account
# si no: gh auth login  (cuenta manuelpenazuniga, protocolo https)

# 4) Toolchain a instalar en el Mac (verificá versiones tras instalar):
#    - Rust + target wasm:  rustup + `rustup target add wasm32v1-none`
#    - Stellar CLI:         `cargo install --locked stellar-cli`  (o brew) → `stellar --version` (era 27.0.0)
#    - Node:                node ≥ 20 (el PC viejo: v24.17.0) → trae `snarkjs` global (`npm i -g snarkjs`)
#    - Circom 2.1:          si vas a recompilar el circuito (Ola 3)
#    - jq:                  `brew install jq`   (lo usa verify/verify.sh)
#    - (Ola 3) circomlib está bajo spike/node_modules (gitignored) → `cd spike && npm install`

# 5) VERIFICAR que todo corre (esto NO debe fallar):
cd contracts/terroir && cargo test          # 11/11 verde (7 tests *_real con prueba real)
stellar contract build                       # wasm ~10.4 KB, hash b48085ad…
cargo clippy --all-targets                   # limpio
cd ../../circuits && snarkjs groth16 verify verification_key.json public.json proof.json   # OK!

# 6) Verificador público (contra Testnet, solo lectura):
cd .. && STELLAR_SOURCE=<tu_identidad> ./verify/verify.sh \
  2ceda2ee11f38491b484858a98c200d48c97ce21fdf8e9217a62634de6da6563
#   → ✓ Lote certificado — premium pagado el 2026-06-28 15:51:55 UTC
```

> Si algo no cuadra, re-orientate con el estado REAL (`git log`, `git status`, `stellar --version`),
> **no** con la narrativa. El repo es la verdad.

---

## 3. Los 3 CLIs de agentes (si vas a orquestar de verdad)

Estos CLIs son **externos** (no vienen del repo); reinstalalos/autenticalos en el Mac. Rutas del PC
viejo (van a cambiar en Mac): opencode `~/.opencode/bin`, agy/codex `~/.local/bin`.

- **opencode** (IMPLEMENTAR pesado): `opencode run --dir <worktree> -m opencode-go/<modelo> "$MSG"`.
  Provider SIEMPRE `opencode-go/…` (nunca `zen`). Modelo cerebro = **DeepSeek V4 Pro**.
- **agy** (AUDITAR Gemini, read-only): `agy --model "Gemini 3.1 Pro (High)" --add-dir <src> --print-timeout 900s -p "$MSG"`.
  `--add-dir` **acotado a fuentes** (nunca el worktree entero: arrastra target/ → cuelga).
- **codex** (auditor adversarial GPT-5.5, read-only): `codex exec -s read-only -m gpt-5.5 -c mcp_servers="{}" < brief.md`.
  ⚠️ **Pedí OK al usuario antes de CUALQUIER codex** (cuota fluctúa).

**Modelos VETADOS (no invocar):** Qwen 3.7 Max, GLM-5.2, Kimi K2.7 Code.

### Gotchas verificados esta sesión (¡importantes!)
- **agy NO respeta read-only** aunque lo lances sin `--dangerously-skip-permissions`: en la Ola 0
  sobrescribió `docs/briefs/_wrapper.md` y corrió `cargo test`. **Mitigación: corré agy en un dir de
  revisión aislado / worktree**, y buscá su reporte **en disco** (a veces no va a stdout).
- **El harness de Claude Code BLOQUEA `agy --dangerously-skip-permissions`** (modo worker/escribe):
  el clasificador lo marca "Create Unsafe Agents". Opciones: (a) el usuario añade una regla de permiso
  Bash; (b) **Claude implementa la tarea worker él mismo** (OK si NO toca fondos, como fue la Ola 1).
  El modo auditor read-only de agy sí corre.
- **codex `-s read-only` NO puede correr `cargo test`** (no puede escribir `target/.cargo-build-lock`)
  → tiende a emitir `NO-PASA — no pude verificar cargo test`. Es **falso negativo de entorno**, no del
  código. Vos ya corrés `cargo test` en el Paso 5; ese NO-PASA no es hallazgo de fondos.
- **La VK horneada se puede recomputar** desde `circuits/verification_key.json` (G1 `x‖y`, G2 swap
  `c1‖c0`) con un script Node y comparar byte-a-byte contra las constantes `VK_*` de `lib.rs`. Hacelo
  (los 3 auditores de la Ola 0 coincidieron en el match). Script de referencia: mirá cómo lo hizo la
  sesión anterior (recomputa alpha/beta/gamma/delta/IC0..IC7; `ic.len()` debe ser 8).

---

## 4. Lo que YA está hecho (no lo rehagas)

| Pieza | Estado | Dónde |
|---|---|---|
| Día 1 spike BN254 on-chain | ✅ | `spike/`, `docs/DECISIONS.md` D-001 |
| T1 v3 circuito 3 eslabones **SOUND** | ✅ auditado | `circuits/terroir_chain.circom` (H1 abierto/doc, H2/H3 cerrados) |
| T2 infra JS (árbol R_cert, witness) | ✅ | `circuits/js/` |
| T3-final contrato (VK horneada, bypass fuera, tests real, E2E) | ✅ **Día 2 CERRADO** | `contracts/terroir/`, tag `dia2-cerrado` |
| T5 token TUSDC (SAC testnet) | ✅ | `scripts/`, `deployments/testnet.json` |
| **Ola 1 · T3D-verify** (verificador QR read-only) | ✅ mergeado | `verify/` |
| **Ola 1 · T7-docs** (README + comentario circom) | ✅ mergeado | `README.md`, circom |

**Contrato en Testnet (Día 2):** `CBHFN7QUJJMA2RXMPVNYCFSCVZDQSOSVIRNVHJKPYHTE4DNHWX5ATJQQ`
(todas las direcciones/tx en `deployments/testnet.json`). Decisión A congelada: 7 señales públicas
`[r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash]`, `ic.len()==8`.

---

## 5. Lo PRÓXIMO: Ola 3 stretch (preparada, NO ejecutada)

**Objetivo:** endurecer custodia — orden de roles **finca→coop→tostador** (recupera H1) **sin romper
Decisión A**. Diseño congelado + aprobado por el usuario (2026-07-01).

**Idea (resumen; el detalle exacto está en el brief):** comprometer un ordinal de rol dentro del hash
de cada hoja acreditada (slot0=COOP `Poseidon(6)`, slot1=FINCA `Poseidon(4)`, slot2=TOSTADOR
`Poseidon(4)`). Roles = literales del circuito ⇒ orden canónico, **cero señales públicas nuevas** ⇒
el contrato **solo re-hornea la VK** (interfaz intacta). Frontera honesta: enforce cobertura+etiquetado
de rol (no-sustitución/no-omisión); el orden temporal *literal* quedaría fuera (rompería Decisión A).

**Cómo ejecutarla (gate estricto, es TOCA-FONDOS):**
1. Leé `docs/briefs/ola3-harden-custody.md` **entero** (tiene los cambios exactos de circuito+infra+VK,
   criterios de aceptación, un test adversarial de rol, y el checklist de auditoría triple).
2. Implementá (circuito = DeepSeek V4 Pro/opencode, o Claude si el worker está bloqueado). Regenerá
   prueba + `verification_key.json`; re-horneá VK en `lib.rs`; regenerá tests `*_real`.
3. Verificá vos (Paso 5): `snarkjs verify` OK, `cargo test` verde, `ic.len()==8`, test de rol rechaza.
4. **Auditoría TRIPLE** (Gemini 3.1 Pro High + GPT-5.5 [pedí OK] + checklist-Claude del brief).
5. **Re-deploy a Testnet + E2E** (happy paga / replay falla / tamper falla); actualizá `deployments/testnet.json`.
6. Regla §6: triple PASA **y** E2E verde → commit (+ tag opcional). Cualquier ALTA de fondos o
   desacuerdo → **STOP → usuario**. **Re-hornear VK sin re-auditar triple + re-deploy = prohibido.**

---

## 6. Reglas duras (STOP → usuario)

- No tocar el circuito sound ni reordenar señales públicas (Decisión A) **sin** el ciclo triple+deploy.
- No cambiar aritmética de premium / binding de floor o payout / definición de `lot_commit` o `nullifier`.
- No mergear nada con hallazgo ALTA de fondos, aunque otro auditor diga PASA.
- Antes de CUALQUIER codex/GPT-5.5: pedí OK. Push solo si el usuario lo pide.
- Detectá cuelgues de agentes: `ps -o pid,etime,time,%cpu` — 30+ min de reloj con ~0% CPU = colgado → kill.

---

## 7. Notas de publicación (repo ahora es público)

- `docs/internal/` y `docs/brainstorming/` **salieron del `.gitignore`** y ahora están en el repo
  público (routing de modelos, benchmarks, brainstorming, runbook). Fue decisión explícita del usuario.
- Barrido de secretos hecho antes del primer push: **sin secret seeds Stellar, sin .env/.pem/.key, sin
  tokens**. `deployments/testnet.json` solo lleva direcciones públicas + hashes de tx. Los `secret` que
  aparecen son `lot_secret`/prosa/valores demo del spike (inofensivos).
- Sigue gitignored: `**/node_modules`, `**/target`, `*.ptau/*.zkey/*.wtns/*.r1cs/*.sym/*.wasm`,
  `**/*_js/`, `.claude/settings.local.json`, `*.log`.
