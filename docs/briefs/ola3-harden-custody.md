# BRIEF Ola 3 (STRETCH) — Endurecer custodia: orden de roles finca→coop→tostador · TOCA FONDOS

> Antepón `docs/briefs/_wrapper.md`. **Este brief TOCA EL CIRCUITO SOUND y la VK horneada** ⇒ tras
> implementarlo es OBLIGATORIA la **auditoría triple** (Gemini 3.1 Pro High + GPT-5.5 + checklist-
> Claude) y **re-deploy + E2E** antes de dar por cerrado. El orquestador **NO** lo mergea sin eso.
> Diseño congelado por Claude (cerebro) + aprobado por el usuario (2026-07-01). No lo re-diseñes:
> impleméntalo tal cual; si una API no existe, deja `// TODO(audit)` en vez de inventar.

## Contexto: qué se cierra
El circuito T1 v3 (`circuits/terroir_chain.circom`) es SOUND pero dejó **H1 abierto** (AUDIT-LOG
ronda 1/3): la cadena de custodia con **orden** finca→coop→tostador NO está garantizada. Hoy las 3
hojas atan el mismo `lot_id` pero **no distinguen rol** — un lote podría probar 3 atestaciones
acreditadas cualesquiera del mismo `lot_id` sin que sean genuinamente {finca, coop, tostador}.
Esta Ola recupera esa propiedad **sin exponer datos nuevos y SIN romper Decisión A**.

## Diseño CONGELADO (impleméntalo exactamente así)

### Propiedad de seguridad objetivo
Toda prueba válida debe demostrar que **las tres hojas acreditadas ∈ `R_cert` corresponden a los tres
roles de custodia canónicos {FINCA, COOP, TOSTADOR}, cada rol exactamente una vez, sobre el MISMO
`lot_id`**. Un atacante NO puede: (a) sustituir un rol por otro (p.ej. hacer pasar una atestación de
finca por una de tostador), ni (b) omitir un rol (probar 2 finca + 1 coop), porque el **rol va
comprometido dentro del hash de la hoja acreditada** — cambiarlo rompe la membership en `R_cert`.

### Constantes de rol (en el circuito, no-cero para evitar ambigüedad)
`ROLE_FINCA = 1`, `ROLE_COOP = 2`, `ROLE_TOSTADOR = 3`. Fijas como literales en el circuito.

### Mapeo slot→rol (fijo por construcción → orden canónico)
- **slot 0 = COOP** (eslabón económico; conserva `price_paid`, `season_id`, `lot_secret`).
- **slot 1 = FINCA**.
- **slot 2 = TOSTADOR**.

Como los roles son **literales del circuito** en cada slot, el orden queda canónico por construcción:
`R_cert` debe contener una hoja de coop acreditada, una de finca y una de tostador, todas del mismo
`lot_id`. (No hace falta hash-chain ni señal pública nueva.)

### Cambios EXACTOS en `circuits/terroir_chain.circom`
1. `leaf_0` (coop) pasa de `Poseidon(5)` a **`Poseidon(6)`** con el rol insertado:
   `leaf_0 = Poseidon(certifier_pk_0, ROLE_COOP, lot_id, season_id, price_paid, lot_secret)`.
2. `leaf_1`, `leaf_2` pasan de `Poseidon(3)` a **`Poseidon(4)`** con su rol:
   `leaf_1 = Poseidon(certifier_pk_1, ROLE_FINCA,     lot_id, attest_data_0)`
   `leaf_2 = Poseidon(certifier_pk_2, ROLE_TOSTADOR,  lot_id, attest_data_1)`
3. **NO toques** nada más de la lógica sound: `lot_commit = Poseidon(lot_id, season_id)`,
   `nullifier_hash = Poseidon(lot_secret, season_id)`, range `price_paid ≥ floor_price`
   (`GreaterEqThan(64)` + `Num2Bits`), `premium_amount === price_paid - floor_price`, binding payout
   hi/lo (rangos 128b + `h2`/`l2`), y los 3 checks de distinción de `certifier_pk`
   (pk0≠pk1, pk0≠pk1... los tres pares) se **conservan tal cual**.
4. **`component main` NO cambia**: mismas 7 señales públicas en el MISMO orden (Decisión A):
   `[r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash]`.
   `TerroirChain(10)` (levels=10) sin cambios.

### Cambios en la infra JS (`circuits/js/`)
- `buildTree.js`: al construir `R_cert`, **etiqueta cada hoja acreditada con su rol** usando las
  MISMAS constantes y el MISMO layout Poseidon que el circuito (coop=`Poseidon(pk,ROLE_COOP,lot_id,
  season_id,price_paid,lot_secret)`; finca/tostador=`Poseidon(pk,ROLE_*,lot_id,attest_data)`).
- `genWitnessInput.js` / `gen_input.js`: emitir el `input.json` coherente con las hojas role-tagged.
- Regenerar `r_cert.json`, `tree.json`, `input.json`, la prueba (`proof.json`/`public.json`) y la
  **`verification_key.json`** (nuevo setup groth16). La raíz `r_cert` (public.json[0]) CAMBIARÁ.

### Re-hornear la VK en el contrato (`contracts/terroir/src/lib.rs`)
- Serializa la nueva `verification_key.json` con **`circuits/serialize.js`** (G1 = be32(x)‖be32(y);
  G2 con **swap `c1‖c0`**, layout EIP-197) y reemplaza las constantes `VK_ALPHA/BETA/GAMMA/DELTA/
  IC0..IC7`. **`ic.len()` sigue == 8** (7 públicos + 1) — si cambia, algo rompió Decisión A: PARA.
- **NO cambies** la interfaz del contrato: `claim_premium`, el parseo de índices 0..6, el orden CEI,
  el floor pin, el nullifier persistente, el payout binding — todo intacto. Solo cambian los bytes VK.
- ⚠️ **VK vieja + circuito nuevo = TODAS las pruebas fallan.** Re-hornear y regenerar prueba van juntos.

### Actualizar tests del contrato (`contracts/terroir/src/test.rs`)
- Los `*_real` usan `circuits/proof.json`/`public.json`: **regenera** las constantes hex embebidas
  (`PUB_*`, `PROOF_*`) desde los nuevos artefactos. `test_happy_path_real` debe pagar; los negativos
  (`bad_root/bad_floor/amount_zero/bad_proof/double_spend/payout_binding`) deben seguir en verde por
  la razón correcta.

## Reglas duras (STOP → usuario si chocas con alguna)
- **NO añadas ni reordenes señales públicas** (Decisión A). Si crees que el orden real de custodia
  (secuencia temporal literal) exige exponer algo nuevo → **PARA**: eso sería otra decisión (rompe A
  y toca el contrato), no esta Ola. Aquí solo se enforce **cobertura+etiquetado de roles** (que ya
  recupera el sustrato de H1: no-sustitución y no-omisión de rol). Documenta esa frontera en README.
- **NO cambies** `lot_commit` ni `nullifier_hash` (el verificador público `verify/` y el anti-replay
  dependen de su semántica; cambiarlos rompería la página QR desplegada).
- No inventes APIs de circomlib/soroban-sdk; marca `// TODO(audit)`.

## Criterios de aceptación (los verifica el orquestador, no tu self-report)
1. `snarkjs groth16 verify verification_key.json public.json proof.json` → **OK**.
2. `component main` mantiene EXACTAMENTE las 7 señales de Decisión A; `nPublic == 7`; `IC.len() == 8`.
3. **Test adversarial de rol** (añádelo, estilo `double_spend_attack.js`): construir un witness con
   una hoja cuyo rol NO coincide con el slot (p.ej. finca en el slot de tostador, o 2 finca) →
   el witness es **RECHAZADO** (falla la membership en `R_cert`). El control (roles correctos) → OK.
4. `cd contracts/terroir && cargo test` → verde (incl. `*_real` regenerados) `&& stellar contract build`
   `&& cargo clippy --all-targets`.
5. Re-deploy a Testnet + `init`/`set_certifier_root(nueva r_cert)`/`set_floor` + **E2E**: happy paga
   `premium_amount`; replay (mismo nullifier) falla; prueba manipulada falla. Actualiza
   `deployments/testnet.json` (nuevo `terroir_contract`, tx, `lot_status_registered`).
6. El verificador `verify/verify.sh` sigue funcionando contra el NUEVO contrato (lee testnet.json).

## Checklist de auditoría (para la TRIPLE de cierre — reporta OK/PROBLEMA archivo:línea)
1. **Rol comprometido de verdad:** cada `leaf_i` incluye la constante de rol correcta en su preimagen
   Poseidon (slot0=COOP, slot1=FINCA, slot2=TOSTADOR); la aridad de Poseidon coincide (6/4/4).
2. **No-sustitución / no-omisión:** ¿es imposible probar con un rol repetido u omitido? (la membership
   role-tagged en `R_cert` lo impide). Piénsalo con saña: ¿algún grado de libertad para falsear rol?
3. **Decisión A intacta:** 7 señales, orden congelado, `ic.len()==8`; el contrato NO cambió parseo.
4. **Soundness previa conservada:** H2 (price_paid atado en leaf_0), H3 (season_id atado ⇒ nullifier
   no reciclable), range, premium binding, payout binding, distinción de pk — TODOS siguen cerrados.
5. **VK == circuito nuevo:** recomputar la serialización de `verification_key.json` (G1 `x‖y`, G2 swap
   `c1‖c0`) y comparar byte-a-byte con `VK_*` en lib.rs. `ic.len()==8`.
6. **Infra coherente:** la raíz que produce `buildTree.js` (JS) == la `r_cert` que el circuito acepta
   (witness válido) — Poseidon JS↔circuito consistente con los roles.
7. **E2E on-chain real:** no solo tests; el nodo paga el premium y bloquea replay/tamper.
8. **Sin regresión de fondos:** ningún camino nuevo de drenaje/doble-cobro/redirección introducido
   por el cambio de aridad de las hojas.

## Salida obligatoria (del implementador y de cada auditor)
Implementador: resumen de cambios + supuestos + `// TODO(audit)` pendientes, y `VEREDICTO: LISTO — <1 línea>`.
Cada auditor: por punto `OK`/`PROBLEMA (archivo:línea)`, y termina con
`VEREDICTO: PASA` **o** `VEREDICTO: NO-PASA — <razón en una línea>`.

## Gate de merge (runbook §6)
Triple audit converge en PASA **y** E2E verde en Testnet → commit + (opcional) tag. Cualquier hallazgo
ALTA de fondos, o un auditor en NO-PASA con hallazgo real, o auditores en desacuerdo → **STOP → usuario**.
Re-hornear VK sin re-auditar triple + re-deploy = prohibido.
