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

## Estado del DAG tras auditoría
- **T1 → re-trabajo (GLM-5.2)** con la corrección de binding de arriba. **Decisión A NO cambia.**
- **T2 / T3-esqueleto:** pueden avanzar (públicos congelados intactos); la VK se hornea al cerrar T1.
- **T5:** cerrado; pendiente re-mint al contrato cuando T3 exista.
