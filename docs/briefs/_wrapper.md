# Contexto común (anteponer a TODO brief)

Repo: `/home/manuel/proyectos/zk-terroir` — proyecto **ZK-Terroir**: procedencia justa de café
demostrable con **zero-knowledge** sin revelar la cadena de suministro. Stack: **Circom/Groth16
(BN254/bn128)** + **Soroban/Stellar** (soroban-sdk 25.1.0, verificación BN254 nativa P25/P26) +
**SEP-41** (TUSDC de test) para pagar el premium.

**Antes de tocar nada, lee:**
- `docs/PLAN-DIA-2.md` §2 (Decisiones A–I, congeladas por el arquitecto).
- `docs/AUDIT-LOG.md` (rondas 1–3: qué se auditó y cerró).
- `docs/DECISIONS.md` (D-001 curva BN254; D-002 modelo de confianza = membership Merkle, **NO EdDSA**).

**Reglas duras (no las violes):**
1. **Orden de señales públicas = Decisión A, CONGELADO:**
   `[r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash]`
   (7 señales → `VK.ic.len() == 8`). No lo reordenes ni lo cambies.
2. **No inventes APIs.** Si dudas de una API, deja `// TODO(audit): verificar contra <doc>`. Un hueco
   marcado vale más que una API inventada.
3. **Toolchain = Soroban, NO Odra/Casper.** Build: `stellar contract build`. Tests: `cargo test`
   (con `soroban_sdk::testutils`). Circuito: `snarkjs groth16 verify`. Ignora cualquier `cargo odra`.
4. **Shell = bash.**
5. **SERÁS AUDITADO** contra los criterios de aceptación al pie de la letra. Incluye tests negativos.
</content>
