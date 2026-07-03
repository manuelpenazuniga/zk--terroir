# BRIEF Ola 7 (STRETCH) — Doble-membership ASP de 2 niveles · TOCA FONDOS

> ⚠️ **APROBADO por el usuario (2026-07-03). Diseño congelado — implementar en worktree spike/w7.**
> Redactado por Claude (cerebro) el 2026-07-03 como paso de diseño de la Ola 7. Antes de ejecutar:
> (1) el usuario aprueba este diseño; (2) se resuelve el gate de auditoría (el plan pide TRIPLE incl.
> **codex/GPT-5.5**, hoy pospuesto — decisión del usuario). Este brief TOCA EL CIRCUITO SOUND y la VK
> horneada ⇒ tras implementarlo es OBLIGATORIA la auditoría del gate acordado + **re-deploy + E2E**.
> Antepón `docs/briefs/_wrapper.md`.

## Contexto: qué recupera
El spec (`zk-terroir.md §3.1`) define el patrón **ASP**: `R_cert` es un árbol de Merkle cuyas **hojas
son las claves públicas de los certificadores acreditados**. El circuito actual (Ola 3) lo tiene
**aplanado**: cada hoja de `R_cert` = `Poseidon(pk_i, ROLE_i, lot_id, …attest)` — mezcla identidad del
certificador con la atestación concreta. Eso obliga a la autoridad a sembrar en `R_cert` cada par
(certificador, atestación) individualmente. La Ola 7 separa los dos niveles = el ASP real: un
certificador acreditado emite atestaciones bajo **su propio subárbol**, sin re-sellar `R_cert`.

**No es que 1 nivel sea inseguro; es que 2 niveles es la arquitectura ASP correcta del spec** y escala
a acreditación real. **Sin señales públicas nuevas** ⇒ Decisión A intacta ⇒ el contrato solo re-hornea VK.

## Diseño CONGELADO (impleméntalo exactamente así)

### Dos árboles, una sola raíz pública
1. **Subárbol de atestaciones (privado, por certificador i):** hojas = atestaciones que ese certificador
   emitió. Para este lote, cada certificador aporta UNA hoja de atestación:
   - slot 0 (COOP):     `attest_0 = Poseidon(lot_id, season_id, price_paid, lot_secret)`  (Poseidon 4)
   - slot 1 (FINCA):    `attest_1 = Poseidon(lot_id, attest_data_0)`                       (Poseidon 2)
   - slot 2 (TOSTADOR): `attest_2 = Poseidon(lot_id, attest_data_1)`                       (Poseidon 2)
   Raíz del subárbol de i = `R_attest_i` (privada). Profundidad `LEVELS_ATTEST = 10` (igual que R_cert,
   para reusar `MerkleInclusion`; se puede bajar a 4 en un refinamiento posterior — NO en esta ola).
2. **`R_cert` (set acreditado, raíz PÚBLICA — la única):** hojas = `Poseidon(pk_i, ROLE_i, R_attest_i)`
   (Poseidon 3). `ROLE_COOP=2 / ROLE_FINCA=1 / ROLE_TOSTADOR=3` (literales, como Ola 3). Profundidad `LEVELS=10`.

### El circuito prueba, por eslabón i (i=0,1,2)
1. **L2 (atestación):** `MerkleInclusion(attest_i, R_attest_i, pathAttest_i)` → el certificador emitió esta atestación.
2. **L1 (acreditación):** `cert_leaf_i = Poseidon(pk_i, ROLE_i, R_attest_i)`; `MerkleInclusion(cert_leaf_i, r_cert, pathCert_i)` → el certificador está acreditado.

`R_attest_i` es **input privado** (no se recomputa el subárbol entero en el circuito; se prueba la
inclusión de la hoja hacia `R_attest_i` y luego `R_attest_i` entra en la preimagen de `cert_leaf_i`).

### Inputs privados nuevos
`R_attest[3]`, `pathAttestElements[3][LEVELS_ATTEST]`, `pathAttestIndices[3][LEVELS_ATTEST]`,
`pathCertElements[3][LEVELS]`, `pathCertIndices[3][LEVELS]`. `certifier_pk[3]`, `attest_data[2]`,
`lot_id/season_id/price_paid/lot_secret` como hoy.

### Lo que NO cambia (consérvalo tal cual)
- `component main` = **mismas 7 señales públicas, mismo orden (Decisión A)**; `IC.len()==8`.
- `lot_commit = Poseidon(lot_id, season_id)`; `nullifier_hash = Poseidon(lot_secret, season_id)`.
- Range `price_paid ≥ floor_price` (`GreaterEqThan(64)`+`Num2Bits`), `premium===price_paid-floor_price`,
  binding payout hi/lo (`Num2Bits(128)`+`h2`/`l2`), y los 3 checks de distinción de `certifier_pk`.
- **Bindings económicos preservados:** `price_paid`, `season_id`, `lot_secret` viven en la preimagen de
  `attest_0` (COOP) ⇒ no se pueden variar sin romper L2 ⇒ nullifier y premium siguen sound.

### Infra JS (`circuits/gen_input.js` VIVO + espejo `circuits/js/`)
- Construir AMBOS árboles: por certificador, un subárbol con su hoja de atestación → `R_attest_i`;
  luego `R_cert` con hojas `Poseidon(pk_i, ROLE_i, R_attest_i)`. Emitir los DOS sets de paths.
- Regenerar `input.json`, prueba, `verification_key.json`. `r_cert` (public.json[0]) CAMBIARÁ.
- **Preservar el nuevo zkey** en `web/public/` (como Ola 5 — el frontend lo necesita; setup no reproducible).

### Re-hornear VK + contrato (`contracts/terroir/src/lib.rs`)
- Serializar la nueva `verification_key.json` con `circuits/serialize.js`; reemplazar `VK_*`.
  `IC.len()` sigue **== 8** (si cambia → rompiste Decisión A → PARA). **Interfaz del contrato intacta.**
- Regenerar tests `*_real` (`PUB_*`, `PROOF_*`) desde los nuevos artefactos.

## Criterios de aceptación (los verifica el orquestador, no el self-report)
1. `snarkjs groth16 verify` → OK. `nPublic==7`, `IC.len()==8`, orden Decisión A.
2. `cargo test` verde (11/11 `*_real` regenerados) + `stellar contract build` + `clippy`.
3. **Test adversarial de 2 niveles** (nuevo): (a) atestación que NO está en el subárbol del certificador
   acreditado → RECHAZO (falla L2); (b) certificador NO acreditado (su `cert_leaf ∉ R_cert`) aunque la
   atestación sea válida → RECHAZO (falla L1); (c) mismatch de `R_attest` entre L1 y L2 → RECHAZO. Control OK.
4. VK recomputada byte-a-byte == `lib.rs`; R1CS nuevo documentado (constraints ↑ por las 6 inclusiones).
5. Re-deploy Testnet + E2E (happy paga / replay falla / tamper falla); actualizar `deployments/testnet.json`.
6. `verify/verify.sh` y el frontend `web/` siguen funcionando contra el nuevo contrato/VK/zkey.
7. **(oportunista)** limpiar `h2`/`l2` muertas (H-D) en este mismo re-hornear.

## Reglas duras (STOP → usuario)
- No añadir ni reordenar señales públicas (Decisión A). Si el diseño exige un público nuevo → PARA.
- No cambiar `lot_commit`/`nullifier`/aritmética de premium.
- Cualquier hallazgo ALTA de fondos → STOP, aunque un auditor diga PASA.

## Checklist de auditoría (reporta OK/PROBLEMA archivo:línea)
1. **2 niveles reales:** L2 (atestación∈subárbol) y L1 (cert_leaf∈R_cert) ambos enforced por eslabón;
   `R_attest_i` es EL MISMO valor en la inclusión L2 y en la preimagen de `cert_leaf_i` (no dos libres).
2. **No-forja:** ¿imposible probar con atestación fuera del subárbol, o certificador no acreditado, o
   `R_attest` inconsistente entre niveles? Piénsalo con saña.
3. **Decisión A intacta:** 7 señales, orden, `IC.len()==8`; contrato sin re-parseo.
4. **Soundness previa conservada:** nullifier/premium/range/payout/distinción pk — todos cerrados; los
   bindings económicos siguen en la hoja de atestación de la coop.
5. **VK == circuito nuevo** (byte-a-byte) e `IC.len()==8`.
6. **Infra coherente:** raíz JS (ambos árboles) == la que acepta el circuito.
7. **E2E on-chain real** + frontend/verify.sh siguen verdes.
8. **Sin regresión de fondos** por el aumento de inclusiones.

## Gate de merge (DECISIÓN DEL USUARIO PENDIENTE)
El plan pide **TRIPLE incl. codex**. Con codex pospuesto, el usuario elige: (a) implementar+dual+HOLD
merge hasta codex; (b) dual+adversarial-Claude y merge (más riesgo); (c) esperar codex. **Re-hornear VK
sin el gate acordado + re-deploy = prohibido.**

## Salida obligatoria (implementador y auditores)
Implementador: resumen + supuestos + `// TODO(audit)`, y `VEREDICTO: LISTO — <1 línea>`.
Auditor: por punto OK/PROBLEMA, y `VEREDICTO: PASA` / `NO-PASA — <razón>`.
