# Semi-final — ZK como "cortafuegos de identidad" para IA diagnóstica

**Modelo:** Opus 4.8 · **Fecha:** 2026-06-19 · Análisis honesto + viabilidad del caso de uso "historial médico → ML/LLM diagnóstico, protegido por ZK".

> Tu frase clave: *"Lo que ofrezco no es la tecnología de LLM y ML diagnóstica, es el puente ZK que entrega la info y salvaguarda la identidad."* Este documento toma esa frase en serio, corrige un error de categoría sobre qué hace ZK, y convierte la visión en algo construible en 10 días.

---

## 1. Veredicto honesto en 30 segundos

- **La motivación: 10/10.** Proteger tu historial médico de aseguradoras/isapres que pueden excluirte o subir tu prima es un problema real, caro y con carga emocional. Historia de jurado de primer nivel.
- **La arquitectura tal como la planteaste: tiene un error de categoría.** "Que el modelo se alimente de tu historial sin conocerlo, gracias a ZK" mezcla dos cosas. **ZK prueba afirmaciones sobre datos; no permite computar sobre datos ocultos.** Ocultar el dato *de la máquina que lo procesa* es FHE o TEE, no ZK.
- **El reencuadre que lo salva (y que ya estás insinuando):** ZK no es el motor del diagnóstico — es **el cortafuegos de identidad**. Su trabajo load-bearing es que un paciente *verificado, único y con datos auténticos* interactúe con el motor diagnóstico como un **seudónimo des-linkable**, de modo que demografía e identidad **nunca** puedan unirse al historial ni al diagnóstico.
- **Para el hackathon:** la visión completa (ML potente + LLM + ZK + diagnóstico) es demasiado. Pero **el slice del cortafuegos ZK es construible** y es, literalmente, lo que dijiste que ofreces. Mockea el motor diagnóstico; el ZK es lo real.

---

## 2. La idea, tal como la planteaste

Un usuario tiene su historial médico. Ese historial alimenta un modelo de ML potente + reglas determinísticas + LLMs, que devuelven un diagnóstico (o un abanico diagnóstico acotado con alto % de certeza). Un médico general puede entonces derivar o no a un especialista, rápido y simple. El modelo **jamás** conoce la identidad del paciente (protegida por ZK) y **jamás** podrá linkear su demografía con su historial. Esa información queda a salvo de aseguradoras/isapres que podrían dejarlo fuera o subirle la prima.

**El valor que ofreces:** no el ML/LLM diagnóstico — **el puente ZK que entrega la info y salvaguarda la identidad.**

---

## 3. La corrección de categoría: qué hace y qué NO hace ZK

Este es el punto más importante del documento. Cada tecnología de privacidad resuelve un problema distinto:

| Tecnología | Qué hace | ¿Sirve para "el modelo lee mi historial sin verlo"? |
|---|---|---|
| **ZK (zero-knowledge)** | Probar una **afirmación sobre datos** sin revelar los datos ("tengo esta credencial", "este cómputo se hizo bien", "soy elegible/único/auténtico") | ❌ No. ZK no oculta el dato de quien lo procesa. |
| **FHE (homomorphic encryption)** | **Computar sobre datos cifrados** sin descifrarlos | ✅ Sí, pero inviable para ML potente + LLM hoy (lentísimo). |
| **TEE (enclave seguro, SGX/Nitro)** | Procesar datos en una **caja de hardware** que ni el operador ve | ✅ Sí, pero requiere confiar en el hardware/fabricante. |
| **MPC (multi-party computation)** | Varios cómputan un resultado **sin compartir sus inputs** | ✅ Parcial, pesado para inferencia grande. |
| **De-identificación / seudonimización** | Quitar identificadores del dato | ⚠️ Re-identificable; no prueba autenticidad ni unicidad. |

**Conclusión:** si tu objetivo fuera que un servidor corra el modelo sobre datos que ni el servidor ve, necesitas FHE/TEE — y para "ML potente + LLM" eso es ciencia-ficción a 10 días. **Pero ese no es realmente tu objetivo.** Tu objetivo es que **nadie pueda unir el historial con tu identidad**. Para eso, ZK es la herramienta correcta — en la capa de identidad, no en la capa de cómputo.

> Regla mental: **ZK protege el *vínculo* (identidad ↔ datos), no oculta el *dato* del que computa.** Tu pitch ("salvaguarda la identidad") es exactamente la capa-vínculo. Estás en lo correcto; solo hay que ubicar ZK donde de verdad trabaja.

---

## 4. El reencuadre correcto: ZK como cortafuegos de identidad

El motor diagnóstico (ML/LLM) hace su trabajo sobre los datos clínicos. ZK hace **cinco** trabajos reales, todos en la capa de identidad/integridad, y todos load-bearing:

1. **Acceso seudónimo verificado (anti-Sybil).** El paciente prueba que es un humano real, único y elegible (afiliado al sistema de salud, mayor de edad, en jurisdicción) **sin revelar quién es**, vía un **nullifier**. El servicio obtiene un usuario "verificado pero anónimo" → no hay identidad que filtrar ni que subpoena.
2. **Autenticidad del historial sin identidad.** El paciente prueba que su registro está **firmado por un prestador/laboratorio real** (commitment + membership Poseidon) — el modelo confía en que el dato es genuino y no manipulado, **sin saber de quién es**. Anti-fraude + anti-tampering.
3. **Des-linkabilidad demográfica.** El payload clínico se entrega bajo el seudónimo, **sin un solo campo demográfico**. ZK garantiza que el seudónimo no puede unirse a la demografía → aunque filtren la base, no hay llave de re-identificación.
4. **Recibos de consentimiento verificables.** ZK prueba que el dato se usó **solo dentro del alcance consentido** (p.ej. "diagnóstico, no comercial, no compartible con aseguradoras").
5. **Compuerta de derivación/pago en Stellar.** El resultado (elegible para derivación / consulta pagada) dispara una acción en Stellar bajo el seudónimo, sin exponer identidad.

**Esto es exactamente "el puente que entrega la info y salvaguarda la identidad".** El ML no es tu aporte (lo dijiste); el cortafuegos ZK sí, y es genuinamente difícil y valioso.

---

## 5. ¿Dónde corre el modelo? Las tres arquitecturas

La pregunta que decide la honestidad técnica de todo el proyecto:

| Arquitectura | Cómo protege el dato | Rol de ZK | Veredicto |
|---|---|---|---|
| **A. Inferencia en el borde (cliente)** | El modelo corre en el dispositivo/dominio de confianza del paciente; el dato nunca sale | ZK prueba a terceros (médico, registro, Stellar) propiedades del resultado y de la elegibilidad | ✅ El encaje ZK más limpio y honesto. Limitación: modelos locales son menos potentes (pero los on-device LLM avanzan rápido) |
| **B. Servidor con TEE/FHE** | Enclave/cifrado oculta el dato del operador | ZK certifica identidad/elegibilidad/procedencia; **no** oculta el dato | ⚠️ Honesto pero añade dependencia de hardware/FHE. ZK es complementario, no el mecanismo de ocultamiento |
| **C. Seudonimización + cortafuegos ZK** | El servidor recibe datos clínicos **des-identificados** atados a un seudónimo ZK; nunca recibe demografía | ZK garantiza: seudónimo = humano real/único, dato auténtico, cero linkage demográfico | ✅ **La opción pragmática y construible.** La protección anti-aseguradora viene de la des-linkabilidad, no de ocultar el dato al cómputo |

**Recomendación: A o C.** Para el hackathon, **C con el motor mockeado** es lo más demostrable. Sé explícito en el README: ZK provee la identidad des-linkable + autenticidad; la protección anti-isapre es la *imposibilidad de re-identificar*, no el ocultar el dato al modelo.

> Honestidad de alcance: el cortafuegos protege la **capa de datos/plataforma** (filtraciones, subpoenas, entrenamiento de modelos, registros on-chain). No protege el encuentro clínico presencial, donde el médico te ve la cara. El valor real: **tu historial longitudinal puede alimentar modelos para siempre sin convertirse jamás en un dataset identificado explotable por una aseguradora.**

---

## 6. La amenaza real (isapre/aseguradora) y por qué ZK la neutraliza

**Modelo de amenaza:** la aseguradora quiere aprender el riesgo de salud de un individuo identificado para negar cobertura o subir prima. Su munición: bases de datos donde *identidad ↔ condición clínica* están unidas.

**Por qué la de-identificación sola no basta:** es re-identificable (cruces demográficos), y no prueba ni autenticidad ni unicidad — un atacante puede inyectar datos falsos o crear identidades múltiples.

**Lo que solo ZK da:** *verificado-pero-des-linkable*. El paciente prueba "soy un afiliado real y único y mi registro es auténtico" entregando **cero** identidad. El vínculo identidad↔clínica se corta **criptográficamente**, no por política. Aunque filtren el servidor o lo subpoeneen, no hay llave de re-identificación que entregar.

Ese "verificado pero des-linkable" es imposible de lograr con cualquier otra herramienta. **Ahí es donde ZK es load-bearing.**

---

## 7. El slice construible para el hackathon — "ZK-MedBridge"

Recorta brutalmente. Construye el cortafuegos; mockea el cerebro.

**Flujo (vertical slice):**
1. **Setup (mock honesto):** un "prestador/laboratorio" firma un mini-registro clínico del paciente (unos pocos campos tipo FHIR) y lo compromete en un **árbol de Merkle con Poseidon** (host function nativa de Stellar). La raíz firmada es la attestation pública.
2. **Prueba client-side (en el navegador, WASM):** el paciente genera un circuito **Circom/Groth16 (o Noir)** que prueba:
   - *membership:* "mi registro pertenece al árbol firmado por un prestador válido" (autenticidad),
   - *unicidad:* `nullifier = Poseidon(secreto, periodo)` (anti-Sybil, sin identidad),
   - *elegibilidad opcional:* un flag determinístico simple (p.ej. "el registro contiene marcadores que ameritan derivación") como *demostración* del gating — sin revelar los datos.
3. **Verificación + seudónimo en Stellar:** un contrato Soroban verifica la prueba con `pairing_check` BN254 nativo, registra el nullifier (anti doble-uso) y emite un **"consult token" seudónimo**.
4. **Motor diagnóstico (MOCKEADO):** un stub recibe **solo** el payload clínico des-identificado bajo el seudónimo y devuelve un diagnóstico/abanico. *Deja clarísimo en el README que el ML/LLM es un stub — ese no es tu aporte.*
5. **Acción real en Stellar:** el resultado dispara una **derivación** o una **microconsulta pagada en USDC** (SEP-41), todo bajo el seudónimo.
6. **El momento "wow":** demuestra que, con acceso total a la cadena Y al servidor, **una aseguradora no puede unir el dato clínico con el paciente.** El vínculo está roto.

**Qué es ZK real vs qué se mockea:**
- **Real:** circuito de autenticidad + nullifier + des-linkabilidad, proving en navegador, verificación on-chain, emisión de seudónimo, pago.
- **Mock honesto:** la firma del prestador, el contenido del registro, y el motor ML/LLM (un stub).

---

## 8. El test de "ZK load-bearing" — ¿pasa?

¿Se rompe el proyecto si quitas ZK? **Sí, completamente.** Sin ZK no puedes tener simultáneamente:
- un paciente **verificado, real y único** (para que el diagnóstico sea confiable y no haya fraude/Sybil), **y**
- **cero linkage** entre su identidad/demografía y su historial (para protegerlo de la aseguradora).

La de-identificación clásica te da lo segundo pero no lo primero (y es re-identificable). Login/KYC te da lo primero pero destruye lo segundo. **Solo ZK da ambos a la vez.** Pasa el test con holgura.

---

## 9. Viabilidad y riesgos

| Riesgo | Severidad | Mitigación |
|---|---|---|
| **Confusión ZK vs FHE/TEE** (jurado técnico pregunta "¿pero el servidor no ve el dato?") | Alta si no la abordas | Adelántate: el README y el video explican que ZK protege el *vínculo*, no oculta el dato al cómputo; la protección es la des-linkabilidad |
| **zkML del modelo potente** (probar la inferencia del LLM en ZK) | Fatal si lo intentas | **No lo intentes.** Probar inferencia de un LLM en ZK es inviable hoy. El modelo es un stub; ZK vive en la identidad |
| **Alcance ("demasiado para el hack")** | Alta | Construye solo el cortafuegos (membership + nullifier + des-linkabilidad + pago). Es comparable en dificultad a ZK-GenoMatch |
| **Costo de verificación on-chain** | Media | Groth16 (más barato) en vez de UltraHonk; mídelo el Día 1 con prueba dummy |
| **El diagnóstico en sí es sensible** | Media | No lo pongas on-chain; cífralo al paciente. On-chain solo va el seudónimo + verificación + pago |
| **"¿Por qué Stellar?"** | Baja | La derivación/microconsulta se liquida en USDC; Stellar = dinero real + Poseidon/BN254 nativos |

**Veredicto de viabilidad:** el slice del cortafuegos es **construible (viabilidad buena/media)**, del mismo orden de dificultad que ZK-GenoMatch. La visión completa no, y no hace falta: el slice ya *demuestra* el aporte que dijiste ofrecer.

---

## 10. Cómo encaja vs GenoMatch / Scholarship

Las tres comparten la **misma columna vertebral**: `credencial/dato privado y auténtico → prueba ZK de propiedad/elegibilidad sin identidad → acción/dinero en Stellar`.

- **ZK-MedBridge** es el más ambicioso y el de **mayor carga emocional/narrativa** (la historia isapre es brutal), pero arrastra el riesgo de la confusión ZK/FHE que debes desactivar explícitamente.
- **ZK-GenoMatch** es un caso más acotado del **mismo patrón** (genoma en vez de historial completo) y más fácil de mantener "honesto" porque el criterio es un match discreto, no "un modelo lee todo tu historial".
- **ZK-Scholarship** es el piso más seguro.

**Mi recomendación honesta:** si la historia médica/isapre es la que te enciende y la que vas a demostrar con credibilidad, **constrúyela — pero como cortafuegos de identidad con el motor mockeado**, no como "ZK que corre el modelo". Si quieres minimizar el riesgo de la pregunta técnica incómoda, **GenoMatch es el mismo corazón con menos superficie de ataque conceptual.** Ambas usan idéntico andamiaje (Poseidon + Groth16 + nullifier + pago), así que **puedes decidir en el Día 3 sin perder trabajo.**

---

## 11. Guion del demo — la frase que gana

> *"En Chile, tu historial médico puede costarte tu isapre: una preexistencia y te suben la prima o te dejan fuera. ¿Y si tu historial pudiera alimentar a la mejor IA diagnóstica del mundo... sin que nadie pueda jamás unir ese historial con tu nombre? ZK-MedBridge no construye la IA — construye el **cortafuegos criptográfico**: pruebas que eres un paciente real, único y con datos auténticos, entregando **cero** identidad. El diagnóstico fluye; tu identidad queda matemáticamente desconectada. Ni una filtración ni una aseguradora pueden volver a unirlas. Eso es ZK haciendo trabajo real sobre algo que importa: tu derecho a no ser castigado por estar enfermo."*

---

### Apéndice — la única diapositiva técnica que el jurado debe entender
```
        TU DISPOSITIVO                         STELLAR / SERVIDOR
  ┌───────────────────────┐            ┌──────────────────────────────┐
  │ Historial firmado por │            │ Contrato Soroban:            │
  │ el prestador (mock)   │            │  - verifica prueba (BN254)   │
  │         │             │  prueba ZK │  - registra nullifier        │
  │   [Circom/Groth16]    │ ─────────► │  - emite seudónimo + pago    │
  │  membership Poseidon  │  (sin      │                              │
  │  + nullifier          │  identidad)│ Motor ML/LLM (STUB):         │
  │  + des-linkabilidad   │            │  recibe payload des-identif. │
  └───────────────────────┘            │  → diagnóstico al seudónimo  │
   El dato y la identidad               └──────────────────────────────┘
   nunca salen juntos.            La aseguradora NUNCA puede unir
                                  identidad ↔ historial. Vínculo roto.
```
**ZK = el corte de la flecha identidad↔datos. No es el cerebro; es el cortafuegos.**
