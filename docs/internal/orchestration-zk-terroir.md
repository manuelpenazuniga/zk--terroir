# Runbook del orquestador — ZK-Terroir (multiagente: opencode · agy · codex)

> **Para el AGENTE ORQUESTADOR (menos capaz que el Claude que diseñó esto).**
> Tú **no diseñas ni auditas con criterio propio lo que toca fondos**: tú **ejecutas este runbook
> paso a paso**, corres comandos, lees strings de veredicto, y aplicas reglas mecánicas de PASA/
> NO-PASA. Cuando una regla diga **STOP → usuario**, te detienes y le pasas el control a un humano
> o a una sesión de Claude "cerebro". No improvises en el camino del dinero.
>
> Este doc es el **operativo verificado para ESTE repo**. El método conceptual está en
> `orchestration-workflow.md` (proyecto Ohu, genérico); la economía/routing en `model-routing.md`;
> la sintaxis de agy en `agy-cli.md`. **Ojo:** esos tres vienen de otro proyecto (Ohu/Casper/Odra)
> — las rutas, el toolchain (Odra≠Soroban) y el shell (zsh≠bash) están **corregidos aquí**. Si algo
> contradice a la salida real de un binario, **gana el binario** (re-verifica con `--version`/`--help`).

---

## 0. Verdad de este entorno (verificado 2026-07-01)

| Cosa | Valor real en esta máquina | ⚠️ El doc de Ohu dice otra cosa |
|---|---|---|
| Repo | `/home/manuel/proyectos/zk-terroir` | Ohu usaba `/Volumes/MacMiniExt/dev/web3/ohu/` |
| Shell | **bash** (`/bin/bash`) | Ohu decía "zsh" → **aquí NO**: usa sintaxis bash normal |
| Toolchain contrato | **Soroban / Stellar** (`stellar contract build`, `cargo test`, soroban-sdk 25.1.0) | Ohu era **Odra/Casper** (`cargo odra …`) → **NO aplica aquí** |
| Circuito | Circom 2.1 + snarkjs, curva **bn128/BN254** | igual |
| opencode | `~/.opencode/bin/opencode` **v1.16.2** | — |
| agy | `~/.local/bin/agy` **v1.0.14** | coincide con `agy-cli.md` |
| codex | `~/.local/bin/codex` **v0.142.2** | — |
| Rama principal | `main` | — |

Antes de arrancar una sesión, re-confirma: `command -v opencode agy codex stellar cargo snarkjs`.

---

## 1. Tu rol (orquestador) vs lo que ya hizo Claude por ti

**Claude (cerebro, ya hecho, no lo repites):** entendió el problema, congeló las decisiones de
diseño (`docs/PLAN-DIA-2.md §2`, Decisiones A–I), y **pre-escribió los briefs y los checklists de
auditoría** que vas a usar (en `docs/briefs/`). Tú **no tienes que diseñar nada nuevo**.

**Tú (orquestador) haces exactamente esto y nada más:**
1. Lees `docs/PLAN-DIA-3.md` → te dice **qué ola** toca y **qué tareas** corren en paralelo.
2. Por cada tarea: creas worktree → pegas el brief (un archivo de `docs/briefs/`) al CLI del modelo
   asignado → lo lanzas en background con log → **verificas tú mismo** (comandos, no el claim del
   agente) → corres los **auditores** que diga la tarea → aplicas la **regla mecánica de veredicto**
   (§6) → mergeas o repites.
3. Actualizas el estado en `docs/PLAN-DIA-3.md` (marca la tarea ✅/🔁/STOP) y avisas.

**Lo que NUNCA haces solo:** cambiar el circuito sound (T1 v3), reordenar las señales públicas
(Decisión A), tocar la aritmética de `premium`, o mergear algo con un hallazgo **ALTA en fondos**.
Eso es **STOP → usuario**.

---

## 2. Los tres CLIs (invocación verificada para este repo)

Prepara el mensaje SIEMPRE así (bash): `MSG="$(cat docs/briefs/<archivo>.md)"` y pásalo **inline**.

### 2.1 opencode — IMPLEMENTAR (trabajo pesado: contrato, circuito, infra)
```bash
opencode run --dir /home/manuel/proyectos/zk-terroir-wNN \
  -m opencode-go/<modelo> "$MSG"
```
- **SIEMPRE provider `opencode-go/…`**, NUNCA `zen` (falla: "No payment method").
- **No** requiere `--dangerously-skip-permissions` (ejecuta tools dentro de `--dir`).
- Prompt **inline**; NO uses `-f` (es array-greedy y se traga el mensaje como filename).
- IDs esperados: `opencode-go/deepseek-v4-pro`, `opencode-go/minimax-m3`, `opencode-go/minimax-m2.7`.
  **Verifica los IDs reales antes del primer uso:** `opencode models 2>/dev/null | grep opencode-go`.

### 2.2 agy — AUDITAR (Gemini, read-only) o WORKER ligero (tests/docs/infra)
```bash
# AUDITAR (read-only): NO --dangerously-skip-permissions. --add-dir ACOTADO a las fuentes.
agy --model "Gemini 3.1 Pro (High)" \
  --add-dir /home/manuel/proyectos/zk-terroir-wNN/contracts/terroir/src \
  --print-timeout 900s -p "$MSG"

# WORKER (escribe): SÍ --dangerously-skip-permissions.
agy --model "Gemini 3.5 Flash (High)" \
  --add-dir /home/manuel/proyectos/zk-terroir-wNN \
  --dangerously-skip-permissions --print-timeout 900s -p "$MSG"
```
- `--model` exige el string **EXACTO** de `agy models` (con paréntesis y mayúsculas). Verifica:
  `agy models`. Esperados: `"Gemini 3.1 Pro (High)"`, `"Gemini 3.5 Flash (High)"`, `"… (Medium)"`.
- **`--add-dir` acotado a la carpeta de fuentes** (`contracts/terroir/src` o `circuits`), **NUNCA el
  worktree entero** (arrastra `target/`, `node_modules/` → agy **se cuelga** 30+ min).
- `--print-timeout 900s` (el default de 5m corta audits largos).
- agy **rechaza** el framing "pentest/security audit" → los briefs ya están fraseados como
  "revisa MI código pre-merge por bugs de correctness/conservación" (no lo cambies).

### 2.3 codex — AUDITORÍA ADVERSARIAL DE CIERRE (GPT-5.5, read-only)
```bash
codex exec -s read-only -m gpt-5.5 -c mcp_servers="{}" < docs/briefs/<archivo>.md
```
- Prompt por **STDIN** (`< archivo`), **NO** como argumento (si lo pasas como arg, imprime
  "Reading additional input from stdin…" y no arranca).
- `-s read-only` (no edita). Modelo **`gpt-5.5`** a secas (NO `gpt-5.5-codex` en cuenta ChatGPT).
- ⚠️ **CUOTA FLUCTÚA → REGLA DURA: antes de CUALQUIER `codex`, STOP y pregunta al usuario**
  "¿corro el auditor GPT-5.5 (codex) ahora?" y espera su OK. No lo lances sin confirmación.

---

## 3. Routing de modelos (sigue `model-routing.md`; VETADOS abajo)

**🚫 VETADOS (no los invoques nunca) — retirados por costo:**
- **Qwen 3.7 Max** · **GLM-5.2** (quemaron una cuenta de Ohu en 1 día).
- **Kimi K2.7 Code** (caro: T2 escaso, ~1.350 req/5h, $0.95/$4.00, sin benchmark independiente).

> Nota histórica: el **Día 2** de ESTE repo se implementó con GLM-5.2 (antes del veto). Ese trabajo
> ya está hecho y auditado; **de aquí en adelante NO se usa GLM-5.2**. El cerebro de contrato/
> circuito pasa a **DeepSeek V4 Pro** (con escalación a Gemini 3.1 Pro High).

**Tabla de asignación (lo que SÍ usas):**

| Tipo de tarea | Implementa (primario) | Escala a | Audita |
|---|---|---|---|
| **Contrato Rust que toca fondos** (`claim_premium`, transfers, storage) | **DeepSeek V4 Pro** (opencode) | Gemini 3.1 Pro High (agy) | **triple:** Gemini 3.1 Pro High + GPT-5.5 + checklist-Claude |
| **Circuito Circom** (soundness, constraints) | **DeepSeek V4 Pro** (opencode) | Gemini 3.1 Pro High (agy) | triple (es el corazón; toca fondos indirectamente) |
| Infra / scripts / deploy / CI | **MiniMax M3** (opencode) o Gemini 3.5 Flash High (agy) | DeepSeek V4 Pro | — |
| Tests (unit/integration) | **Gemini 3.5 Flash (Medium)** (agy) | MiniMax M2.7 (opencode) | — |
| Frontend / página de verificación (QR) | **Gemini 3.5 Flash (High)** (agy) | MiniMax M3 | — |
| Docs (README, comentarios) | **Gemini 3.5 Flash (Medium)** (agy) | MiniMax M3 | — |
| Comprensión de repo (extender, no reconstruir) | **DeepSeek V4 Pro** (1M ctx) | MiniMax M3 | — |
| Diseño / brief / decisión de merge / audit de fondos | **Claude / usuario** (NO tú) | — | — |

**Regla de proporción (control de gasto):** manda ≥80% de subtasks a workers ligeros (Flash/M3);
escala a premium (DeepSeek V4 Pro) solo lo pesado. Si escalas >20% a premium, algo está mal enrutado.

---

## 4. El ciclo por tarea (paso a paso, mecánico)

Para cada tarea `Tk` de la ola actual en `docs/PLAN-DIA-3.md`:

**Paso 1 — Worktree aislado** (uno por tarea; no dejes que los agentes se pisen):
```bash
cd /home/manuel/proyectos/zk-terroir
git worktree add /home/manuel/proyectos/zk-terroir-wNN -b spike/wNN-<tarea> main
```

**Paso 2 — Brief**: la tarea en el plan te dice qué archivo de `docs/briefs/` usar.
```bash
MSG="$(cat docs/briefs/<archivo>.md)"
```
No edites el brief salvo sustituir marcadores literales (`<worktree>` → la ruta real).

**Paso 3 — Lanzar en background** con log (para no bloquear y poder vigilar cuelgues):
```bash
opencode run --dir /home/manuel/proyectos/zk-terroir-wNN -m opencode-go/deepseek-v4-pro "$MSG" \
  > /tmp/wNN.log 2>&1 &
echo "PID=$!"
```
(En Claude Code: `run_in_background: true` y lee el log.)

**Paso 4 — Vigila cuelgues (watchdog).** Compara **CPU acumulada vs reloj**:
```bash
ps -o pid,etime,time,%cpu -p <PID>
```
Si lleva 30+ min de reloj pero **segundos de CPU** y `%cpu~0` → está **colgado** (backend no
responde), no trabajando. `kill <PID>` y re-lanza. Lanza un aviso a los ~9 min:
`( sleep 540; ps -o pid,etime,time,%cpu -p <PID>; git -C /home/manuel/proyectos/zk-terroir-wNN log --oneline -3 ) &`

**Paso 5 — VERIFICA TÚ MISMO (nunca confíes en el "hecho" del agente).** Comandos de este repo:
- Contrato: `cd /home/manuel/proyectos/zk-terroir-wNN/contracts/terroir && cargo test && stellar contract build && cargo clippy --all-targets`
- Circuito: `cd .../circuits && ./gen_proof.sh && snarkjs groth16 verify verification_key.json public.json proof.json`  (debe decir `OK`)
- Confirma que **commiteó**: `git -C /home/manuel/proyectos/zk-terroir-wNN log --oneline -3` y `git … status`.
  Si hizo el trabajo pero **no commiteó**, commitea tú (un reinicio del entorno puede perderlo).

**Paso 6 — Audita** según el nivel de la tarea (§5). Auditores en **paralelo** (background).

**Paso 7 — Veredicto mecánico** (§6): convergen en PASA → merge; alguno NO-PASA con hallazgo real
→ ronda de fix (vuelve al Paso 2 con el brief de fix); disagreement o ALTA-en-fondos → **STOP → usuario**.

**Paso 8 — Merge + limpieza + estado:**
```bash
cd /home/manuel/proyectos/zk-terroir
git merge --ff-only spike/wNN-<tarea>
git worktree remove /home/manuel/proyectos/zk-terroir-wNN
git branch -d spike/wNN-<tarea>
# marca la tarea ✅ en docs/PLAN-DIA-3.md, commitea el estado, y (si el usuario lo pidió) push.
```

---

## 5. Disciplina de auditoría (por nivel de riesgo)

| Tipo de cambio | Auditoría requerida |
|---|---|
| **No toca fondos** (docs, página QR de solo-lectura, tests, comentarios) | **dual:** checklist-Claude (pre-escrito) + Gemini 3.1 Pro High (agy) |
| **Toca fondos** (contrato: transfer/premium/nullifier/root/floor; circuito) | **triple:** Gemini 3.1 Pro High (agy) + **GPT-5.5 (codex, pide OK)** + checklist-Claude |
| **Antes de declarar cerrada una fase / antes de tocar Testnet** | **pase adversarial de cierre** con GPT-5.5 (familia distinta) — es un gate, no un trámite |

**Por qué el panel NO es redundante (no lo saltes):** cada familia caza una clase distinta de bug.
Gemini = correctness + prueba algebraica; GPT-5.5 = adversarial/juego ("¿quién gana dinero rompiendo
la regla?"); el checklist-Claude = conservación/estructura (CEI, checked_*, orden de señales). "Compila
y conserva fondos" **no** es "no se puede saquear". En Ohu, el pase GPT-5.5 halló drenajes que 176
tests verdes y el análisis por-tarea NO vieron. **Nunca saltes el pase adversarial antes de cerrar.**

Los **checklists-Claude** ya están escritos en los briefs de `docs/briefs/` (secciones "Checklist de
auditoría"). Tú los aplicas ítem por ítem sobre el código real; no los inventas.

---

## 6. Regla mecánica de veredicto (cómo decides sin criterio propio)

Cada brief de auditoría termina obligando al modelo a imprimir una última línea:
`VEREDICTO: PASA` **o** `VEREDICTO: NO-PASA — <razón>`.

```
recoge veredictos de: agy(Gemini) [+ codex(GPT-5.5) si toca fondos] + tu checklist manual
1. TODOS dicen PASA  ............................→ MERGE (Paso 8)
2. Alguno dice NO-PASA con hallazgo concreto ...→ RONDA DE FIX (nuevo brief de fix; NO mergees)
3. Auditores se CONTRADICEN entre sí ...........→ STOP → usuario (no desempates tú)
4. Cualquier hallazgo de severidad ALTA que
   toque FONDOS (drenaje, doble-cobro, redirigir
   premium, bypass de root/floor/nullifier) .....→ STOP → usuario (aunque otro diga PASA)
5. Un auditor se cuelga / no imprime VEREDICTO ..→ re-lanza 1 vez; si vuelve a fallar, STOP → usuario
```
Cuando hagas **STOP → usuario**, entrega: la tarea, los logs de los auditores, y el diff. No mergees.

---

## 7. Gotchas de ESTE repo (Soroban/Stellar, no Odra/Casper)

- **`odra-test` no existe aquí.** Es Soroban: tests con `soroban_sdk::testutils` (`cargo test`), build
  con `stellar contract build`. Ignora cualquier `cargo odra …` de los docs de Ohu.
- **Verifica en el nodo real, no solo en tests.** `soroban_sdk` (host VM de test) no serializa igual
  que Testnet. Bugs que solo el nodo expone:
  - **Trustline del payout:** si `payout` es cuenta **G…**, necesita **trustline a TUSDC** o el
    `transfer` revierte (tumba el primer E2E). Establécela o usa dirección **C…**. (Ver PLAN-DIA-2 §8.3.)
  - **wasm bulk-memory/sign-ext:** el toolchain moderno puede emitir opcodes que el nodo rechaza →
    `wasm-opt` lowering a MVP si el deploy falla (ver `scripts/deploy.sh`).
- **VK horneada:** el contrato lleva la VK del circuito **T1 v3** como constantes (`lib.rs` `VK_*`).
  Si alguien regenera el circuito, la VK **debe** re-serializarse con `circuits/serialize.js`
  (swap G2 `c1‖c0`). VK vieja + circuito nuevo = todas las pruebas fallan. **STOP → usuario** si un
  agente toca el circuito sin re-hornear la VK.
- **NUNCA confíes en "exit 0"** de instalaciones: verifica el binario (`which`, `--version`).
- **Paths con espacios** (claves de wallet) → citar SIEMPRE en `.env` y comandos.
- **Reinicios del entorno** scramblean el transcript: si algo no cuadra, re-oriéntate con el estado
  REAL (`git log`, `git worktree list`, `ps`), NO con tu narrativa. El repo es la verdad.

---

## 8. Arranque rápido (qué haces al retomar)

```bash
cd /home/manuel/proyectos/zk-terroir
git status && git log --oneline -5 && git worktree list   # ¿qué está a medias?
sed -n '1,40p' docs/PLAN-DIA-3.md                          # ¿qué ola toca?
```
Ejecuta la ola actual siguiendo §4–§6. Ante la más mínima duda sobre **fondos**, **STOP → usuario**.
Tu éxito se mide en tareas **auditadas y mergeadas sin drenar el escrow**, no en velocidad.
</content>
</invoke>
