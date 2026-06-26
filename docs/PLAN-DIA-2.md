# Plan Día 2 — Cadena de 3 eslabones + `claim_premium` paga USDC

**Esquema de trabajo:** Claude (este agente) **planifica y audita**; los **agentes de OpenCode
implementan**. Cada tarea trae: modelo asignado (ver `docs/internal/models-bench.md`), prompt
listo para pegar, criterios de aceptación, y la **compuerta de auditoría** que aplico antes de dar
una tarea por cerrada. Nada se da por hecho sin verificación reproducible.

> **Base ya probada (Día 1):** verificador Groth16 **BN254 genérico desplegado y verificado
> on-chain** (`CCK6X7HZ…SK6W`, tx `80acbe1b…`). Circuito de 1 eslabón (Merkle membership +
> nullifier, circomlib) verifica on-chain. Serializador snarkjs→BN254 (con swap G2 `c1‖c0`)
> funcionando. **No se reescribe cripto; se compone sobre esto.**

---

## 1. Meta del Día 2 (Definition of Done)

Una prueba real de **cadena de 3 eslabones** que, enviada a `claim_premium` en **testnet**:
1. verifica Groth16 BN254 on-chain,
2. exige `r_cert == raíz almacenada`,
3. rechaza `nullifier` repetido (anti doble-cobro, persistente),
4. **transfiere `premium_amount` en USDC (SEP-41)** a la wallet de la cooperativa,
5. registra `lot_commit` (para el QR del Día 3).

**Hito demostrable:** `claim_premium(proof)` → balance USDC de la coop sube exactamente
`premium_amount`; un segundo intento con el mismo nullifier **falla**; una prueba manipulada **falla**.

---

## 2. Decisiones pre-tomadas (para que los agentes NO improvisen)

Estas las fijo yo como arquitecto. Son los puntos donde un agente suele equivocarse:

| # | Decisión | Razón |
|---|---|---|
| A | **Public inputs, orden EXACTO** (circuito `public [...]` y contrato deben coincidir 1:1): `[r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash]` (7 señales → `IC.len()==8`). | El bug clásico de Groth16 es desalinear el orden señal↔parseo. Se congela aquí. |
| B | **Modelo de confianza = membership en `R_cert` por eslabón + hash-chain** (NO EdDSA; ver D-002). Cada eslabón aporta `leaf_i = Poseidon(certifier_pk_i, attest_data_i)`, se prueba `leaf_i ∈ R_cert` (Merkle Poseidon, prof. 10), y `chain_i = Poseidon(chain_{i-1}, leaf_i)` con `chain_0 = Poseidon(lot_id, season_id)`. | Reusa el patrón del eslabón 1 ya verificado; circomlib nativo BN254. |
| C | **`nullifier_hash = Poseidon(lot_secret, season_id)`** (privados: `lot_secret`, `season_id` opcionalmente público o derivado en `lot_commit`). `lot_commit = Poseidon(lot_id, season_id)` público. | Igual que el spec; un nullifier por lote+temporada. |
| D | **Range proof:** `price_paid ≥ floor_price` vía circomlib `GreaterEqThan(64)`; constreñir `price_paid, floor_price < 2^64` con `Num2Bits`. Montos en centavos. | Evita under-constrain; 64 bits sobra para precios. |
| E | **Binding de payout:** `payout_hi/payout_lo` = mitades de 16 bytes de la pubkey ed25519 (32B) como **public inputs** (cada mitad < campo BN254). El contrato recibe `payout: Address` como arg y **verifica** que sus 32 bytes partidos en hi/lo == los public inputs. **Fallback si la extracción de bytes de `Address` se complica:** admin registra la dirección de la coop en `init`/`set_payout` y el contrato paga a esa; el proof sólo aporta `payout_commit`. (MVP de 1 coop → fallback aceptable y honesto en README.) | Liga el destino del dinero a la prueba sin Poseidon on-chain. Riesgo de plomería → ver compuerta de auditoría T3. |
| F | **Verificación inline, VK horneada.** `claim_premium` **incorpora** la lógica de `verify_proof` (del spike) con la **VK del circuito de 3 eslabones como constante**, en vez de cross-contract call. | 1 contrato, 1 tx, más barato; reusa código ya auditado del spike. |
| G | **SEP-41 = escrow.** Se crea un asset de test (`TUSDC`, SAC en testnet), se **mintea al contrato** (escrow). `claim_premium` transfiere desde su propia balance (`from = current_contract_address`, auth automática del propio contrato). | Modelo simple y realista para el MVP; `payout` recibe USDC real de testnet. |
| H | **Storage:** `certifier_root` (instance), `nullifiers: Map`/entry **persistent** (replay debe sobrevivir), `lots: Map<lot_commit, timestamp>` persistent, `token: Address`, `admin: Address`. Bump de TTL en escrituras. | Si el nullifier vive en `temporary`, el replay-protection expira → vulnerabilidad. |
| I | **Orden checks-effects-interactions:** validar root → validar+insertar nullifier → registrar lot → **transferir USDC al final**. Si el transfer paniquea, revierte todo (atómico). | Anti-reentrada / consistencia. |

**Guardarraíl de alcance (MVP vs stretch):**
- **MVP (obligatorio hoy):** circuito 3 eslabones (membership+chain+range+nullifier), `claim_premium`
  (verify+root+nullifier+SEP-41 transfer), E2E testnet, test de doble-cobro y de prueba inválida.
- **Stretch (sólo si sobra tiempo):** `region_root`, doble membership (atestación∈subárbol del
  certificador ∧ certificador∈set), `lot_status` read endpoint, binding payout hi/lo "fuerte".

---

## 3. Routing de modelos (de `models-bench.md`, con promo x3 de MiniMax M3 activa)

| Tarea | Tipo | Modelo primario | Escalar a |
|---|---|---|---|
| T1 Circuito 3 eslabones | web3/cripto-crítico, math | **GLM-5.2** (effort max) | DeepSeek V4 Pro |
| T2 Infra JS (árbol R_cert, witness gen) | backend/Node | **MiniMax M3** ⚡ | DeepSeek V4 Pro |
| T3 Contrato `claim_premium` | web3-crítico (fondos), Rust | **GLM-5.2** | DeepSeek V4 Pro |
| T4 Tests (contrato + circuito) | tests | **MiniMax M3** ⚡ | MiniMax M2.7 (overflow) |
| T5 Setup token TUSDC/SAC + deploy | devops/CI | **MiniMax M3** ⚡ | GLM-5.2 |
| T6 Auditoría (dual, paralelo) | security/audit | **DeepSeek V4 Pro ∥ Kimi K2.6** | GLM-5.2 (desempate) |
| T7 Docs (README + comments) | docs | **MiniMax M3** ⚡ (1M ctx) | Kimi K2.6 (README público) |

**Notas de routing:**
- **Sin Qwen 3.7 Max** (malos resultados previos). GLM-5.2 es el cerebro (T1/T3); su escalación es V4 Pro.
- Con la promo x3, **M3 es el caballo de batalla** (infra, tests, devops, docs) — misma cuota/precio que M2.7 pero más calidad (bench §6.5). M2.7 sólo overflow.
- **Independencia de auditoría:** los auditores (V4 Pro, Kimi K2.6) **≠ el implementador** (GLM-5.2). GLM-5.2 sólo entra como desempate que yo dirijo, nunca a auto-auditar su propio código.
- Premium tocado sólo en T1/T3 (GLM-5.2) + desempate de T6. Ratio workers:premium sano.

---

## 4. DAG de ejecución (olas)

```
Ola 1 (paralelo):      T1 circuito ──┐     T5 token TUSDC ──┐     T3-esqueleto contrato (sin VK)
                                     │                      │
Ola 2:        T2 infra/witness ◄─────┘ (necesita señales)   │
              T3-final ◄── (hornea VK de T1) ◄──────────────┘
Ola 3:        T4 tests (necesita T1+T3) ──► E2E deploy+invoke testnet (T5 da el token)
Ola 4:        T6 auditoría dual (sobre T1+T3) + mi compuerta de auditoría final
```

Congelar **interfaz de señales (Decisión A)** ANTES de lanzar T2 y T3-final. Esa es la barrera real.

---

## 5. Especificación por tarea (con prompt para pegar en OpenCode)

### T1 — Circuito `terroir_chain.circom` (3 eslabones) · GLM-5.2
**Deliverable:** `circuits/terroir_chain.circom` + `verification_key.json` + prueba que verifica
off-chain (`snarkjs groth16 verify` = OK). Reusa `MerkleLevel`/`MerkleInclusion` de
`spike/link1/terroir_link.circom`.
**Aceptación:**
- `public [r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash]` (orden exacto, Decisión A).
- 3 memberships en `r_cert` + hash-chain (Decisión B) + range `price_paid ≥ floor_price` (D) + nullifier (C) + binding payout hi/lo (E).
- compila sin warnings de señales no usadas; `snarkjs groth16 verify` OK; al manipular cualquier input público la verificación falla.

> **Prompt:** "Implementa `circuits/terroir_chain.circom` (circom 2.1, curva bn128, circomlib).
> Extiende `spike/link1/terroir_link.circom`. Señales públicas EN ESTE ORDEN:
> `[r_cert, floor_price, lot_commit, premium_amount, payout_hi, payout_lo, nullifier_hash]`.
> Para i=1..3: `leaf_i=Poseidon(certifier_pk_i, attest_data_i)`, prueba `leaf_i ∈ r_cert`
> (Merkle Poseidon prof.10, reusa MerkleInclusion), `chain_i=Poseidon(chain_{i-1}, leaf_i)`,
> `chain_0=Poseidon(lot_id, season_id)`. `lot_commit=Poseidon(lot_id, season_id)` (público).
> `nullifier_hash=Poseidon(lot_secret, season_id)`. Range: `GreaterEqThan(64)` sobre
> `price_paid ≥ floor_price` con `Num2Bits` de 64 en ambos. `payout_hi/payout_lo` públicos
> ligados con constraint anti-malleabilidad (`h2<==payout_hi*payout_hi`, idem lo).
> NO uses EdDSA ni circomlib Poseidon nativo on-chain. Entrega circuito + script de setup groth16
> + prueba que pasa `snarkjs groth16 verify`."

### T2 — Infra JS: árbol `R_cert` + generador de witness · MiniMax M3
**Deliverable:** `circuits/js/buildTree.js` (construye `R_cert` con circomlibjs Poseidon, mismas
constantes que el circuito) + `circuits/js/genWitnessInput.js` (emite `input.json` válido para
T1) + un `r_cert.json` con la raíz para sembrar on-chain.
**Aceptación:** la raíz calculada en JS == la `r_cert` que el circuito acepta (witness válido);
`input.json` produce prueba que verifica off-chain.

> **Prompt:** "En `circuits/js/`, usando `circomlibjs` (buildPoseidon, BN254, MISMAS constantes que
> circomlib), construye el árbol Merkle de certificadores acreditados (`R_cert`, prof.10) y un
> generador de `input.json` para `terroir_chain.circom`: 3 hojas `leaf_i=Poseidon(pk_i, attest_i)`
> insertadas en el árbol con sus paths, hash-chain, `price_paid≥floor`, nullifier, payout hi/lo
> (parte una pubkey ed25519 de 32B en 2×16B). Exporta `r_cert.json` con la raíz. Verifica que el
> witness es aceptado por el .wasm del circuito."

### T3 — Contrato `claim_premium` · GLM-5.2
**Deliverable:** `contracts/terroir/src/lib.rs` con `init`, `set_certifier_root`, `claim_premium`,
`lot_status`. Reusa la lógica de pairing del spike (`spike/contract/src/lib.rs`), VK de 3 eslabones
horneada (Decisión F). Compila a wasm.
**Aceptación:** ver compuerta de auditoría T3 abajo. Tests verdes (T4). Despliega en testnet.

> **Prompt:** "Implementa el contrato Soroban `contracts/terroir` (soroban-sdk 25.1.0). Reusa la
> verificación Groth16 BN254 de `spike/contract/src/lib.rs` (módulo `crypto::bn254`,
> `pairing_check`), con la VK del circuito de 3 eslabones HORNEADA como constantes (de
> `verification_key.json`, serializadas al layout BN254 con el swap G2 c1‖c0 — ver
> `spike/circuit/serialize.js`). Funciones: `init(admin, token: Address)`;
> `set_certifier_root(admin, r_cert: BytesN<32>)` (require_auth admin);
> `claim_premium(proof: Proof, pub_signals: Vec<Fr>, payout: Address)` que: (1) exige
> `pub_signals[0]==certifier_root`; (2) reconstruye nullifier=`pub_signals[6]`, exige no-usado en
> Map PERSISTENTE, lo inserta; (3) verifica Groth16; (4) valida binding payout (Decisión E o
> fallback); (5) transfiere `premium_amount=pub_signals[3]` (i128) en SEP-41 desde
> `current_contract_address` a `payout`; (6) registra `lot_commit=pub_signals[2]` con timestamp.
> Orden checks-effects-interactions (transfer al final). `lot_status(lot_commit)->Option<u64>`.
> Storage persistente con bump de TTL."

### T4 — Tests · MiniMax M3 ⚡
**Deliverable:** tests unitarios del contrato (happy path, doble-cobro, root malo, proof malo,
amount 0) usando `soroban_sdk::testutils` + token mock; y un script E2E (`scripts/e2e.sh`) que
despliega, setea root, mintea TUSDC, invoca, y asserta el balance.
**Aceptación:** `cargo test` verde; E2E en testnet: balance de payout sube `premium_amount`; replay
falla; proof manipulada falla.

### T5 — Token TUSDC (SEP-41) + despliegues · MiniMax M3
**Deliverable:** `scripts/setup_token.sh` (crea asset test, deploy SAC, mintea al escrow) +
`scripts/deploy.sh`. Direcciones en `deployments/testnet.json`.
**Aceptación:** el contrato escrow tiene balance TUSDC > premium; `payout` puede recibir.

### T6 — Auditoría dual · DeepSeek V4 Pro ∥ Kimi K2.6
Corre AMBOS en paralelo sobre T1 (soundness del circuito) y T3 (seguridad del contrato), **diff de
hallazgos**. Auditores **independientes del implementador** (GLM-5.2). Entregan lista con severidad.
Yo sintetizo y decido; si empatan o discrepan en algo de fondos, desempata GLM-5.2 bajo mi criterio.

### T7 — Docs · MiniMax M3 ⚡ (README público → Kimi K2.6)
Actualiza `README.md` (sección "qué es real / qué es mock", flujo prueba→pago) y comentarios.
M3 (1M ctx) se traga repo + specs y razona el sistema. El **README público que leen los jueces**
se escala a **Kimi K2.6** para prosa pulida. NO toca lógica.

---

## 6. Mis compuertas de auditoría (lo aplico yo antes de cerrar cada tarea)

**Circuito (T1) — soundness:**
- [ ] Toda señal privada está **constreñida** (no hay grados de libertad para hacer trampa); `pathIndices` booleanos; `Num2Bits` cubre el rango sin overflow.
- [ ] Membership se exige en **los 3** eslabones (no sólo el primero); la hash-chain liga el orden y al `lot_id`.
- [ ] Orden de señales públicas == Decisión A (lo verifico contra `public.json`).
- [ ] Re-genero prueba y: verifica off-chain (true) **y on-chain** (con el verificador genérico ya desplegado, pasándole la nueva VK) → true; manipular **cada** público → false.
- [ ] Poseidon del circuito == Poseidon de `circomlibjs` (la raíz JS coincide con la que el circuito acepta).

**Contrato (T3) — seguridad de fondos:**
- [ ] checks-effects-interactions: nullifier insertado **antes** del transfer; transfer es la última acción.
- [ ] Nullifier en storage **persistente** + bump TTL (replay sobrevive); doble-cobro testeado → falla.
- [ ] `r_cert` comparado contra el almacenado; `set_certifier_root` con `require_auth(admin)`.
- [ ] La **VK horneada == `verification_key.json` del circuito desplegado** (comparo hash/bytes).
- [ ] El `Vec<Fr>` de públicos se arma en el MISMO orden que el circuito (Decisión A).
- [ ] `premium_amount` como `i128`, `> 0`, sin overflow; transfer maneja fallo (panic→revert atómico).
- [ ] Binding de payout verificado (o fallback documentado); un atacante no puede redirigir el premium.
- [ ] Tests negativos pasan: doble-cobro, root malo, proof malo, amount 0, payout no ligado.

**E2E (T4/T5):**
- [ ] En testnet: claim válido paga **exactamente** `premium_amount`; delta de balance verificado; replay falla; proof manipulada falla.

**Síntesis de auditoría (T6):** consolido hallazgos de los 2 auditores; un hallazgo de severidad
alta en manejo de fondos **bloquea** el cierre hasta corregir y re-auditar.

---

## 7. Qué te pediré como handoffs

1. Cuando un agente termine una tarea, me pasas su diff/branch → corro la compuerta de auditoría correspondiente y te devuelvo PASA / hallazgos.
2. Yo congelo la **interfaz de señales (Decisión A)** y la VK; si un agente la cambia, lo marco como regresión.
3. El cierre del Día 2 lo declaro yo sólo cuando el hito demostrable (sección 1) corre en testnet y pasa auditoría.
