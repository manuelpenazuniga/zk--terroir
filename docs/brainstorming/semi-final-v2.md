# Semi-final v2 — Torneo unificado de ideas + head-to-head MedBridge vs GenoMatch

**Modelo:** Opus 4.8 · **Fecha:** 2026-06-19 · Compite TODO: las 20 ideas de `brainstorming-opus-4.8.md` + las 15 de `brainstorming-gemini-3.5-flash.md` + tu idea **MedBridge**. Objetivo: ser justo.

---

## 0. Cómo hago esto justo (metodología)

1. **Una sola rúbrica para todas** (6 criterios, /30). La de Gemini era de 4 criterios (/20) y **no medía "ZK load-bearing" ni "diferenciación"** — dos cosas que deciden un hackathon. Re-puntúo sus ideas con la rúbrica completa para que compitan en igualdad.
2. **Fusiono duplicados.** Cold-chain, credenciales académicas, genómica y fair-trade aparecen en *ambos* documentos. Esa convergencia independiente es una señal fuerte, no un empate que romper. Marco la fuente de cada candidato: **[O]** Opus, **[G]** Gemini, **[U]** tú, **[O+G]** convergencia.
3. **Aplico la misma corrección técnica a todos.** El hallazgo validado "RISC Zero no prueba en navegador" (ver `brainstorming-opus-4.8.md` §7) penaliza por igual a mi viejo encuadre de GenoMatch **y** a las ideas de Gemini que "parsean un genoma/DNA client-side con RISC Zero" (DNA-Privacy, EcoSeed). Justicia = misma vara.
4. **Crédito honesto.** No inflo lo mío ni minimizo lo de Gemini. De hecho, una idea de Gemini termina por encima de tu MedBridge en la rúbrica, y lo dejo así.

### Rúbrica (1–5 cada uno, /30)
**ZK-LB** (¿se rompe sin privacidad? ¿es obligatoria?) · **Feas** (factibilidad real a 10 días, con la corrección de proving) · **$STR** (¿la prueba mueve dinero/acción real en Stellar?) · **Dif** (diferenciación vs la pila de privacy-pools) · **Demo** (historia de una frase + flujo visible en 2 min) · **Fit** (tu dominio bio/food/edtech).

---

## 1. Qué aportó mejor cada fuente (crédito justo)

**Opus (mi doc):** la tesis "compuerta de privacidad que libera dinero real" (ambas tecnologías inseparables); el marco "elige el stack por *quién custodia el secreto*"; la validación con fuentes (RISC Zero ≠ navegador; Groth16 más barato que UltraHonk; Poseidon/BN254 nativos como edge de jurado).

**Gemini (su doc) — donde me ganó, y lo reconozco:**
- **Passkeys / Secp256r1 (account abstraction nativa).** Yo no lo destaqué. Es un edge de UX **real y diferenciador**: el usuario autoriza la prueba con TouchID/FaceID, sin seed phrases. Sube el "wow" y usa una feature distintiva de Stellar. **Adoptado.**
- **ASP / compliant-privacy** como criterio de juzgamiento de la SDF: probar pertenencia a un set *limpio* sin revelar la dirección. Buen gancho para "privacidad cumplidora". **Adoptado.**
- **EURC** junto a USDC (corredores europeos).
- Ideas nuevas y fuertes: **ZK-Therapy-Reimburse**, **ZK-ClinicalTrial (milestones)**, **ZK-PatientPools (rare-disease)**.

**Gemini — donde se equivocó (misma vara que a mí):**
- Recomendó **Noir casi para todo**; mi validación muestra que UltraHonk está "al borde" del límite de instrucciones y **Groth16 es más barato/seguro**. Su afirmación de que los verificadores son "incredibly cheap, fast, and stable" es optimista — son *viables*, no triviales.
- **DNA-Privacy (#2)** y **EcoSeed (#15)**: "parsear un archivo genómico client-side con RISC Zero" tiene **el mismo error que yo cometí** — RISC Zero no prueba en navegador, así que o el genoma sale del dispositivo (rompe la promesa) o no es client-side. Penalizados igual.

**Tú (MedBridge):** el reencuadre de **ZK como cortafuegos de identidad** y la narrativa de mayor carga emocional del lote (la amenaza isapre/aseguradora). Detalle en `semi-final.md`.

---

## 2. Correcciones técnicas aplicadas a TODAS por igual

| Corrección | A quién afecta | Efecto en el puntaje |
|---|---|---|
| RISC Zero no prueba en navegador → para "el dato del usuario nunca sale del dispositivo" usa Circom/Noir | GenoMatch [O], DNA-Privacy [G], EcoSeed [G], MedBridge [U] | Baja Feas si el pitch dependía de proving client-side con RISC Zero |
| RISC Zero **sí** es ideal cuando un *operador* prueba en servidor | ClinicalTrial [G], Soil-Carbon [O+G], ColdChain [O+G], Solvency [O] | Mantiene/sube Feas (uso honesto) |
| Groth16 < UltraHonk en costo de verificación on-chain | Todas las ideas Noir de Gemini | Riesgo a medir el Día 1; preferir Groth16 si el circuito es simple |
| Poseidon + BN254 nativos = más barato + edge de jurado | Todas | Bonus transversal si se usan explícitamente |
| Passkeys (Secp256r1) = UX sin seed phrase | Todas las de consumidor | +Demo si se integra |

---

## 3. Leaderboard unificado (matriz maestra /30)

Candidatos fusionados, re-puntuados con la rúbrica completa. Ordenado por total.

| Candidato (fusión) | Fuente | ZK-LB | Feas | $STR | Dif | Demo | Fit | **/30** | Tier |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **GenoMatch / DNA-Privacy** (elegibilidad genómica → compensación/descuento) | O+G | 5 | 4 | 5 | 5 | 5 | 5 | **29** | **S** |
| **Scholarship / Credential / UndergradCredit** (ingreso·GPA → beca/préstamo) | O+G | 5 | 5 | 5 | 4 | 5 | 4 | **28** | **S** |
| **Therapy-Reimburse** (beneficio salud mental des-linkable → pago) | G | 5 | 4 | 5 | 4 | 5 | 4 | **27** | **S** |
| **MedBridge** (cortafuegos de identidad para IA diagnóstica) | U | 5 | 3 | 4 | 5 | 4 | 5 | **26** | **A** |
| **PatientPools** (crowdfunding rare-disease, diagnóstico firmado + ASP) | G | 4 | 4 | 5 | 4 | 4 | 4 | **25** | **A** |
| **ClinicalTrial milestones** (eficacia de cohorte → funding USDC) | G | 4 | 3 | 5 | 4 | 4 | 5 | **25** | **A** |
| **ColdChain / Organic-Escrow** (cadena de frío → release USDC) | O+G | 4 | 5 | 4 | 3 | 4 | 5 | **25** | **A** |
| **Terroir / FairPrice / FairFarmer** (fair-trade → premium) | O+G | 4 | 4 | 5 | 4 | 4 | 4 | **25** | **A** |
| **CarbonProof / Soil-Carbon** (huella/secuestro → créditos) | O+G | 5 | 3 | 4 | 4 | 3 | 4 | **23** | **B** |
| **Aid** (caridad anónima + prueba de impacto) | O | 4 | 3 | 5 | 4 | 4 | 3 | **23** | **B** |
| **SkillProof / SkillMatch** (score sobre umbral, bias-free) | O+G | 4 | 5 | 3 | 3 | 4 | 4 | **23** | **B** |
| **Remit** (corredor remesas + compliance privado) | O | 5 | 3 | 5 | 3 | 3 | 3 | **22** | **B** |
| **StudyGroup** (pools de estudio, pass-to-reclaim) | G | 3 | 5 | 4 | 3 | 3 | 4 | **22** | **B** |
| **Tanda / ROSCA** (ahorro rotativo, aportes privados) | O | 3 | 3 | 5 | 4 | 4 | 3 | **22** | **B** |
| **VaxPass** (credencial de salud anónima) | O | 4 | 5 | 3 | 2 | 4 | 4 | **22** | **B** |
| **AllergenFree** (certificación sin receta) | O | 4 | 5 | 3 | 3 | 3 | 4 | **22** | **B** |
| **Solvency local** (proof-of-reserves cooperativa) | O | 4 | 4 | 4 | 3 | 3 | 3 | **21** | **B** |
| **FoodWaste-Tax** (donación de excedente → badge ESG) | G | 4 | 4 | 3 | 3 | 3 | 4 | **21** | **C** |
| **Pro** (profesional licenciado anónimo) | O | 4 | 4 | 4 | 3 | 3 | 3 | **21** | **C** |
| **Consent biobanco** | O | 4 | 3 | 3 | 4 | 2 | 5 | **21** | **C** |
| **CleanAthlete** (anti-doping bajo umbral WADA) | G | 4 | 4 | 3 | 3 | 3 | 3 | **20** | **C** |
| **Salary-Coop** (benchmarking salarial agregado) | O | 4 | 3 | 3 | 4 | 3 | 3 | **20** | **C** |
| **AntiCheat** (integridad de examen sin proctoring) | O | 4 | 2 | 3 | 4 | 3 | 4 | **20** | **C** |
| **Reputation** (skills portables) | O | 3 | 3 | 3 | 3 | 3 | 4 | **19** | **C** |
| **EcoSeed** (semilla GMO-free, RISC0 parse DNA) | G | 4 | 2 | 4 | 3 | 3 | 4 | **18** | **C⚠** |
| **UrbanGarden** (descuento predial por área verde, RISC0 imágenes) | G | 3 | 2 | 3 | 3 | 3 | 2 | **16** | **C⚠** |
| **Diagnostic-Fairness** (zkML del modelo) | O | 5 | 1 | 3 | 5 | 3 | 5 | **22*** | **🔴 trampa** |

\* Diagnostic-Fairness puntúa alto en papel pero **zkML de un modelo real es inviable a 10 días** → trampa de alcance. No construir.

⚠ EcoSeed/UrbanGarden: dependen de RISC Zero procesando DNA/imágenes pesadas; honestas solo si el operador prueba en servidor, lo que mata el ángulo "tu dato privado nunca sale".

---

## 4. Lectura del torneo (tiers)

- **Tier S (28–29): los favoritos.** GenoMatch, Scholarship, Therapy-Reimburse. Comparten la **misma columna vertebral** (`credencial auténtica privada → prueba ZK de propiedad sin identidad → dinero en Stellar`) y todos tienen Feas alta y narrativa clara.
- **Tier A (25–26): alto techo, más superficie.** MedBridge (techo narrativo máximo, riesgo de encuadre), PatientPools, ClinicalTrial, ColdChain, FairPrice.
- **Tier B/C:** sólidas pero menos diferenciadas o con dinero/Stellar más débil (badges en vez de pagos), o con riesgo técnico (AntiCheat, EcoSeed, UrbanGarden).

**Observación de justicia:** por la rúbrica, **una idea de Gemini (Therapy-Reimburse, 27) supera a tu MedBridge (26)** y casi empata con mi Scholarship. No lo maquillo: es, probablemente, **la mejor síntesis ajustada por riesgo** del lote — captura el corazón emocional de MedBridge (proteger tu salud de un tercero poderoso + pago real) con factibilidad Tier-S y **cero riesgo de error de categoría** (no hay modelo que explicar). Volvemos a esto en §6.

---

## 5. Head-to-head: **MedBridge vs GenoMatch** (lo que pediste)

Matriz granular. Ambas son tuyas-por-afinidad (biotech) y comparten andamiaje, así que la decisión es de *perfil de riesgo*, no de arquitectura.

| Dimensión | 🧬 **GenoMatch** | 🏥 **MedBridge** | Gana |
|---|---|---|:---:|
| **Claridad del rol de ZK** (¿sobrevive a un jurado técnico?) | Alta — un *match discreto* (¿portas la variante? sí/no). Sin ambigüedad | Media — debes **desactivar** "¿pero el servidor no ve el dato?" (ZK≠FHE) antes de que lo pregunten | **Geno** |
| **Factibilidad del circuito (10d)** | Alta — membership Poseidon + igualdad + nullifier | Media-alta — mismo core + des-linkabilidad + servicio mockeado | **Geno** |
| **Honestidad conceptual** (fácil de mantener verdadero) | Alta — el claim es exactamente lo que el circuito hace | Media — "alimenta el modelo" tienta a sobre-prometer; hay que disciplinar el discurso | **Geno** |
| **Potencia narrativa / emocional** | Alta — discriminación genética, GINA | **Máxima** — "una preexistencia y la isapre te castiga por enfermar" | **Med** |
| **Diferenciación** | Muy alta | **Máxima** — casi nadie hará "cortafuegos de identidad para IA" | **Med** |
| **Demo en 2 min** | Alta — flujo limpio, "no se revela un solo gen" | Media-alta — requiere mostrar el *corte del vínculo* identidad↔datos | **Geno** |
| **Fit con tu dominio** | Máximo (biotech) | Máximo (biotech + ML) | empate |
| **Stellar = dinero real** | Pago de compensación de enrolamiento | Microconsulta/derivación pagada | empate |
| **Riesgo "demasiado para el hack"** | Bajo | Medio | **Geno** |
| **Superficie de ataque conceptual** | Baja | Media (error de categoría) | **Geno** |
| **Techo si sale perfecto** | Muy alto | **El más alto del torneo** | **Med** |
| **Varianza (riesgo)** | Baja | Media-alta | depende |

### Veredicto del head-to-head
- **GenoMatch = mayor valor esperado / menor varianza.** Narrativa top, ZK cristalino, casi imposible que un jurado técnico lo desarme. Es la jugada para **asegurar un top-3**.
- **MedBridge = mayor techo / mayor varianza.** Si clavas el encuadre "cortafuegos" en el video (y tu credibilidad biotech+ML lo vende), es **la historia que gana el #1**. Si titubeas en explicar ZK≠FHE, un juez técnico puede restarte.
- **No tienes que elegir hoy.** Ambas usan idéntico andamiaje (Poseidon + Groth16 + nullifier + pago). Construye el núcleo común y **decide en el Día 3** según cómo te sientas con el encuadre.

### Sensibilidad al peso (transparencia)
Si subes el peso de **Dif + Demo** (lo que un hackathon arguablemente premia), MedBridge y GenoMatch quedan **empatados**. Si subes **Feas + claridad-ZK** (lo que un jurado técnico premia), GenoMatch se despega. Mi rúbrica usa pesos iguales; ajústalos a tu lectura del jurado.

---

## 6. La síntesis que quizá los vence a ambos: **Therapy-Reimburse** (de Gemini)

Vale la pena decirlo porque pediste justicia: la idea de Gemini de **beneficio de salud mental des-linkable** puede ser el **mejor punto medio**:
- Tiene el **corazón emocional de MedBridge** (proteger tu información de salud de un tercero poderoso — aquí el *empleador*, no la isapre) y **paga de verdad** (USDC a una dirección fresca, sin rastro).
- Tiene **factibilidad Tier-S** (membership en nómina + invoice firmada por terapeuta + dirección de salida fresca + pago) — más simple que MedBridge.
- **Cero error de categoría**: no hay modelo que explicar; ZK solo hace lo que sabe hacer (membership + des-linkabilidad). El claim es 100% honesto.
- Con **Passkeys** (el aporte de Gemini) el demo es buenísimo: autorizas con FaceID y el dinero llega sin que tu jefe sepa que fuiste a terapia.

**Una variante "MedBridge-lite" honesta:** generaliza Therapy-Reimburse a *cualquier* servicio de salud → "accede a salud y paga **sin que tu empleador/aseguradora pueda saber qué consultaste**". Eso es tu visión de cortafuegos, **construible y a prueba de jurado técnico**, sin prometer que un LLM corre bajo ZK.

---

## 7. Recomendación final

Tres caminos, elige por apetito de riesgo:

1. **Quieres asegurar podio (menor varianza):** **Scholarship** (piso Tier-S, dos range proofs) o **GenoMatch** (mismo riesgo, más wow). Empieza por el andamiaje común.
2. **Quieres pegar al #1 con la historia más memorable y disciplinarás el encuadre:** **MedBridge** como *cortafuegos de identidad* con el modelo mockeado (ver `semi-final.md`). Alto techo, exige que clavés el "ZK≠FHE" en 15 segundos de video.
3. **Quieres el mejor riesgo-ajustado, fusionando lo mejor de los tres:** **Therapy-Reimburse / MedBridge-lite** — el corazón de tu idea, la factibilidad de GenoMatch, los Passkeys de Gemini, y cero superficie de ataque conceptual.

**Mi apuesta personal, siendo honesto:** si tu credibilidad biotech+ML en cámara es fuerte y te disciplinas en el discurso → **MedBridge** por el techo. Si quieres dormir tranquilo → **GenoMatch**. Y si tuviera que poner mi propio dinero al *valor esperado* → **Therapy-Reimburse/MedBridge-lite**, porque maximiza (emoción × factibilidad × honestidad) que es justo lo que un jurado de "Real-World ZK" recompensa.

> Las tres comparten columna vertebral. Construye el núcleo (membership Poseidon + nullifier + verificador Groth16 + pago SEP-41 + Passkey) en los Días 1–4; la "personalidad" (genoma / firewall / terapia) se decide después sin perder trabajo.

---

### Apéndice — cosas que ROBO de Gemini para cualquier opción que elijas
- **Passkeys (Secp256r1):** autoriza la prueba con biometría, sin seed phrase. Gran momento de demo.
- **ASP / allowlist dentro del circuito:** "soy de un grupo limpio" = privacidad cumplidora (gancho SDF).
- **Dirección de salida fresca** en el output del circuito: el pago llega a una wallet sin vínculo con el solicitante.
- **EURC** si la historia es europea (CBAM, GDPR).
