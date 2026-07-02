# Brainstorming — Stellar Hacks: Real-World ZK
**Modelo:** Opus 4.8 · **Fecha:** 2026-06-19 · **Deadline del hackathon:** 2026-06-29 12:00 PST (~10 días)

> Documento de estrategia + lluvia de ideas + análisis de viabilidad para ganar el hackathon "Stellar Hacks: Real-World ZK" ($10,000). Pensado para un builder con experiencia en blockchain, algo de ZK, y dominio en **edtech, foodtech y biotech**.

---

## 1. Análisis profundo del hackathon (qué premia REALMENTE)

### 1.1 Lo que el brief dice explícitamente
- **Track único de innovación abierta.** $10k repartidos en 5 puestos ($5k / $2k / $1.25k / $1k / $0.75k). No hay categorías que "rellenar"; gana la calidad relativa.
- **Requisitos mínimos:** repo open-source + README claro, video de 2–3 min mostrando que funciona, y ZK que sea *load-bearing* ("no namechecked en el README").
- **Tres caminos probados:** RISC Zero (zkVM, escribes Rust normal), Noir (DSL tipo Rust, pruebas UltraHonk más caras), Circom (Groth16, verificación más barata pero más difícil de escribir).
- **Verificadores ya existen como "starter code":** verificador RISC Zero de Nethermind, verificador UltraHonk para Noir, y la PoC de Privacy Pools de Nethermind (Circom + Groth16). **No tienes que escribir criptografía nueva.**
- **Stellar = movimiento de dinero real.** El brief lo repite: stablecoins, remesas, RWA, settlement. "Proyectos que llevan ZK a esos casos de uso son especialmente bienvenidos."

### 1.2 Lo que el brief premia entre líneas (meta-juicio)
- **"Mild projects win hackathons all the time when they're sharp and well-executed."** → Traducción: un *vertical slice* estrecho y pulido vence a un moonshot a medio terminar. El enemigo es el alcance.
- **"We'd rather see an honest work-in-progress than a polished mystery."** → Documentar datos mock honestamente **suma puntos**, no resta. Esto reduce drásticamente el riesgo de construcción.
- **El gap es el premio:** "powerful primitives" vs "finished product". El jurado sabe que las primitivas P25/P26 (BN254, Poseidon, BLS12-381) no dan pagos privados llave-en-mano. El valor está en **cerrar ese gap con un caso de uso concreto.**
- **El video de 2–3 min es donde se gana.** Un jurado cansado recuerda una *historia de una frase*. "Pruebo que tengo la mutación genética para un ensayo clínico sin revelar mi genoma, y cobro la compensación en USDC" es memorable. "Otro privacy pool" no lo es.

### 1.3 El test de "ZK genuinamente esencial"
Antes de aceptar cualquier idea, pasa este filtro: **¿el proyecto se rompe si quitas la privacidad?** Si la respuesta es "no, simplemente sería menos privado", la idea es débil. Si es "sí, nadie lo usaría / sería ilegal / revelaría un secreto comercial / pondría a alguien en peligro", la idea es fuerte. Los mejores casos de uso son aquellos donde **la privacidad no es un nice-to-have sino una necesidad legal, económica o de seguridad física.**

### 1.4 La tesis para ganar (resumen ejecutivo)
> **Construye una "compuerta de privacidad que libera dinero real en Stellar", en un dominio (bio/food/edtech) donde la privacidad es obligatoria, usando el sistema de pruebas correcto según quién custodia el secreto (ver §7), sobre un verificador ya existente, con un vertical slice estrecho y datos mock honestos, y un demo de 2 min con una historia de una frase.**

> ⚠️ **Corrección de la v1 (validada con evidencia, ver §7):** la elección del sistema de pruebas NO es "RISC Zero por default". Depende de **quién custodia el secreto**. Si lo custodia el *consumidor* y debe probarse en su propio dispositivo (genoma, finanzas, score) → **Circom/Groth16 o Noir** (proving en navegador vía WASM; RISC Zero NO prueba en navegador). Si lo custodia un *operador de confianza* que prueba en su servidor (cadena de frío, huella de carbono, reservas) → **RISC Zero** (escribes Rust). Ver la tabla de decisión en §7.

---

## 2. Lluvia de ideas (20 ideas, fuera de la caja)

Cada idea: **pitch de una frase · por qué ZK es esencial · rol de Stellar · stack · gancho real.**

### 🧬 Biotech / Salud

**1. ZK-GenoMatch — Matching a ensayos clínicos sin revelar tu genoma**
Pruebas que tu genoma contiene (o NO contiene) los marcadores de elegibilidad de un ensayo clínico, sin exponer un solo SNP, y cobras la compensación de enrolamiento en USDC.
- *ZK esencial:* el genoma es el dato más sensible que existe (discriminación por seguros — GINA en EEUU). Los pacientes se niegan a compartirlo; los sponsors necesitan pruebas anti-fraude. Sin ZK no hay producto.
- *Stellar:* la prueba válida + nullifier (anti-Sybil) desbloquea el pago de compensación al paciente; el sponsor verifica on-chain.
- *Stack:* RISC Zero (guest program en Rust parsea un VCF mock y evalúa criterios), verificador RISC0 de Nethermind, contrato Soroban que paga.
- *Gancho:* discriminación genética es ley en EEUU; matching de ensayos es un mercado de miles de millones.

**2. ZK-VaxPass — Credencial de salud verificable y anónima**
Pruebas que tienes una vacuna/serología válida emitida por una autoridad reconocida, sin revelar tu identidad ni tu historia clínica completa, con nullifier para evitar reuso.
- *ZK esencial:* historial médico es PHI/HIPAA; revelar identidad para entrar a un evento es desproporcionado.
- *Stellar:* attestation del emisor anclada en contrato; gating de acceso/pago.
- *Stack:* Noir (membership + nullifier) o RISC Zero.
- *Gancho:* post-pandemia, pasaportes de salud privados siguen siendo relevantes.

**3. ZK-ColdChain Pharma — Prueba de cadena de frío sin revelar la telemetría**
Un distribuidor farmacéutico prueba que un lote de vacunas/insulina **nunca salió del rango 2–8°C** durante el transporte, sin revelar el log completo de sensores (que expone rutas, tiempos y socios — secreto comercial), y libera el pago al transportista.
- *ZK esencial:* el log de telemetría es inteligencia competitiva; el comprador solo necesita el veredicto pass/fail.
- *Stellar:* prueba pass → settlement automático del pago + sello de compliance.
- *Stack:* RISC Zero (Rust evalúa min/max sobre serie temporal de sensores).
- *Gancho:* la OMS estima que ~25% de las vacunas se degradan por fallos de cadena de frío.

**4. ZK-Consent — Prueba de uso de datos de biobanco dentro del consentimiento**
Un investigador prueba que su análisis usó datos de pacientes **solo dentro del alcance consentido** (p.ej. "solo investigación de cáncer, no comercial"), sin exponer ni los datos ni el modelo, y desbloquea el pago de acceso al biobanco.
- *ZK esencial:* cumplimiento de consentimiento GDPR/HIPAA es auditable pero los datos no pueden salir.
- *Stellar:* pago de licencia de datos condicionado a la prueba de compliance.
- *Stack:* RISC Zero (verificable computation sobre query autorizada).
- *Gancho:* gobernanza de datos biomédicos es un dolor regulatorio enorme.

**5. ZK-Diagnostic Fairness — Prueba de que un diagnóstico IA usó el modelo auditado**
Una clínica prueba que su recomendación diagnóstica salió de un **modelo aprobado/auditado** (no de uno alterado), sin revelar los pesos del modelo (propiedad intelectual) ni los datos del paciente.
- *ZK esencial:* el modelo es IP del fabricante; los datos son PHI; el regulador solo necesita garantía de integridad.
- *Stellar:* settlement de licencia por inferencia + sello de auditoría.
- *Stack:* RISC Zero (zkML ligero; factible solo con modelos pequeños — riesgo alto).
- *Gancho:* "IA médica auditable" es un tema candente regulatorio (FDA/EU AI Act).

### 🌱 Foodtech / Agri

**6. ZK-Terroir — Prueba de origen/fair-trade sin revelar la cadena de suministro**
Un tostador de café prueba que un lote es *single-origin / fair-trade / orgánico* (cada eslabón certificado) sin revelar su lista de proveedores (secreto comercial), y paga el **premium fair-trade** automáticamente al cooperativa.
- *ZK esencial:* la cadena de proveedores es el activo competitivo; el comprador final solo quiere la garantía.
- *Stellar:* prueba válida → pago del premium en USDC a la cooperativa (caso de uso clásico de Stellar en mercados emergentes).
- *Stack:* RISC Zero o Noir (membership sobre Merkle de certificados).
- *Gancho:* greenwashing y fraude de "fair-trade" son endémicos; Stellar ya mueve remesas a esos países.

**7. ZK-CarbonProof — Huella de carbono bajo umbral sin revelar el proceso**
Un fabricante de alimentos prueba que la huella de CO₂ de un producto está bajo un umbral (para acceder a un mercado/premium "low-carbon"), sin revelar datos de proceso propietarios, y liquida créditos de carbono en Stellar.
- *ZK esencial:* la receta/proceso es secreto industrial; el regulador/comprador solo necesita el número bajo umbral.
- *Stellar:* retiro/transferencia de créditos de carbono tokenizados condicionado a la prueba.
- *Stack:* RISC Zero (cálculo LCA en Rust → output: huella ≤ umbral).
- *Gancho:* CBAM europeo obliga a probar huellas de carbono — mercado regulatorio gigante.

**8. ZK-AllergenFree — Certificación sin revelar la receta**
Un productor prueba que su producto pasó tests de alérgenos (gluten, frutos secos) bajo el umbral legal, sin revelar la formulación.
- *ZK esencial:* la receta es IP; el consumidor/retailer solo quiere el veredicto.
- *Stellar:* sello de certificación + pago de la auditoría.
- *Stack:* RISC Zero (range proof sobre resultados de laboratorio).
- *Gancho:* recalls por alérgenos son costosísimos y legales.

**9. ZK-Microinsurance Agrícola — Payout paramétrico sin doxear tu parcela**
Un pequeño agricultor prueba que (a) posee una parcela en una región y (b) un oráculo satelital/climático confirma sequía/inundación que dispara el seguro paramétrico — **sin revelar las coordenadas GPS exactas** (riesgo de robo/extorsión) — y cobra el payout en USDC.
- *ZK esencial:* revelar la ubicación exacta de la parcela y la propiedad pone en riesgo físico al agricultor; la aseguradora necesita prueba anti-fraude.
- *Stellar:* prueba del trigger → payout automático (caso de uso insignia de Stellar para inclusión financiera).
- *Stack:* RISC Zero (Rust evalúa: punto dentro de polígono de región + dato de oráculo cruza umbral).
- *Gancho:* seguro paramétrico agrícola es la killer-app de fintech-for-development.

**10. ZK-FairFarmer — Prueba de que el agricultor cobró por encima del precio piso**
Una marca prueba a sus consumidores que **todos** sus agricultores cobraron por encima del precio justo, sin revelar contratos individuales ni montos.
- *ZK esencial:* los montos de contrato son confidenciales; el consumidor solo quiere la garantía agregada.
- *Stellar:* pagos confidenciales reales a agricultores + prueba agregada de compliance.
- *Stack:* confidential token / Circom (range proofs agregados).
- *Gancho:* ESG verificable es un mercado en explosión.

### 🎓 Edtech

**11. ZK-Scholarship — Elegibilidad de beca need-based sin revelar tus finanzas**
Un estudiante prueba que su ingreso familiar está bajo un umbral **Y** su nota está sobre otro umbral, combinando credenciales del banco y de la universidad, sin revelar ingreso exacto, notas exactas ni identidad — y se libera el desembolso de la beca en USDC.
- *ZK esencial:* revelar pobreza es estigmatizante; el fraude de becas es real; el donante solo necesita "califica/no califica".
- *Stellar:* desembolso transfronterizo de la beca (Stellar brilla en pagos internacionales a estudiantes).
- *Stack:* RISC Zero o Circom (dos range proofs combinados sobre credenciales firmadas).
- *Gancho:* becas internacionales + privacidad financiera = historia con corazón y números.

**12. ZK-SkillProof — Prueba de que aprobaste sin revelar tu puntaje ni tu identidad**
Pruebas que sacaste ≥X en un examen de certificación (idiomas, CFA, AWS) sin revelar el puntaje exacto ni quién eres, para postular a un empleo o un airdrop de talento.
- *ZK esencial:* el puntaje exacto puede usarse en tu contra; quieres probar el umbral, no el número.
- *Stellar:* credencial anclada + posible bounty/pago por talento verificado.
- *Stack:* Noir o Circom (range proof sobre score firmado por el emisor).
- *Gancho:* mercados de talento anónimos basados en skill verificado.

**13. ZK-AntiCheat — Prueba de integridad de un examen remoto sin grabar al estudiante**
Pruebas que un examen online se rindió bajo condiciones válidas (tiempo, sin segundas sesiones, entorno verificado) sin enviar video ni datos biométricos crudos a un servidor.
- *ZK esencial:* la vigilancia con cámara (proctoring) es invasiva y está siendo prohibida; ZK prueba la integridad sin el dato crudo.
- *Stellar:* sello de validez del examen + pago de la certificación.
- *Stack:* RISC Zero (verifica condiciones localmente → prueba), riesgo medio.
- *Gancho:* el proctoring con IA tiene una reacción regulatoria fuerte (privacidad estudiantil).

**14. ZK-Reputation — Reputación de skills portable sin doxear tu historial**
Acumulas credenciales de skill de varias plataformas y pruebas un perfil agregado ("senior backend + 3 certs cloud") sin revelar de qué plataformas ni tu identidad, para un marketplace freelance.
- *ZK esencial:* unir tu historial entre plataformas es doxeo; quieres probar el agregado, no las fuentes.
- *Stellar:* reputación anclada + pagos del marketplace.
- *Stack:* Noir (membership + agregación).
- *Gancho:* identidad profesional soberana.

### 🌐 Transversales / Wildcards (fuera de la caja total)

**15. ZK-Aid — Donación caritativa anónima con prueba de impacto**
Un donante da fondos anónimamente y recibe una **prueba de que el dinero llegó a beneficiarios verificados** (refugiados, damnificados) sin revelar identidad del donante ni de los receptores.
- *ZK esencial:* donantes de alto perfil quieren anonimato; los receptores (refugiados) necesitan protección; el fraude en ayuda es masivo.
- *Stellar:* Stellar **ya** se usa para ayuda humanitaria transfronteriza — encaje perfecto.
- *Stack:* privacy pool + prueba de distribución a allowlist verificada.
- *Gancho:* "caridad verificable y privada" es una historia poderosa para el jurado.

**16. ZK-Pro — Consulta anónima con un profesional licenciado verificado**
Un médico/abogado prueba que tiene licencia vigente (sin revelar su nombre) para dar consejo en una plataforma anónima, y cobra la consulta en USDC.
- *ZK esencial:* el profesional quiere anonimato (responsabilidad/competencia); el usuario necesita garantía de que es real.
- *Stellar:* pago de la consulta condicionado a la prueba de licencia.
- *Stack:* Noir (membership en registro de colegiados + nullifier).
- *Gancho:* telemedicina/asesoría anónima pero confiable.

**17. ZK-Remit — Prueba de propósito de remesa para compliance sin doxear a la familia**
Un migrante prueba que su remesa es para "sostén familiar" (no sancionada, no lavado) cumpliendo AML, sin revelar la identidad del receptor al público.
- *ZK esencial:* compliance AML exige garantías; la familia receptora necesita privacidad; selective disclosure con view key para el regulador.
- *Stellar:* corredor de remesas (el caso de uso #1 de Stellar) con compliance privado.
- *Stack:* privacy pool + view key (patrón de "selective disclosure" que el brief destaca).
- *Gancho:* el brief literalmente lista "private cross-border remittance corridor" como moonshot.

**18. ZK-Tanda — ROSCA/tanda con aleatoriedad verificable y aportes privados**
Un grupo de ahorro rotativo (tanda/chama/susu) usa ZK para probar que cada miembro aportó y que el orden del payout fue sorteado de forma justa y verificable, sin revelar montos individuales.
- *ZK esencial:* los miembros no quieren revelar su capacidad de ahorro; necesitan confianza en la justicia del sorteo.
- *Stellar:* settlement de los aportes y payouts del pool.
- *Stack:* Circom (fairness + range proofs) + VRF.
- *Gancho:* las ROSCAs mueven billones informalmente en mercados emergentes — Stellar las formaliza.

**19. ZK-Salary Co-op — Benchmarking salarial sin revelar tu sueldo**
Empleados aportan su salario a un pool que computa estadísticas agregadas (mediana, percentiles) **probadas correctas**, sin que nadie vea sueldos individuales.
- *ZK esencial:* revelar tu sueldo es tabú/peligroso laboralmente; el agregado es valioso para todos.
- *Stellar:* coordinación/incentivos del co-op de datos.
- *Stack:* RISC Zero (cómputo agregado verificable sobre inputs comprometidos).
- *Gancho:* transparencia salarial sin sacrificio individual.

**20. ZK-Solvency Local — Prueba de reservas para una cooperativa/fintech sin exponer cuentas**
Una cooperativa de ahorro o emisor de stablecoin local prueba que sus reservas cubren los depósitos (solvencia ≥ 100%) sin revelar saldos de cuentas individuales.
- *ZK esencial:* los saldos son confidenciales; los depositantes solo necesitan la garantía de solvencia.
- *Stellar:* proof-of-reserves on-chain para un activo emitido en Stellar.
- *Stack:* RISC Zero o Circom (suma de saldos comprometidos ≥ pasivos).
- *Gancho:* post-FTX, proof-of-reserves es estándar de oro; aplicado a cooperativas reales.

---

## 3. Abstracción profunda y matriz de viabilidad

### 3.1 Rúbrica de evaluación (1–5, mayor = mejor)
- **ZK-LB (ZK load-bearing):** ¿se rompe el proyecto sin privacidad? ¿es la privacidad legal/económicamente obligatoria?
- **Feas (Factibilidad en ~10 días):** complejidad del circuito + disponibilidad de datos + reuso de verificadores existentes.
- **$STR (Stellar como movimiento de dinero):** ¿la prueba libera/condiciona dinero real, o Stellar es solo "el verificador"?
- **Dif (Diferenciación):** ¿destaca frente a la pila de privacy-pools que verá el jurado?
- **Demo (Demo-ability + narrativa):** ¿hay historia de una frase y un flujo visible end-to-end en 2 min?
- **Fit (Encaje con tu dominio bio/food/edtech/zk):** ¿tu expertise hace el demo creíble y profundo?

### 3.2 Matriz

| # | Idea | ZK-LB | Feas | $STR | Dif | Demo | Fit | **Total/30** | Veredicto |
|---|------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|---|
| 1 | ZK-GenoMatch (ensayos) | 5 | 4 | 5 | 5 | 5 | 5 | **29** | 🏆 Top |
| 9 | ZK-Microinsurance agrícola | 5 | 4 | 5 | 5 | 4 | 4 | **27** | 🏆 Top |
| 11 | ZK-Scholarship | 5 | 5 | 5 | 4 | 5 | 4 | **28** | 🏆 Top |
| 3 | ZK-ColdChain Pharma | 4 | 5 | 4 | 4 | 4 | 5 | **26** | 🥈 Fuerte |
| 7 | ZK-CarbonProof | 5 | 4 | 4 | 4 | 3 | 4 | **24** | 🥈 Fuerte |
| 6 | ZK-Terroir (fair-trade) | 4 | 4 | 5 | 4 | 4 | 4 | **25** | 🥈 Fuerte |
| 15 | ZK-Aid (caridad) | 4 | 3 | 5 | 4 | 4 | 3 | **23** | 🥈 Fuerte |
| 17 | ZK-Remit compliance | 5 | 3 | 5 | 3 | 3 | 3 | **22** | 🟡 Sólido |
| 12 | ZK-SkillProof | 4 | 5 | 3 | 3 | 4 | 4 | **23** | 🟡 Sólido |
| 20 | ZK-Solvency local | 4 | 4 | 4 | 3 | 3 | 3 | **21** | 🟡 Sólido |
| 2 | ZK-VaxPass | 4 | 5 | 3 | 2 | 4 | 4 | **22** | 🟡 Sólido |
| 8 | ZK-AllergenFree | 4 | 5 | 3 | 3 | 3 | 4 | **22** | 🟡 Sólido |
| 18 | ZK-Tanda (ROSCA) | 3 | 3 | 5 | 4 | 4 | 3 | **22** | 🟡 Sólido |
| 19 | ZK-Salary Co-op | 4 | 3 | 3 | 4 | 3 | 3 | **20** | 🟡 Sólido |
| 16 | ZK-Pro (licencia) | 4 | 4 | 4 | 3 | 3 | 3 | **21** | 🟡 Sólido |
| 4 | ZK-Consent biobanco | 4 | 3 | 3 | 4 | 2 | 5 | **21** | 🟠 Nicho |
| 10 | ZK-FairFarmer | 4 | 3 | 4 | 3 | 3 | 4 | **21** | 🟠 Nicho |
| 13 | ZK-AntiCheat | 4 | 2 | 3 | 4 | 3 | 4 | **20** | 🟠 Riesgo |
| 14 | ZK-Reputation | 3 | 3 | 3 | 3 | 3 | 4 | **19** | 🟠 Riesgo |
| 5 | ZK-Diagnostic Fairness (zkML) | 5 | 1 | 3 | 5 | 3 | 5 | **22** | 🔴 Trampa* |

\* **Idea 5 es una trampa de alcance:** ZK sobre ML (zkML) es técnicamente fascinante y máximamente diferenciado, pero generar pruebas de inferencia de un modelo no-trivial en 10 días es irreal. Solo viable con un modelo juguete, lo que mata la credibilidad. Evítala salvo que reduzcas a un clasificador lineal de 5 features.

### 3.3 Patrones que emergen del análisis
1. **Las ganadoras comparten una estructura:** `credencial/dato privado → prueba off-chain (RISC Zero) → verificación on-chain → dinero se mueve en Stellar`. Esa estructura hace ZK y Stellar **inseparables**.
2. **Range proofs + membership proofs cubren el 80% de los casos reales** y son los circuitos más baratos/rápidos de construir. Las ideas que se reducen a "valor en rango" o "estás en el set" son las más factibles (#11, #3, #8, #12).
3. **RISC Zero gana en factibilidad** cuando la lógica de dominio es no-trivial (parsear VCF, evaluar elegibilidad, calcular LCA) porque escribes Rust en vez de circuitos. Circom gana si quieres verificación on-chain barata y la lógica es un range/membership simple.
4. **El "Fit" con tu dominio es un multiplicador de credibilidad en el video.** Un médico-builder explicando privacidad genómica vence a un cripto-builder explicando lo mismo, aunque el código sea idéntico.

---

## 4. Finalistas — análisis a fondo

### 🏆 Finalista A — ZK-Scholarship (#11) · *la apuesta segura para ganar*
**Por qué:** máxima factibilidad (dos range proofs combinados, datos 100% mockeable, sin oráculos, sin live data), ZK indiscutiblemente esencial (privacidad financiera + anti-fraude de becas), Stellar load-bearing (desembolso transfronterizo = caso insignia), narrativa con corazón Y números, y encaja con tu edtech.
**El slice ganador:** un estudiante con (1) una credencial firmada de ingreso del "banco" y (2) una credencial firmada de GPA de la "universidad" genera client-side una prueba `ingreso < $30k AND gpa > 3.5`. El contrato Soroban verifica y dispara un pago en USDC testnet. Nullifier previene doble cobro.
**Riesgo:** bajo. Es esencialmente dos range proofs + un pago. El reto es la UX y la historia, no la cripto.

### 🏆 Finalista B — ZK-GenoMatch (#1) · *la apuesta de máximo impacto / "wow"*
**Por qué:** la historia más memorable del hackathon ("pruebo que tengo la mutación sin revelar mi genoma y cobro"), máxima diferenciación, ZK esencial al extremo (el dato más sensible que existe), y aprovecha tu biotech como nadie más puede.
**El slice ganador (arquitectura corregida — ver §7):** un "laboratorio" mock firma/compromete tu genotipo en un **árbol de Merkle con hash Poseidon** (host function nativa de Stellar) → esa raíz es la attestation pública. En tu **navegador**, un circuito **Noir o Circom** prueba: *"los SNPs en las posiciones que el ensayo exige pertenecen al árbol comprometido (membership Poseidon) Y cumplen los criterios (igualdad/rango), Y nullifier = Poseidon(secreto, trial_id)"* — sin revelar un solo SNP. La prueba se genera client-side vía WASM (el genoma nunca sale del navegador). El contrato Soroban verifica con `pairing_check` BN254 nativo y paga la compensación.
**Por qué NO RISC Zero aquí:** RISC Zero no prueba en navegador (solo Local Prover/Bonsai), lo que rompería el "tu genoma nunca sale del dispositivo". El circuito no "parsea un VCF" on-chain; representa el genotipo como vector comprometido y prueba propiedades sobre él — un patrón membership+igualdad estándar.
**Riesgo:** medio-bajo. El circuito es membership + igualdad + un hash (estándar). El reto real es el **costo de verificación on-chain**: si usas Noir/UltraHonk, mide instrucciones el Día 2 (está "al borde" del límite — ver §7); Circom/Groth16 es el camino más barato y seguro.

### 🏆 Finalista C — ZK-Microinsurance Agrícola (#9) · *la apuesta "real-world Stellar" pura*
**Por qué:** es literalmente el caso de uso que Stellar evangeliza (inclusión financiera, payouts a agricultores), ZK esencial (seguridad física por la geolocalización + anti-fraude), y combina foodtech + payments.
**El slice ganador:** agricultor prueba `punto-en-polígono (mi parcela ∈ región) AND oráculo_lluvia < umbral_sequía` sin revelar el GPS exacto. Prueba válida → payout USDC automático.
**Riesgo:** medio. La parte de oráculo añade una pieza (puedes mockear el feed climático honestamente). El point-in-polygon en Rust es factible.

### Comparación de finalistas
| Criterio | A: Scholarship | B: GenoMatch | C: Microinsurance |
|---|:---:|:---:|:---:|
| Probabilidad de **terminar** | ★★★★★ | ★★★★ | ★★★½ |
| Factor **"wow" / memoria del jurado** | ★★★½ | ★★★★★ | ★★★★ |
| **Stellar** como dinero real | ★★★★★ | ★★★★ | ★★★★★ |
| Profundidad de **tu expertise** | ★★★★ | ★★★★★ | ★★★★ |
| Riesgo técnico | Bajo | Medio-bajo | Medio |

---

## 5. Recomendación final

**Construye el Finalista B (ZK-GenoMatch) si quieres maximizar el techo (1er puesto / historia inolvidable), o el Finalista A (ZK-Scholarship) si quieres maximizar la probabilidad de terminar bien (top-5 casi garantizado).**

Mi recomendación de mayor valor esperado: **ZK-GenoMatch con un alcance brutalmente recortado.** Tienes la credibilidad biotech para venderlo, es la historia que el jurado recordará, y el circuito (membership Poseidon + igualdad + nullifier) es estándar y factible. Si en el día 4 ves que te complica, **el plan B es pivotar a Scholarship reutilizando el 80% del andamiaje** (ambos son "credencial privada → prueba de elegibilidad → pago en Stellar"; mismo verificador, distinta lógica de elegibilidad). Diseña el repo para que ese pivote sea barato.

### Plan de construcción (10 días) — stack corregido: **Circom/Groth16 (o Noir) con proving en navegador**
- **Día 1–2 — Andamiaje + el spike de riesgo #1.** Clona el verificador que vas a usar (`soroban-examples/groth16_verifier` o la PoC de Privacy Pools para Circom; `ultrahonk_soroban_contract` para Noir), despliégalo en testnet, y **mide el costo real de verificación de una prueba dummy en instrucciones**. *Hito: una prueba dummy verifica on-chain y conoces su costo.* Esto valida lo único que de verdad puede hundir el proyecto (ver §7).
- **Día 3–4 — Circuito de dominio.** Escribe el circuito (Circom o Noir): membership Poseidon sobre el genotipo comprometido + chequeo de criterios + nullifier. Genera la prueba con snarkjs/bb.js. *Hito: prueba real de un genoma mock verifica on-chain.*
- **Día 5–6 — Contrato de pago.** Contrato Soroban que: verifica la prueba (`pairing_check` BN254 nativo), chequea el nullifier (anti doble-cobro), y transfiere USDC testnet al paciente vía interfaz SEP-41. *Hito: prueba válida → pago ejecutado.*
- **Día 7–8 — UX en el navegador (el momento "wow").** Frontend que genera la prueba **client-side vía WASM** (snarkjs o bb.js, como la PoC de Privacy Pools): cargas el genotipo, la prueba se genera localmente, y ves el pago. *El genoma nunca sale del navegador — ahí está el demo.*
- **Día 9 — README honesto + pulido.** Documenta qué es mock (el genotipo, la firma del "laboratorio", el "sponsor") y qué es real (el circuito, la verificación on-chain, el pago). El brief premia la honestidad.
- **Día 10 — Video de 2–3 min.** Guion abajo. Buffer para imprevistos.

### Guion del video (la frase que gana)
> *"Tu genoma es el dato más sensible que tienes — y compartirlo para un ensayo clínico puede costarte tu seguro médico. Con ZK-GenoMatch, pruebas matemáticamente que tienes la mutación que el ensayo necesita, **sin revelar un solo gen**, y cobras tu compensación en segundos sobre Stellar. El genoma nunca sale de tu dispositivo. Esto es ZK haciendo trabajo real sobre dinero real."*

### Cómo de-riesgar (lecciones del brief + validación de §7)
1. **El spike de costo de verificación es lo primero (Día 1).** El único riesgo que hunde el proyecto es que la verificación on-chain no quepa en el límite de instrucciones. Mídelo con una prueba dummy antes de escribir nada de lógica. Si usas Noir/UltraHonk y no cabe → cae a Circom/Groth16.
2. **Reusa verificadores, no escribas cripto.** Los verificadores ya existen como starter code (Groth16 en `soroban-examples`, UltraHonk en `ultrahonk_soroban_contract`, RISC0 de Nethermind).
3. **Elige el sistema de pruebas por quién custodia el secreto** (tabla §7). Para GenoMatch/Scholarship el secreto es del usuario → proving en navegador → Circom/Noir, NO RISC Zero.
4. **Mockea con honestidad y dilo en el README** — el jurado lo premia explícitamente.
5. **Vertical slice estrecho** — un solo ensayo, un solo criterio de elegibilidad, un solo pago. Resiste la tentación de generalizar.
6. **Diseña para el pivote A↔B** — mismo verificador y andamiaje, distinta lógica de elegibilidad.

---

## 6. Apéndice — decisiones rápidas de stack (corregido tras §7)

**Primero pregunta: ¿quién custodia el secreto?** Eso decide el sistema de pruebas más que la complejidad de la lógica.

| Situación | Usa… | Por qué |
|---|---|---|
| **El consumidor** custodia el secreto y prueba en su dispositivo (genoma, finanzas, score, salud) | **Circom/Groth16** (snarkjs WASM) · o **Noir** (bb.js WASM) si la ergonomía pesa | Proving en navegador ✅. Groth16 = verificación on-chain más barata y probada (PoC Privacy Pools). |
| **Un operador de confianza** custodia datos de negocio y prueba en su servidor (cadena de frío, huella de carbono, reservas, agregados) | **RISC Zero** (Local/Bonsai) | Escribes Rust normal; el proving pesado va en servidor; la receipt se envuelve en Groth16 → verificación barata. |
| Range/membership simple, mínimo costo on-chain | **Circom/Groth16** | Circuitos pequeños, el camino más barato y battle-tested en Stellar. |
| Privacy pool / shielded transfer | **Fork de la PoC de Nethermind** | Ya trae pool + verificador Groth16 + ASP + proving en navegador. |

**Aprovecha la tecnología del sponsor:** construye commitments/Merkle con la host function **Poseidon** nativa y verifica con **`pairing_check` BN254** nativo (P25/P26). Es más barato Y le muestra al jurado que usas exactamente lo que el protocolo acaba de enviar.

**Regla de oro para los 10 días:** empieza por el camino que te deje *terminar y que verifique barato on-chain*. Para los casos de consumidor de este documento (los finalistas A y B), eso es **Circom/Groth16 con proving en navegador** — no RISC Zero.

---

## 7. Validación de viabilidad real (con evidencia, 2026-06-19)

Esta sección verifica las suposiciones técnicas con fuentes y corrige la v1 del documento. Es el corazón del "¿esto se puede construir en 10 días de verdad?".

### 7.1 Las ventajas de Stellar/ZK que esta propuesta explota
- **Host functions ZK nativas (P25 X-Ray + P26 Yardstick):** BN254 (`g1_add`, `g1_mul`, `pairing_check`; +MSM, aritmética de campo escalar y curve-membership en P26) y Poseidon/Poseidon2 (`poseidon`, `poseidon2`). Paridad con los precompiles EIP-196/197 de Ethereum → **interoperable con el tooling ZK existente** (Circom, snarkjs, Noir). [CAP-0074, CAP-0075]
- **Verificación on-chain barata** porque la matemática pesada (pairings, MSM) vive en el host, no en WASM interpretado. *Esta es la razón de ser del hackathon ahora.*
- **Movimiento de dinero trivial:** activos nativos + interfaz de token SEP-41 (USDC testnet) → el "pago" del demo es una llamada, no un ERC-20 a medida.
- **Finalidad rápida y barata (~5s)** → en el video, el payout se siente instantáneo.
- **Anchors / SEP-24** para la historia fiat on/off-ramp (relevante si algún día sale de testnet).
- **Edge de jurado:** usar Poseidon nativo para los commitments y `pairing_check` BN254 para verificar **exhibe justo lo que el equipo de protocolo acaba de enviar**. Pocos competidores lo harán explícito.

### 7.2 Hallazgos que CORRIGEN la v1
| # | Suposición v1 | Realidad validada | Impacto |
|---|---|---|---|
| 1 | "GenoMatch genera la prueba client-side con RISC Zero" | RISC Zero solo tiene **Local Prover** (hardware potente) o **Bonsai** (GPU remoto). **No prueba en navegador.** | ❌ Rompía "el genoma nunca sale del dispositivo". **Cambiado a Circom/Noir.** |
| 2 | "Noir/Circom solo para circuitos simples" | **bb.js (Noir) y snarkjs (Circom) prueban en navegador vía WASM, rápido** (RSA en ~0.2s; firma <3s en M1). La PoC de Privacy Pools ya lo hace en Stellar. | ✅ Habilita el patrón "secreto nunca sale del dispositivo". Es la pieza clave. |
| 3 | "RISC Zero por default, es más rápido de terminar" | Cierto solo cuando **un operador prueba en servidor**. Para secreto-del-consumidor es inviable. | 🔧 Default reescrito por "quién custodia el secreto" (§6). |
| 4 | (No evaluado) Costo de verificación on-chain | **UltraHonk (Noir) está "al borde" del límite de instrucciones**; un pairing con `ark_bn254` en software = **560M instr. vs límite 100M**. Lo salvan las host functions nativas BN254. **Groth16 (Circom) es mucho más barato.** | ⚠️ Riesgo #1. Spike obligatorio el Día 1. Groth16 es el camino seguro. |

### 7.3 Estado real de los verificadores (starter code)
- **Groth16 / Circom — el más maduro y barato.** `stellar/soroban-examples/groth16_verifier` + la **PoC de Privacy Pools de Nethermind** (Circom + Groth16 + proving en navegador WASM). Es el camino con más código de referencia funcionando y la verificación más barata. **Recomendado para los finalistas A y B.**
- **UltraHonk / Noir — funciona pero al borde del costo.** `indextree/ultrahonk_soroban_contract` y `yugocabrio/ultrahonk-rust-verifier`, actualizados a Nargo v1.0.0-beta.9 / bb v0.87.0 (oct-2025), migrando de `ark_bn254` (software, 560M instr.) a los **precompiles BN254 nativos**. Viable con P26, pero **mídelo antes de comprometerte**. Mejor ergonomía de escritura que Circom.
- **RISC Zero — para verifiable computation de operador.** Verificador Groth16 de Nethermind (`stellar-risc0-verifier`): la receipt STARK se envuelve en Groth16/BN254 → verificación barata. Proving pesado off-chain (servidor/Bonsai). **Ideal para ColdChain, CarbonProof, Solvency, Salary-Coop**, NO para secreto-del-consumidor.

### 7.4 Veredicto de viabilidad por finalista
| Finalista | ¿Viable en 10 días? | Stack correcto | Riesgo principal | Mitigación |
|---|---|---|---|---|
| **A · Scholarship** | ✅ **Alta** | Circom/Groth16, proving en navegador | Ninguno técnico serio (2 range proofs + nullifier + pago) | El reto es UX/historia, no cripto. Es el "floor" más seguro. |
| **B · GenoMatch** | ✅ **Buena** (con stack corregido) | Circom/Groth16 (o Noir), proving en navegador, Merkle Poseidon | Costo de verificación si eliges Noir; tamaño del circuito de membership | Spike de costo Día 1; mantén el árbol de genotipo pequeño; Groth16 si dudas |
| **C · Microinsurance** | 🟡 **Media** | RISC Zero (operador) o Circom; + oráculo | La pieza de oráculo climático + point-in-polygon añade alcance | Mockea el feed del oráculo honestamente; recorta a una sola región/trigger |

### 7.5 Conclusión de la validación
La propuesta **sigue en pie y es más fuerte**, con una corrección central: **para los casos donde el usuario custodia el secreto (los más ganadores: GenoMatch, Scholarship), el proving va en el navegador con Circom/Groth16 — no RISC Zero —, los commitments con Poseidon nativo y la verificación con BN254 nativo.** El único riesgo que puede hundir el proyecto es el costo de verificación on-chain; se neutraliza con (a) elegir Groth16 y (b) medirlo el Día 1 con una prueba dummy antes de escribir lógica. Todo lo demás es reuso de starter code + datos mock honestos.

### 7.6 Fuentes
- ZK Proofs on Stellar (docs): https://developers.stellar.org/docs/build/apps/zk
- Stellar X-Ray / Protocol 25: https://stellar.org/blog/developers/announcing-stellar-x-ray-protocol-25 · CAP-0074 (BN254), CAP-0075 (Poseidon)
- Yardstick / Protocol 26: https://stellar.org/blog/foundation-news/yardstick-stellar-protocol-26
- RISC Zero proving (Local vs Bonsai): https://dev.risczero.com/api/1.1/generating-proofs/remote-proving · https://dev.risczero.com/bonsai
- Noir en navegador (bb.js/WASM): https://noir-lang.org/docs/tutorials/noirjs_app · benchmarks de proving client-side
- Noir on Stellar E2E (costo "al borde"): https://jamesbachini.com/noir-on-stellar/
- UltraHonk Soroban verifier (estado + costo 560M instr.): https://github.com/indextree/ultrahonk_soroban_contract · https://hackmd.io/@indextree/rJPW3jU6lx
- Groth16 verifier / Privacy Pools PoC (proving en navegador): https://github.com/stellar/soroban-examples/tree/main/groth16_verifier · https://github.com/NethermindEth/stellar-private-payments
