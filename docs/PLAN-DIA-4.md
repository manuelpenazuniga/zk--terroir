# Plan Día 4 — Próximos pasos (post-Ola 3)

> **Contexto.** Olas 0–3 están **cerradas y mergeadas a `main`** (working tree limpio, tag
> `ola3-cerrada`). El DoD del Día 3 (`docs/PLAN-DIA-3.md §1`) puntos 1–3 está cumplido y el punto 4
> (stretch, role-tag custodia) también. Este documento define **lo que viene**: cerrar los hilos
> abiertos, tapar los huecos de reproducibilidad/honestidad que la revisión de código destapó, y
> —si el objetivo lo pide— construir el "wow moment" (proving en navegador) y el video que el spec
> original promete pero que **todavía no existen en el repo**.
>
> Redactado por Claude (planifica + audita). Se ejecuta con el sistema multiagente de
> `docs/internal/orchestration-zk-terroir.md`. **Regla de oro intacta:** nada que toque fondos se
> cierra sin verificación reproducible + auditoría; cualquier hallazgo ALTA de fondos → STOP → usuario.

---

## 0. Meta y prioridad (DECIDIDO por el usuario, 2026-07-02)

El **deadline está extendido** (confirmado por el usuario) → **Escenario A**. La prioridad es:

> **Ola 4 (integridad) y Ola 5 (proving en navegador) salen YA, en ese orden.** El "wow moment"
> (proving client-side) es el objetivo inmediato; la Ola 4 lo precede porque deja el repo reproducible
> y publicable antes de construir encima. Ola 6 (video) sigue a Ola 5; Ola 7 (endurecimiento) después.

**Decisiones del usuario aplicadas en este plan:**
- **H-C (reproducibilidad):** hacerla **con time-box** — si no toma mucho, se arregla; si se complica,
  **se salta documentando el gap** honestamente en el README (ver T4.1).
- **codex/GPT-5.5:** **pospuesto** para más adelante. El gate de las tareas que tocan circuito pasa a
  ser **dual (checklist-Claude + Gemini)**; si mi juicio + Gemini dan OK, **se avanza** al siguiente
  paso (codex queda como 3er leg de refuerzo posterior, no bloqueante).
- **`git push`:** **aprobado** (T4.4).
- **Ola 7:** empezar por **doble-membership ASP de 2 niveles** (confirmado).

> **Nota de rol:** Claude (este agente) **solo planifica y audita**; la implementación de cada tarea la
> hacen los agentes de OpenCode (routing en `docs/internal/model-routing.md`), y Claude corre la
> compuerta de auditoría antes de cerrar. Este documento es el plan, no la ejecución.

---

## 1. Hilos abiertos heredados + hallazgos nuevos de la revisión de código

Cuatro cosas quedan sin cerrar. Dos venían anotadas (memoria/parked-state); **dos son hallazgos
nuevos** de leer el código (no los reportes) hoy.

| # | Hilo | Origen | Severidad | Toca fondos |
|---|---|---|---|---|
| **H-A** | **Repo sin publicar.** `origin` existe (`github.com/manuelpenazuniga/zk--terroir.git`) pero **nunca se hizo `git push`**. El hackathon exige **repo open-source**; hoy es privado/local. | parked-state | **Bloqueante para "entrega"** | no |
| **H-B** | **3er leg adversarial (codex/GPT-5.5) sobre Ola 3 pendiente.** La Ola 3 (role-tag, toca circuito de fondos) se cerró con dual-audit + pase adversarial-Claude **en lugar de** codex, por cuota. El 3er auditor independiente sigue debiendo. | parked-state | Media (audit de fondos incompleta) | sí (audita) |
| **H-C** | **Reproducibilidad rota en clon limpio (HALLAZGO NUEVO).** `.gitignore` excluye `node_modules/`, `*_js/`, `*.wasm`, `*.zkey`, `*.ptau`; **circomlib no está trackeado en ningún lado** (ni en `spike/`). Como `terroir_chain.circom` hace `include "../spike/node_modules/circomlib/…"`, el **paso 1 del README** (`cd circuits && ./gen_proof.sh`) **falla en cualquier máquina que no sea esta**. | revisión de código | **Alta (credibilidad ante jueces)** | no |
| **H-D** | **Señales muertas en el circuito sound (HALLAZGO NUEVO).** `terroir_chain.circom:165-168`: `h2 <== payout_hi*payout_hi; l2 <== payout_lo*payout_lo;` se **calculan y nunca se usan** — mismo *smell* que el `chain[3]` muerto que una auditoría previa marcó. El binding real de payout lo hacen `Num2Bits(128)` (vivo, líneas 160-163) + el contrato; estas dos constraints anti-maleabilidad son **inertes**. No es una vulnerabilidad (no relajan nada), pero ensucian el circuito y un auditor lo marcará. | revisión de código | Baja (cleanup / audit-hygiene) | sí (toca circuito) |

**Matiz de honestidad adicional (no es un hilo, es precisión para README/video):** el contrato
combina los 7 public inputs con `bn.g1_mul` **en un loop** + `bn.g1_add` (`lib.rs:248-252`), no con
una host-function **MSM batcheada**. Es correcto y nativo, pero el README dice "MSM native". Ante un
juez ZK afilado conviene precisar: *"la combinación lineal de public inputs corre sobre `g1_mul`/`g1_add`
nativos (P26); es la operación MSM hecha con scalar-muls nativos, no una precompilación MSM dedicada."*
Con 7 inputs la diferencia de costo es irrelevante; la de **precisión narrativa** no.

---

## 2. La brecha grande: el "wow moment" prometido no existe en el repo

El spec es explícito en tres lugares:
- **`zk-terroir.md §6` (Día 3):** *"Frontend: la marca carga la cadena (mock), genera la prueba **en
  el navegador**, ve el pago; el consumidor escanea el QR y ve 'verificado + tx'."*
- **`zk-terroir.md §8`** (guion del video) y **§12** (checklist de entrega): *"Video 2–3 min mostrando
  **proving en navegador** → verificación → pago."*
- **`techs-specs §5.2`:** *"Haz visible el pago cross-border… ponlo en primer plano del video."*

**Estado real:** el proving es **CLI** (`circuits/gen_proof.sh`, snarkjs vía `npx`). El verificador
público (`verify/`) es **bash + `stellar` CLI** (read-only, correcto para el QR). **No hay** página
web, **no hay** proving en navegador, **no hay** video. La materia prima **sí existe** y es lo bueno:
`circuits/terroir_chain_js/terroir_chain.wasm` (generador de witness, 9 MB) + `terroir_chain_0001.zkey`
(8.7 MB) son exactamente lo que se carga en un navegador con snarkjs para probar client-side.

Cerrar esta brecha es la **Ola 5**. Es la diferencia entre "un backend que verifica" y el pitch
que gana ("la prueba se generó en **mi** navegador; los datos nunca salieron").

---

## 3. Olas de trabajo (DoD, gate, modelo, brief)

Routing de modelos según `docs/internal/model-routing.md`: **cerebro de circuito/contrato = DeepSeek
V4 Pro** (escala a **Gemini 3.1 Pro High**); infra/docs/frontend = MiniMax M3; auditoría = auditores
**independientes del implementador**. GLM-5.2 / Qwen 3.7 Max retirados; Kimi K2.7 Code vetado.

### Ola 4 — Integridad y reproducibilidad (INCONDICIONAL, barata, casi no toca fondos)

Objetivo: que el repo sea **honesto, reproducible y publicable**, y cerrar la deuda de auditoría.
Cuatro tareas; las dos primeras son independientes y paralelizables.

#### T4.1 · Reproducibilidad del circuito (cierra **H-C**) · MiniMax M3 · **no toca fondos**
**Problema:** clon limpio no compila el circuito (circomlib ausente; rutas apuntan a
`spike/node_modules`). **Deliverable (elige UNA estrategia, en orden de preferencia):**
1. **`circuits/package.json`** con `"circomlib": "^2.0.5"` como dependencia + cambiar los `include` de
   `terroir_chain.circom` a `circomlib/circuits/…` (resolución vía `-l node_modules`), y documentar
   `npm install` como paso 0 en README + `gen_proof.sh`. *(Preferida: liviana, estándar, no infla git.)*
2. **Vendorizar** los 4 ficheros de circomlib que se usan (`poseidon.circom`, `mux1.circom`,
   `comparators.circom`, `bitify.circom` + sus deps) bajo `circuits/lib/` y commitearlos. *(Fallback si
   la resolución de includes se complica; hace el repo 100% self-contained sin `npm install`.)*
**Aceptación:** en un `git clone` fresco (simular con `git archive | tar` en /tmp), `cd circuits &&
npm ci && ./gen_proof.sh` (o el paso vendorizado) llega a `snarkjs groth16 verify … OK` **sin** tocar
`spike/node_modules`. **Gate:** dual (checklist-Claude + un auditor) — **no toca lógica del circuito**
(solo includes/build), así que la VK **no debería cambiar**; si cambia, es regresión → re-hornear +
re-auditar. *(Verificar hash de `verification_key.json` antes/después: debe ser idéntico.)*

> **⏱️ TIME-BOX (decisión del usuario):** la estrategia 1 (`package.json` + `-l`) es rápida y es la
> que se intenta. **Si en ~30–45 min no queda compilando limpio** (p.ej. la resolución de includes se
> enreda con las deps transitivas de circomlib), **NO** se persigue: se **salta** y se documenta el gap
> honestamente en el README — *"la regeneración del circuito desde cero requiere circomlib local; los
> artefactos verificables (`verification_key.json`, `proof.json`, `public.json`, `serialized.json`)
> están commiteados y la verificación on-chain es reproducible sin regenerar."* No bloquea la Ola 5.

> **Sub-decisión (documentar en README, no reabrir):** los artefactos pesados regenerables
> (`*.ptau/*.zkey/*.wtns`) **siguen gitignored** — se regeneran con `gen_proof.sh`. Lo que importa es
> que `verification_key.json`, `proof.json`, `public.json`, `serialized.json` **ya están commiteados**,
> así que la **verificación on-chain es reproducible sin regenerar el circuito**. La regeneración del
> circuito (para auditar soundness) es lo que T4.1 arregla.

#### T4.2 · Cerrar el 3er leg adversarial (cierra **H-B**) + limpiar señales muertas (cierra **H-D**)
**Dos sub-tareas acopladas porque ambas tocan el circuito role-tag:**

- **T4.2a · codex/GPT-5.5 sobre Ola 3 — POSPUESTO (decisión del usuario).** No se corre ahora; queda
  como **3er leg de refuerzo posterior**. El gate efectivo pasa a **dual (checklist-Claude + Gemini
  3.1 Pro High)** sobre el diff que toque circuito: **si mi juicio + Gemini dan OK, se avanza** al
  siguiente paso. Cuando haya cuota, correr codex con `docs/briefs/ola3-harden-custody.md` en sandbox
  con cwd aislado (**gotcha reconfirmado:** agy/agentes escapan read-only aun con `--add-dir` → aislar cwd).
- **T4.2b · quitar `h2`/`l2` muertas** (`terroir_chain.circom:165-168`). ⚠️ **TOCA EL CIRCUITO SOUND** →
  cambia el R1CS → **re-hornear VK** → re-serializar → re-deploy → re-E2E. **Por eso NO se hace suelto.**
  Como codex está pospuesto y esto es un cleanup **no-funcional**, la recomendación es **NO re-hornear la
  VK auditada solo por cosmética**: dejar `h2/l2` con un comentario `// dead, kept to preserve audited VK`
  y anotarlo. **Solo** se hace el re-spin si Ola 7 (que igual toca el circuito) lo arrastra — ahí se
  limpia `h2/l2` en el mismo re-hornear y se ahorra una VK.

**Gate T4.2:** dual (Claude + Gemini). Si Gemini o Claude ven **ALTA de fondos** → STOP → usuario. Si
ambos dan OK → **se avanza** (codex queda como refuerzo pendiente en la memoria `ola3-parked-state`,
no bloquea). Si hubo re-spin de circuito: VK recomputada byte-a-byte == `lib.rs` + `cargo test` verde +
E2E Testnet verde.

#### T4.3 · Precisión de honestidad en README (matiz MSM + trusted-setup) · MiniMax M3 · no toca lógica
- Precisar la frase MSM (§1 de este doc): `g1_mul`/`g1_add` nativos en loop, no MSM batcheada.
- **Añadir nota honesta de trusted setup:** `gen_proof.sh` corre un Powers-of-Tau **de juguete** (una
  sola contribución, entropía hardcodeada `-e="terroir-chain-1"`). Para un MVP está bien, pero el README
  debe decir **"setup no ceremonial; en producción, ceremonia multi-party"** — la convocatoria premia
  exactamente esta clase de honestidad ("honest WIP > polished mystery").
**Aceptación:** README no sobre-vende ninguna pieza; `cargo build` sigue verde (no toca código).

#### T4.4 · Publicar el repo (cierra **H-A**) · **APROBADO por el usuario**
Acción manual, no de agente: revisar que no haya secretos (ya auditado: `deployments/testnet.json` solo
direcciones públicas; claves en keystore local, no en git), luego `git push origin main --tags`.
**Orden:** publicar **después** de T4.1 + T4.3. Si T4.1 se **saltó** por time-box, publicar igual pero
con el README **ya honesto sobre el gap de regeneración** (T4.3), para que el clon no sorprenda a nadie.
Publicar antes de T4.3 = arriesgar que el primer juez lea un README que sobre-vende.

---

### Ola 5 — El "wow": proving en el navegador (cierra la brecha del §2)

**Objetivo inmediato tras la Ola 4** (Escenario A). Es el entregable de mayor impacto/riesgo del proyecto.

#### T5.1 · Frontend estático de proving client-side · MiniMax M3 (escala a DeepSeek si se atasca la serialización)
**Deliverable:** `web/` (o `frontend/`) — **una página estática, sin backend** — que:
1. Carga `snarkjs` (bundle o CDN pinneado) + `terroir_chain.wasm` + `terroir_chain_0001.zkey` (servidos
   como assets estáticos; ~18 MB combinados, aceptable para demo — mostrar spinner de carga).
2. Toma **datos mock de la cadena** (3 eslabones finca→coop→tostador, precargados/editables) y **reusa
   la lógica de `circuits/gen_input.js` + `circuits/js/buildTree.js`** portada a browser para armar el
   `input.json` (árbol R_cert Poseidon, hojas role-tag, paths).
3. Llama `snarkjs.groth16.fullProve(input, wasm, zkey)` **en el navegador** → obtiene `proof` + `publicSignals`.
4. **Serializa** al layout BN254 (reusar `circuits/serialize.js`, swap G2 `c1‖c0`) y **arma la tx**
   `claim_premium(proof, pub_signals, payout)` contra el contrato de `deployments/testnet.json`
   (firmar con Freighter/wallet, o mostrar el comando `stellar` listo para pegar si no se integra wallet).
5. Muestra el **pago** (delta de balance de la coop / tx hash) y el **QR** (`verify/gen_qr.sh` ya define
   el payload `zkterroir:verify?lot_commit=…`) → link al verificador read-only.

**El momento que gana (ponerlo en pantalla):** un cartel *"la prueba se generó en tu navegador — estos
datos nunca salieron de tu equipo"* mientras se ve el spinner de `fullProve`.

**Riesgos técnicos (de-riskear en este orden):**
- **La prueba del navegador debe ser idéntica a la del CLI.** Poseidon de `circomlibjs` (browser) ==
  Poseidon de circomlib (circuito) — ya está verificado JS↔circuito (audit T2), pero re-confirmar que
  `fullProve` en browser produce un `public.json` que **verifica on-chain `true`** contra la VK
  horneada. *Este es el único punto que puede obligar a trabajo real; lo demás es plomería/UI.*
- **Tamaño del zkey (8.7 MB):** cargable, pero lento en móvil. Aceptable para demo desktop; documentar.
- **Firma de la tx:** integrar Freighter es lo ideal; **fallback honesto** = generar la prueba en
  browser y mostrar el `stellar contract invoke` exacto para que el operador lo ejecute (el proving —lo
  ZK-load-bearing— ya ocurrió en el navegador; la firma es plomería).

**Gate:** **no re-audita fondos** (usa el contrato + VK **ya auditados**; el frontend no cambia lógica
on-chain). Auditoría = **dual sobre el frontend** (no filtra secretos: mock keys claramente marcadas;
no hay claves de escritura embebidas) + **prueba E2E manual**: generar prueba en navegador → verifica
on-chain `true` → paga premium → QR resuelve. **Aceptación:** demo reproducible en Chrome; una prueba
manipulada en el form → on-chain `false`/panic.

---

### Ola 6 — Video 2–3 min (checklist §12)

Depende de Ola 5 para la mejor versión (proving en vivo), pero **se puede grabar con el CLI** si Ola 5
se descarta. **Guion ya escrito** en `zk-terroir.md §8`; **plano técnico clave** definido ahí
(finca→coop→tostador, candados "∈ set acreditado", invoice "≥ piso", una flecha pública `prueba → USDC`).
**Deliverable:** MP4 2–3 min. **Herramienta disponible:** el entorno tiene automatización de Chrome
(`gif_creator`) para capturar el flujo del frontend sin edición manual. **Aceptación:** el video muestra,
en orden, (1) proving en navegador, (2) verificación on-chain, (3) pago USDC a la coop, (4) QR del
consumidor — los 4 hitos del checklist. **No es tarea de agente de código**; es captura + narración.

---

### Ola 7 — Endurecimiento del primitivo (STRETCH, solo con diseño aprobado)

⚠️ **TODAS tocan el circuito sound → re-hornear VK → triple audit (incl. codex) → re-deploy → re-E2E.**
El orquestador **NO** las lanza solo: requieren que **Claude/usuario diseñe y apruebe el brief** (misma
regla que fue la Ola 3). Backlog priorizado (de `PLAN-DIA-2 §2 stretch` + `PLAN-DIA-3 §1.4`):

| Candidato | Qué añade | Fidelidad al spec | Costo |
|---|---|---|---|
| **`region_root`** (origen elegible) | prueba `region ∈ set permitido` en el eslabón finca — el predicado "origen elegible" del spec §3.2.3 que hoy **no** se prueba | Alta (cierra un predicado prometido) | Circuito + posible 8º público → **rompe Decisión A/`IC.len()==8`** → contrato + VK. **El más caro.** |
| **Doble-membership (ASP 2 niveles)** | hoy `leaf ∈ R_cert` es **un** nivel; el spec §3.1 pide *atestación ∈ subárbol del certificador* **∧** *certificador ∈ set acreditado* | Alta (es el modelo ASP completo) | Circuito (2 memberships por eslabón) + VK; **sin** públicos nuevos (Decisión A intacta) → **mejor ratio valor/costo** |
| **Revivir hash-chain (orden estricto)** | role-tag (Ola 3) ata *rol↔hoja* pero no impone **secuencia** finca→coop→tostador; la cadena de custodia real recupera H1 | Media (role-tag ya da 80% de la propiedad) | Circuito + exponer `chain[N]` como público → **rompe Decisión A** → contrato + VK |

**DECIDIDO (usuario):** Ola 7 empieza por **doble-membership ASP de 2 niveles** — máxima fidelidad al
patrón ASP del spec §3.1 **sin tocar los públicos** (Decisión A intacta → sin re-tocar el parseo del
contrato → menor superficie de re-auditoría de fondos). `region_root` y revivir la hash-chain implican
un público nuevo → romper Decisión A → quedan **después**, como candidatos 2 y 3.

**Aprovechar el re-spin:** como doble-membership ya toca el circuito y re-hornea la VK, es el momento de
**limpiar `h2/l2` muertas (T4.2b)** en el mismo horneado — se cierra H-D sin una VK extra.

**Gate Ola 7 (por candidato):** diseño aprobado por humano → brief pre-escrito → implementa (DeepSeek
V4 Pro) → **triple audit** (Claude + Gemini + codex, porque toca fondos) → VK recomputada byte-a-byte →
`cargo test` + E2E Testnet verde → memoria actualizada.

---

## 4. Riesgos y gotchas transversales (heredados + nuevos)

| Riesgo | Dónde muerde | Mitigación |
|---|---|---|
| **Clon limpio no compila** (H-C) | Primer juez que clone → mala primera impresión | Ola 4 T4.1 **antes** de publicar (T4.4) |
| **agy/agentes escapan read-only** aun con `--add-dir` | Auditorías tocan `input.json`/`tree.json` del repo real | cwd sandbox aislado; restaurar artefactos post-audit (ya documentado en parked-state) |
| **Trustline del payout** | El primer E2E con cuenta `G…` revierte si no tiene trustline a TUSDC | Ya resuelto en Ola 3 (payout `zkq-t0` con trustline); si se re-deploya, re-establecer (`PLAN-DIA-2 §8.3`) |
| **Re-hornear VK dos veces** | Cualquier cambio al circuito (T4.2b, Ola 7) | Agrupar todos los cambios de circuito en **un** re-spin por ola |
| **zkey 8.7 MB en browser** (Ola 5) | Carga lenta en móvil | Demo desktop; spinner; documentar |
| **Prueba browser ≠ prueba CLI** (Ola 5) | Si Poseidon browser difiere → on-chain `false` | Re-confirmar `fullProve` browser verifica on-chain antes de invertir en UI |
| **Trusted setup de juguete** | Un juez ZK pregunta por la ceremonia | Nota honesta en README (T4.3) — no es blocker, es encuadre |

---

## 5. Orden de ejecución (DECIDIDO — Escenario A, deadline extendido)

```
YA (en orden):
  Ola 4  →  T4.1 (repro, TIME-BOX ~30–45min o se salta)
         →  T4.2 (gate dual Claude+Gemini; codex pospuesto; h2/l2 se difiere a Ola 7)
         →  T4.3 (README honesto: MSM + trusted-setup + gap de repro si se saltó T4.1)
         →  T4.4 (git push, APROBADO)
  Ola 5  →  proving en navegador (el "wow")   ← objetivo inmediato tras Ola 4

DESPUÉS:
  Ola 6  →  video 2–3 min (sobre el frontend de Ola 5)
  Ola 7  →  doble-membership ASP 2 niveles (1º) + limpiar h2/l2 en el mismo re-spin
            → luego region_root / hash-chain (rompen Decisión A → van al final)
```

**Recordatorio de rol:** este documento es **el plan**. La ejecución la hacen los agentes de OpenCode
bajo el runbook `docs/internal/orchestration-zk-terroir.md`; Claude audita cada ola antes de cerrarla.

## 6. Estado de decisiones (todas resueltas por el usuario 2026-07-02)

1. ✅ **Meta = Escenario A** (deadline extendido). Olas 4 y 5 salen ya.
2. ✅ **codex/GPT-5.5 pospuesto** → gate dual (Claude + Gemini); si ambos OK, se avanza. Codex = refuerzo posterior.
3. ✅ **`git push` aprobado** (T4.4), tras T4.1 + T4.3.
4. ✅ **Ola 7 = doble-membership ASP de 2 niveles primero**.
5. ⏱️ **H-C con time-box** — si se complica, se salta documentando el gap (T4.1).

**Pendiente de humano solo para Ola 7:** aprobar el **brief de diseño** de doble-membership antes de
tocar el circuito sound (misma regla de gate que fue la Ola 3). No bloquea Olas 4–6.

---

## 7. Progreso de ejecución (orquestador)

| Tarea | Estado | Nota |
|---|---|---|
| **T4.1** repro circuito (H-C) | ✅ **HECHA** | `package.json`+lock (circomlib 2.0.5 / circomlibjs 0.1.7 pin exacto), includes vía `-l node_modules`, requires JS a `circomlibjs`. **R1CS byte-idéntico** + VK sin cambio (`471397e9…`). Clon limpio (`git archive`→`npm ci`→`gen_proof.sh`) → `snarkjs verify OK`. checklist-Claude PASA. |
| **T4.2** 3er leg + h2/l2 | ⏸️ **DIFERIDO (por diseño)** | T4.2a codex = pospuesto (cuota); gate efectivo dual (Claude+Gemini) ya cumplido en Ola 3. T4.2b h2/l2 = se limpia en el re-spin de Ola 7 (no re-hornear VK solo por cosmética). |
| **T4.3** README honesto | ✅ **HECHA** | MSM precisa (`g1_mul`/`g1_add` loop), trusted-setup de juguete, `npm ci`+`npx snarkjs`; quita "hash-chain" stale, role-tag = REAL, orden temporal = Ola 7. `cargo build` verde. |
| **T4.4** git push | ✅ **HECHA** | Barrido de secretos OK (solo direcciones públicas); `git push origin main --tags`. |
</content>
</invoke>
