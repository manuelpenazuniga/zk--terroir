# Audit log — Día 2

Auditor: Claude (planifica+audita). Método: leer el código real + reproducir on-chain, no confiar en el self-report del agente.

---

## T1 — Circuito `terroir_chain.circom` · GLM-5.2 · **❌ NO PASA (bloqueado)**

**Reproducción mecánica (todo OK):** `snarkjs groth16 verify` → OK; serializado y verificado
**on-chain** con el verificador genérico (`CCK6X7HZ…`) → `true` (7 públicos, IC=8). Orden público ==
Decisión A ✅. Rangos `Num2Bits` ✅. Poseidon JS↔circuito consistente ✅. `pathIndices` booleanos ✅.

**Pero 3 hallazgos de soundness que el tamper-test NO detecta** (el tamper sólo prueba
no-maleabilidad del vector público; NO prueba que un prover honesto-pero-malicioso pueda **generar
una prueba nueva** con valores elegidos):

### H1 (ALTA — claim de producto roto) · hash-chain muerta
`chain[3]` (cabeza de la cadena de custodia) se calcula (líneas 147-151) pero **nunca se
constriñe ni se expone**. La cadena queda anclada al inicio (`chain[0]===lot_commit`) pero su salida
flota libre → **no impone ninguna restricción**. La "integridad de custodia encadenada" del pitch
**no está garantizada**: reordenar/cambiar eslabones no viola ninguna constraint observable.

### H2 (ALTA — drena el escrow) · premium/precio sin respaldo
`price_paid` es input privado **libre** (sólo `Num2Bits(64)` + `≥ floor`), y `premium_amount =
price_paid - floor_price` (líneas 101-106). `price_paid` **NO está comprometido en ninguna hoja
acreditada**. → Un prover con 3 hojas válidas pone `price_paid = floor + ENORME` y se auto-paga un
premium arbitrario (hasta ~2⁶⁴ centavos). Con el escrow en i128::MAX (T5), drena lo que quiera.

### H3 (ALTA — doble cobro / claims repetidos) · lot/secret sin atar
`lot_id`, `lot_secret`, `season_id` son privados **libres**; `nullifier_hash =
Poseidon(lot_secret, season_id)` (89-93). Nada ata `lot_secret`/`lot_id` a las hojas acreditadas. →
El prover elige un `lot_secret` fresco en cada claim → nullifier distinto cada vez → el Map
anti-doble-cobro **nunca coincide** → reclama el MISMO lote infinitas veces.

**Causa raíz (única):** el circuito prueba dos cosas **desconectadas**: (a) "conozco 3 hojas ∈
R_cert" y (b) "aritmética sobre privados libres (precio, secreto, lote)". No hay vínculo
criptográfico entre (a) y (b). Es justo el insight de privacy-pools que se perdió: **los secretos
económicos/del lote deben ir COMMITTED dentro de las hojas que se prueban ∈ R_cert.**

**Corrección requerida (mantiene Decisión A / públicos intactos; sólo cambian internos + VK):**
1. **Atar economía y lote a la atestación de la cooperativa (eslabón 0):**
   `leaf_0 = Poseidon(certifier_pk_0, lot_id, price_paid, lot_secret)` → probar `leaf_0 ∈ R_cert`
   fija `price_paid`, `lot_id`, `lot_secret` a una atestación acreditada (mata H2 y H3).
2. **Atar todos los eslabones al MISMO lote:** incluir `lot_id` en `leaf_1`, `leaf_2`
   (`leaf_i = Poseidon(certifier_pk_i, lot_id, attest_data_i)`).
3. **Custodia (H1):** o se elimina la cadena muerta y la propiedad pasa a ser "los 3 eslabones
   atestan el mismo `lot_id`", o se **consume `chain[3]`** (p.ej. exponerlo / atarlo a un público).
   Para el MVP, (2) ya da la propiedad de custodia mínima → aceptable documentarlo.
4. `premium_amount = price_paid - floor_price` se mantiene, pero ahora `price_paid` está respaldado.

> Crédito al agente: el trabajo mecánico (compilación, orden, rangos, Poseidon, payout hi/lo,
> pipeline real on-chain) es correcto y se conserva. El fallo es de **binding semántico**, no de plomería.

---

## T5 — Token TUSDC (SEP-41) · MiniMax M3 · **✅ PASA (con notas menores)**

**Reproducción on-chain:** SAC `CAQJK77D…W6P6` existe; balance del escrow (terroir `G…JJ5J`) =
`9223372036854775807` ✅. Sin secretos hardcodeados en `scripts/` ✅. `deployments/testnet.json`
sólo direcciones públicas ✅. Scripts idempotentes (declarado).

**Notas (no bloqueantes):**
- Escrow == issuer == admin == `terroir`. Cuando exista T3, **re-mintear al contrato real**
  (`ESCROW_ADDRESS=<CONTRACT_ID> ./setup_token.sh`). Idealmente issuer ≠ escrow.
- Mint = i128::MAX es un *smell* (saturación). Para el demo, un monto sano (p.ej. 1e7 USDC) basta.

---

## Estado del DAG tras auditoría (ronda 1)
- **T1 → re-trabajo (GLM-5.2)** con la corrección de binding de arriba. **Decisión A NO cambia.**
- **T2 / T3-esqueleto:** pueden avanzar (públicos congelados intactos); la VK se hornea al cerrar T1.
- **T5:** cerrado; pendiente re-mint al contrato cuando T3 exista.

---

# Ronda 2 — T1 v2 / T2 / T3-esqueleto

## T2 — Infra JS (buildTree/genWitnessInput) · MiniMax M3 · **✅ PASA**
Reproducido: `r_cert.json` (10148290…615317) == `public.json[0]` ✅ (Poseidon JS↔circuito consistente).
`genWitnessInput` produce el `input.json` que genera una prueba que verifica off-chain y **on-chain
`true`**. Hojas nuevas (leaf_0 de 4 entradas) coherentes con el circuito v2. Sin regresión.

## T1 v2 — Circuito · GLM-5.2 · **❌ NO PASA (1 residual ALTA)**
Reproducido: off-chain OK, on-chain `true`, orden público intacto.
- **H2 (premium arbitrario) → ✅ CERRADO:** `leaf_0 = Poseidon(pk_0, lot_id, price_paid, lot_secret) ∈
  R_cert` ata `price_paid` a una atestación; `premium === price_paid - floor_price`. Ya no es libre.
- **H1 (cadena) → ✅ ACEPTADO con nota:** cadena eliminada; `lot_id` en cada hoja ata los 3 al mismo
  lote. Se pierde el **orden** finca→coop→tostador. Aceptable para MVP; **documentar en README** que
  el orden de custodia es stretch Día 3.
- **H3 (doble cobro) → ❌ RESIDUAL (ALTA, sigue drenando):** `lot_secret` quedó atado, **pero
  `season_id` sigue siendo input privado LIBRE** (`terroir_chain.circom:83`, sólo se usa en
  `lot_commit` y `nullifier_hash`, no va en ninguna hoja). `nullifier_hash = Poseidon(lot_secret,
  season_id)` → el atacante varía `season_id` → nullifier fresco en cada claim → **reclama el mismo
  lote infinitas veces**. **Fix:** meter `season_id` en `leaf_0`
  (`Poseidon(pk_0, lot_id, season_id, price_paid, lot_secret)`) para que la atestación fije la temporada.
- LOW: `leaf_1`/`leaf_2` sin constraint de distinción → un atacante podría usar el mismo certificador
  acreditado para ambos (≥2 miembros distintos en vez de 3). Documentar o añadir `pk_1 != pk_2`.

## T3-esqueleto — Contrato · GLM-5.2 · **✅ PASA como esqueleto (2 fixes obligatorios antes de T3-final)**
Reproducido: `stellar contract build` → wasm 8883 B; `cargo test` → 6 pasan + 1 ignored.
- Decisiones A/F/G/H/I aplicadas correctamente. **CEI correcto** (transfer al final; nullifier se
  inserta DESPUÉS de verify). Nullifier en storage **persistente** + TTL ✅. Overflow i128 ✅.
- **Binding de payout (Decisión E "strong") → ✅ VERIFICADO REAL:** usa `addr.to_payload()` +
  `AddressPayload::{AccountIdPublicKeyEd25519,ContractIdHash}` (feature `hazmat-address`) — **API real
  del SDK 25.1.0** (confirmado en `address_payload.rs`). Layout hi/lo (hi=addr[0..16], lo=addr[16..32]
  en los 16 bytes bajos de cada Fr) **coincide** entre `gen_input.js`, `check_payout_binding` y el test.
- **Bypass `#[cfg(test)]` en `verify()` (lib.rs:237-249) → ✅ BIEN GATEADO:** `cfg(not(test))` usa la
  verificación real en el wasm; el bypass NO entra al binario (confirmado: build real compila el path
  real). **Pero:** bajo `cfg(test)` `verify()` SIEMPRE devuelve `true`, así que el happy-path **no
  ejercita cripto**. → en T3-final hay que **eliminar el bypass** y usar prueba+VK reales, o la
  verificación queda sin cobertura automática.
- **❌ HALLAZGO NUEVO (ALTA, inflación de premium): `floor_price` NO se valida.** `claim_premium`
  hace `let _floor_price = pub_signals.get(1)` (lib.rs:119) y **nunca lo compara contra un piso
  almacenado**. Como en el circuito `premium = price_paid - floor_price` y `floor_price` es un público
  que **provee el prover**, éste pone `floor_price` bajo → infla `premium_amount` hasta el `price_paid`
  atestado completo, y el contrato paga ese monto. **Fix:** almacenar el piso (`set_floor(admin,floor)`)
  y exigir `pub_signals[1] == floor_almacenado` en `claim_premium`.

### Estado DAG (ronda 2)
- **T1 → v3 (GLM-5.2):** atar `season_id` en `leaf_0` (+ opcional `pk_1!=pk_2`). Regenerar prueba/VK. Re-audito.
- **T3-final** debe además: (a) pin de `floor_price`; (b) quitar el bypass `cfg(test)`. (Ya dependía de la VK de T1.)
- **T2 ✅, payout binding ✅** — no se tocan.

---

# Ronda 3 — T1 v3 / T3 floor fix

## T1 v3 — Circuito · GLM-5.2 · **✅ PASA**
Reproducido: off-chain OK, on-chain `true`, Decisión A intacta (IC=8).
- **H3 (doble cobro) → ✅ CERRADO Y PROBADO ADVERSARIALMENTE.** `season_id` ahora va dentro de
  `leaf_0 = Poseidon(pk_0, lot_id, season_id, price_paid, lot_secret)` (`terroir_chain.circom:168-173`)
  y es el MISMO signal usado en `nullifier_hash`. **Ataque ejecutado** (`circuits/double_spend_attack.js`):
  season'=season+1, recomputando `lot_commit'`/`nullifier'` pero manteniendo `r_cert` y los paths →
  el witness es **RECHAZADO**: `Assert Failed @ MerkleInclusion line 47 (root===cur)` vía `inc0`
  (`TerroirChain line 180`). El control (input original) genera witness OK (exit 0). → no se puede
  reciclar el nullifier variando la temporada.
- **LOW (distinción de certificadores) → ✅ cerrado:** 3 `IsEqual().out===0` (pk0≠pk1, pk0≠pk2,
  pk1≠pk2). *Nit:* el comentario `:135-136` dice que pk0 no se chequea, pero el código SÍ lo chequea
  (más estricto). Corregir comentario (no funcional).
- Recordatorio H1: cadena de orden sigue fuera (MVP) → documentar en README.

**Veredicto: el circuito de 3 eslabones es SOUND.** H1 (aceptado/documentado), H2 (cerrado), H3 (cerrado+probado).

## T3 floor fix — Contrato · GLM-5.2 · **✅ PASA**
Reproducido: build wasm 9673 B, 5 funciones, `cargo test` → 10 pasan + 1 ignored.
- `set_floor(admin, i128)` admin-only + rechaza negativos; `claim_premium` exige
  `pub_signals[1]==floor_almacenado` (`lib.rs:163-175`), orden root→floor→amount→nullifier→crypto→payout→transfer (CEI ok).
- **Inflación de premium → ✅ CERRADA en conjunto:** floor pinned (contrato) + `price_paid` atado en
  `leaf_0` (T1 v3) → `premium = price_paid − floor` con ambos extremos fijados. *Nit:* el comentario
  `lib.rs:165-168` dice que H2 sigue abierto; está **desactualizado** (T1 v2/v3 ya ató `price_paid`).
  Actualizar comentario (no funcional).

### Estado DAG (ronda 3) — circuito y lógica de fondos SOUND
- **T1 v3 ✅, T2 ✅, T3 esqueleto+floor ✅, T5 ✅.**
- **Desbloqueado: T3-final** (§8.2) — hornear VK de T1 v3, **quitar bypass `cfg(test)`**, deploy,
  + T5 re-mint al contrato + trustline del payout, y **E2E en testnet** (hito Definition of Done §1).
- Cleanups menores (no bloqueantes): 2 comentarios desactualizados (circuito :135, contrato :165) → T7/docs.

---

# Ronda 4 — T3-final · **⏳ PENDIENTE DE AUDITORÍA DE CIERRE (no confundir con PASA)**

**Situación (2026-07-01):** el T3-final está **implementado en el working tree pero SIN COMMITEAR**
(último commit sigue siendo `be996a2`). Reproducción mecánica preliminar de lo observado (aún **no**
es el veredicto del panel):
- `contracts/terroir/src/lib.rs`: VK real horneada (`VK_ALPHA…VK_IC7`), `verify()` (~274) llama al
  `groth16_verify` real **sin** bypass `#[cfg(test)]` (comentario y código coinciden), floor binding
  presente, CEI intacto.
- `contracts/terroir/src/test.rs`: suite con **prueba real** (`test_happy_path_real`,
  `test_double_spend_real`, `test_bad_root_real`, `test_bad_floor_real`, `test_amount_zero_real`,
  `bad_proof`, `payout_binding`) — snapshots `*_real.1.json` nuevos en el working tree.
- `deployments/testnet.json`: contrato `CBHFN7QU…TJQQ`, `e2e.happy_path=pass`, `replay_blocked=pass`,
  `tampered_proof_blocked=pass`, `lot_status_registered=1782661915`, tx registradas.

**⚠️ Gate faltante = Ola 0 de `docs/PLAN-DIA-3.md`:** falta el **pase adversarial de cierre**
(triple: Gemini 3.1 Pro High + **GPT-5.5** + checklist) sobre este diff antes del commit. El brief está
en `docs/briefs/ola0-close-dia2.md`. **No** se declara Día 2 cerrado hasta que el panel converja en
`VEREDICTO: PASA` y se haga `git commit` + `git tag dia2-cerrado`. Cualquier hallazgo ALTA de fondos
(drenaje / doble-cobro / redirigir premium / bypass de root-floor-nullifier) → STOP → usuario.
