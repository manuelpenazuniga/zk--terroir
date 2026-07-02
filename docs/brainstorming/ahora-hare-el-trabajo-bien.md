# Ahora haré el trabajo bien

**Modelo:** Opus 4.8 · **Fecha:** 2026-06-25 · **Deadline:** 2026-06-29 12:00 PST (~4 días)

> 7 ideas de alta calidad, filtradas con criterio de premio de innovación: **máximo ratio valor / complejidad operativa**, dolor cotidiano real, sin rocket science, donde **ZK y especialmente las tecnologías nuevas de Stellar** hacen trabajo load-bearing. No buscamos "dónde aplicar ZK"; buscamos **dónde un dolor diario se resuelve mejor con ZK + Stellar que con cualquier otra cosa.**

---

## 0. El filtro (por qué descartamos casi todo lo anterior)

Las 35 ideas de los brainstorming previos fallan, en su mayoría, el mismo test: **complejidad operativa alta sin agregación de valor proporcional.** zkML de un modelo real, FHE/TEE, parsear un genoma o una imagen satelital client-side, oráculos climáticos, cadena de frío con sensores IoT reales — son técnicamente seductoras y comercialmente huecas para un MVP de 4 días. El jurado lo huele.

Las que **sí** pasan comparten una firma simple y poderosa:

```
credencial/dato auténtico y privado
   → prueba ZK barata (membership Poseidon · nullifier · range/ratio)
   → acción o dinero real en Stellar (USDC/EURC SEP-41)
```

Es el mismo esqueleto para las 7 (el #7 lo extiende con agregación para escalar). Eso significa que **construyes el núcleo una vez** (commitment Poseidon + verificador Groth16 + nullifier + pago SEP-41 + Passkey) y la "personalidad" de cada idea es solo la lógica de elegibilidad. Bajísima complejidad, máxima opcionalidad.

### Lo que la convocatoria pide entre líneas (y casi nadie va a hacer)

| Palanca nueva de Stellar | Por qué la quieren ver | Cómo la exhibimos |
|---|---|---|
| **Poseidon nativo (P25)** | Es la primitiva que acaban de enviar; hace los commitments/Merkle baratos | Todos los commitments y árboles usan `poseidon`/`poseidon2` del host, no un hash en WASM |
| **BN254 nativo (P26: MSM, campo escalar)** | Es la razón de lanzar el hack *ahora*; abarata la verificación | Verificación Groth16 on-chain vía `pairing_check`/MSM nativos |
| **Privacidad cumplidora (ASP)** | Criterio explícito de juzgamiento de la SDF | "Pertenezco a un set limpio/certificado" dentro del circuito |
| **Passkeys (Secp256r1)** | UX sin seed phrase = adopción real | Autorizas la prueba con FaceID/TouchID; smart wallet en Soroban |
| **Confidential Token (estándar SDF/Nethermind/OZ/Zama)** | Es lo más nuevo del stack de privacidad | Saldos/montos cifrados con divulgación selectiva a un auditor |
| **USDC/EURC + SEP-41** | Stellar = dinero real | Cada idea **mueve dinero o libera valor**, no emite un badge decorativo |

**Regla de oro:** cada una de las 7 ideas exhibe *una palanca distinta* como protagonista. Así el portafolio cubre todo lo que el jurado quiere ver, y la idea que elijas ya viene con su "guiño técnico" incorporado.

---

## 1. 🌱 ZK-Terroir — *prueba de origen justo sin entregar tu cadena de suministro*

**Pitch:** Una marca de café/cacao/vino prueba que un lote es **single-origin y fair-trade certificado en cada eslabón**, sin revelar su lista de proveedores (su activo competitivo), y el **premium fair-trade se paga automáticamente en USDC a la cooperativa** en el país de origen.

- **El dolor cotidiano:** el fraude "fair-trade/orgánico" y el greenwashing son endémicos. El comprador no puede verificar la cadena; el productor de mercado emergente cobra tarde, mal y con comisiones. La marca **no puede** publicar su lista de proveedores porque es lo único que la diferencia.
- **Por qué ZK es load-bearing:** sin ZK tienes que elegir entre *transparencia* (revelas proveedores → pierdes ventaja) u *opacidad* (nadie te cree). ZK rompe el dilema: pruebas que **cada eslabón pertenece al set de certificados** (membership Merkle-Poseidon) sin nombrar a ninguno. Quita la privacidad y el producto no existe.
- **Palanca nueva que exhibe:** **Poseidon nativo** (árbol de certificaciones) + **ASP** (el "set limpio" = certificadores acreditados) + **USDC cross-border**, el caso insignia de Stellar.
- **Dinero real:** prueba válida → el contrato Soroban libera el premium en USDC directo a la wallet de la cooperativa. Cero intermediarios, finalidad en ~5s.
- **Stack mínimo:** circuito Circom/Groth16 (membership Poseidon + cadena de N eslabones); verificador Groth16 (`soroban-examples`); pago SEP-41. **Mock honesto:** la firma del certificador y los lotes; **real:** circuito, verificación on-chain, pago.
- **Ratio valor/complejidad:** ⭐⭐⭐⭐⭐ — es un solo membership proof encadenado. Una tarde de circuito, todo el resto es UX y narrativa.
- **Gancho comercial:** ESG verificable es un mercado en explosión; las marcas *pagan* por probar sostenibilidad sin filtrar proveedores. Stellar ya mueve remesas a esos corredores.

---

## 2. 🍞 ZK-FoodWaste-Tax — *prueba tu donación de excedente sin abrir tus libros*

**Pitch:** Un supermercado prueba que donó **≥X% de su excedente diario** a bancos de alimentos para reclamar el beneficio fiscal/ESG, **sin exponer volúmenes de venta ni inventario** (datos que la competencia mata por tener), y recibe un **crédito de cumplimiento verificable on-chain**, opcionalmente con un incentivo igualado en USDC.

- **El dolor cotidiano:** muchas jurisdicciones premian fiscalmente donar excedente alimentario (Francia incluso lo obliga), pero auditar el ratio exige abrir cifras de venta/merma — inteligencia comercial que ninguna cadena entrega. Resultado: comida a la basura porque "probarlo" cuesta más que tirarla.
- **Por qué ZK es load-bearing:** el auditor/fisco solo necesita el **veredicto** `donaciones / merma ≥ umbral`. ZK prueba ese *ratio* sin revelar numerador ni denominador. Sin ZK, o no donan (para no abrir libros) o donan sin poder reclamar el beneficio.
- **Palanca nueva que exhibe:** **range/ratio proof** + **verificación barata BN254 (P26)** + **atestación de cumplimiento** que vale dinero (no un badge decorativo): habilita el crédito fiscal y/o un *match* de un fondo en USDC.
- **Dinero real:** la prueba libera un **incentivo igualado en USDC/EURC** desde un fondo de impacto o municipal, y emite el recibo fiscal verificable. Donar deja de ser un costo hundido.
- **Stack mínimo:** circuito de ratio sobre recibos de donación firmados (banco de alimentos) y log de merma comprometido; Groth16 + SEP-41. **Mock honesto:** las firmas del banco de alimentos y los logs.
- **Ratio valor/complejidad:** ⭐⭐⭐⭐⭐ — un range proof sobre una división. Trivial de construir, enorme de contar.
- **Gancho comercial:** retail + govtech + ESG. Cada municipio con incentivo de donación es un cliente; el banco de alimentos gana trazabilidad; la cadena gana deducción sin filtrar datos.

---

## 3. 🗣️ VeritasVoice — *feedback anónimo verificable + agente que anonimiza el texto*

**Pitch:** Un cliente o empleado **verificado** deja feedback que es **demostrablemente de un miembro real del grupo** (nullifier = uno por período, anti-spam/anti-Sybil) **sin revelar quién**; y un **agente de IA reescribe el texto** para borrar la huella estilística y los datos que delatan al autor — de modo que ni el *contenido* permita re-identificarlo. Opcional: micro-recompensa en USDC por feedback verificado.

- **El dolor cotidiano:** las reseñas y encuestas internas viven la tensión imposible — *anónimo* (nadie confía: ¿es bot, competencia, ex-empleado resentido?) vs *verificado* (nadie habla: represalias del jefe, del restaurante, del proveedor). Glassdoor, encuestas de clima, NPS: todos cojean por esto.
- **Por qué ZK es load-bearing:** ZK da lo único que nadie más da — **verificado pero des-linkable**. Pruebas "soy empleado/cliente real y único" entregando **cero** identidad; el nullifier impide reseñas falsas masivas sin doxearte. Login mata el anonimato; anonimato puro mata la confianza. Solo ZK da ambos.
- **La pieza que lo eleva (tu idea, y es brillante):** ZK protege la *identidad criptográfica*, pero el **texto libre filtra al autor** (estilo, jerga, un detalle que solo tú sabes). Un **agente de IA anonimizador** normaliza tono, parafrasea y elimina PII/stylometry **antes** de publicar — cierra el agujero de re-identificación que ZK por sí solo no toca. Es la diferenciación que casi nadie tendrá: **ZK para la identidad, IA para el contenido.**
- **Palanca nueva que exhibe:** **nullifier Poseidon** (un voto/reseña por período) + **ASP** (el "grupo limpio" = nómina/clientes verificados) + **Passkeys** (firmas con FaceID, sin seed phrase; momento "wow" del demo).
- **Dinero/acción real:** micro-bounty en USDC por feedback verificado, o desbloqueo de una encuesta de clima cuyos resultados agregados se anclan on-chain. La verificación es lo que hace el pago no-falsificable.
- **Stack mínimo:** membership + nullifier en Circom/Groth16; agente anonimizador = LLM (Claude) con un prompt de des-identificación estilística; Passkeys vía Stellar Wallets Kit. **Mock honesto:** el directorio de empleados/clientes firmado.
- **Ratio valor/complejidad:** ⭐⭐⭐⭐ — circuito estándar (membership+nullifier) + una llamada a un LLM. La complejidad es de producto, no criptográfica.
- **Gancho comercial:** RRHH (clima laboral sin represalias), marketplaces (reseñas anti-fake verificadas), gobernanza comunitaria. Literalmente listado en el tier "mild" de la convocatoria → bajo riesgo de "no encaja".

---

## 4. 🎫 ZK-ProofOfRight — *desbloquea tu descuento/subsidio sin revelar por qué calificas*

**Pitch:** Una persona prueba que **tiene derecho a un precio subsidiado** (estudiante, adulto mayor, zona rural, bajos ingresos, discapacidad) **sin revelar su identidad ni la condición sensible**, y el **descuento o subsidio se liquida en Stellar** — con divulgación selectiva opcional para que el ente que financia audite el total sin ver quién.

- **El dolor cotidiano:** para acceder a tarifas sociales (transporte, farmacia, datos móviles, matrícula) tienes que **exhibir tu pobreza/edad/diagnóstico** en una ventanilla o un formulario. Es estigmatizante, se filtra, y el comercio sufre fraude de quien no califica. Ocurre millones de veces al día.
- **Por qué ZK es load-bearing:** el comercio solo necesita "¿califica? sí/no". ZK prueba `ingreso < umbral` **o** "porto una credencial válida de estudiante/discapacidad" (membership) **sin revelar el número ni cuál**. Sin ZK: o revelas todo (estigma + fuga de datos) o hay fraude. La privacidad es el producto.
- **Palanca nueva que exhibe:** **range/membership proof** + **divulgación selectiva con view-key** (el patrón exacto que la convocatoria dice que es la estrategia de privacidad de Stellar: el financiador reconstruye el agregado, no la identidad) + **USDC/EURC**.
- **Dinero real:** prueba válida → el contrato aplica el descuento pagando la diferencia desde el fondo de subsidio en USDC, o emite un voucher gastable. El subsidio llega al precio, no a un trámite.
- **Stack mínimo:** dos primitivas que ya dominas (range proof sobre credencial firmada + nullifier anti doble-cobro); Groth16 + SEP-41. **Mock honesto:** la credencial firmada del emisor (universidad/registro civil/banco).
- **Ratio valor/complejidad:** ⭐⭐⭐⭐⭐ — es "Scholarship" generalizado a *cualquier* subsidio cotidiano. El circuito más simple del lote, la superficie comercial más amplia.
- **Gancho comercial:** govtech + retail + fintech de inclusión. Todo programa de tarifa social del planeta es un cliente; el ente público reduce fraude y elimina su responsabilidad sobre datos sensibles (no los almacena nunca).

---

## 5. 🗳️ ZK-FairVote — *presupuesto participativo / decisión comunitaria, un humano un voto, sin revelar quién*

**Pitch:** Los miembros de una comunidad, cooperativa o barrio prueban que **son elegibles y únicos** (nullifier = un voto por persona, anti-Sybil) y votan **sin que su voto sea linkable a su identidad**; al cerrar, **el presupuesto del proyecto ganador se desembolsa en USDC** automáticamente.

- **El dolor cotidiano:** las votaciones comunitarias y los presupuestos participativos se rompen por dos cosas — **relleno de urnas** (Sybil: una persona, mil cuentas) y **coerción/represalia** (si tu voto es rastreable, votas con miedo). Y cuando el proyecto gana, el dinero tarda meses o se desvía.
- **Por qué ZK es load-bearing:** necesitas simultáneamente **anti-Sybil** (que cada humano elegible vote una sola vez) y **anonimato** (que el voto no se ate a la cara). Login da lo primero, urna de papel da lo segundo; **solo el nullifier ZK da ambos**. Y el desembolso automático corta el desvío.
- **Palanca nueva que exhibe:** **nullifier Poseidon** (unicidad sin identidad) + **ASP** (el padrón = set elegible certificado) + **desembolso real en USDC** atado al resultado.
- **Dinero real:** el resultado **mueve el presupuesto**: el contrato Soroban libera los fondos del proyecto ganador a su wallet. La votación no es simbólica; es la llave del dinero.
- **Stack mínimo:** membership en padrón + nullifier + conteo; Groth16 + SEP-41. **Mock honesto:** el padrón firmado por la junta/municipio.
- **Ratio valor/complejidad:** ⭐⭐⭐⭐ — membership + nullifier + un contador. La pieza de gobernanza es producto, no cripto exótica.
- **Gancho comercial:** govtech (presupuestos participativos municipales), cooperativas, DAOs del mundo real, juntas de vecinos. Encaja de lleno con tu premio de **innovación social** y con la "privacidad cumplidora" que la SDF premia.

---

## 6. 💸 ZK-ConfidentialPayroll — *nómina en stablecoin con sueldos privados y auditoría demostrable*

**Pitch:** Una empresa o cooperativa paga a su equipo en USDC/EURC donde **cada sueldo individual queda confidencial on-chain**, pero la empresa puede **probarle a un auditor (o al fisco) los totales y rangos** mediante una **view-key** — sin exponer la planilla persona por persona.

- **El dolor cotidiano:** pagar en cripto es transparente *de más* — todo el mundo ve quién gana cuánto, lo que genera conflicto interno, fuga de talento y riesgo de seguridad (saber el sueldo de alguien = blanco de extorsión). Por eso casi nadie hace nómina on-chain, pese a que el rail es perfecto para equipos remotos/transfronterizos. Es un dolor *mensual*.
- **Por qué ZK es load-bearing:** necesitas privacidad (sueldos ocultos) **y** auditabilidad (el contador/fisco verifica totales). Esa combinación — confidencial al público, demostrable al auditor — es exactamente **divulgación selectiva**. Sin ella, o es público (inusable) o es opaco (no auditable).
- **Palanca nueva que exhibe:** el **estándar Confidential Token** (lo más nuevo del stack: SDF + Nethermind + OZ + Zama) para los montos cifrados + **ZK + view-key** para probar `Σ sueldos = total` y `cada sueldo ∈ rango legal` sin revelarlos. Es la idea que muestra que leíste **toda** la pestaña de privacidad.
- **Dinero real:** es nómina — el dinero **es** el producto. Pagos reales en USDC/EURC a wallets de empleados, con prueba de totales para compliance.
- **Stack mínimo:** confidential token (interfaz del estándar) + prueba de suma/rango con view-key; Groth16. **Mock honesto:** la planilla de entrada y la identidad del "auditor".
- **Ratio valor/complejidad:** ⭐⭐⭐ — la más ambiciosa de las 6 (el confidential token añade una pieza), pero la de **mayor viabilidad comercial inmediata** y la que mejor exhibe el estándar más nuevo. Si quieres impresionar a un jurado técnico de la SDF, esta es la bala.
- **Gancho comercial:** toda empresa con equipo remoto/transfronterizo (el mercado donde Stellar ya gana). B2B claro, recurrente, con disposición a pagar.

---

## 7. 🗳️ ZK-Quorum — *urna secreta institucional: ID verificada, voto anónimo, escala sin caerse, auditable para siempre*

**Pitch:** Empresas, sindicatos, comunidades y municipios corren elecciones **secretas, verificables y auditables**: cada votante prueba que es un miembro registrado (**ID verificada**) **sin revelar quién es**, emite **un solo voto** (nullifier), y el sistema **no se cae con cientos de miles de votantes** porque la avalancha descansa en el ledger de alta capacidad de Stellar — y cualquiera puede **re-auditar la elección entera después**, sin comprometer jamás el secreto del voto.

- **El dolor cotidiano:** la votación institucional está rota por los dos extremos. En papel: cara, lenta, y siempre hay un perdedor que grita "fraude" sin poder probarlo ni desmentirlo. Electrónica centralizada: "confía en el servidor del administrador" — una caja negra que nadie audita. Los **sindicatos** necesitan *por ley* voto secreto para elegir directiva o aprobar una huelga; las **empresas**, para juntas de accionistas y comités; los **municipios**, para referendos. Hoy se paga a notarios y firmas administradoras precisamente porque **nadie confía en la otra parte**.
- **Por qué ZK es load-bearing:** se necesitan **cuatro cosas a la vez que ninguna otra herramienta da juntas** — (1) *solo miembros elegibles votan* (membership), (2) *uno por cabeza* (nullifier, anti-relleno de urnas), (3) *el voto no es linkable a la persona* (secreto), y (4) *el conteo es demostrablemente correcto* sin abrir los votos (prueba de tally). Login da 1 y 2 pero mata 3. El papel da 3 pero no 1, 2 ni 4 demostrables. **Solo ZK da las cuatro.** Quita la privacidad y es vigilancia; quita la prueba y es una caja negra.
- **La arquitectura que NO se cae (el corazón de tu pedido):** la trampa ingenua es *verificar un SNARK por votante on-chain* — eso colapsa con volumen. El truco IQ-190 es **no verificar una prueba por voto, sino commitments baratos por voto y UNA sola prueba para todo el conteo.** La avalancha de votantes se absorbe en la **capa clásica de Stellar**, hecha justo para esto: miles de operaciones baratas por segundo, finalidad en ~5s. Cada votante solo escribe un **commitment + nullifier** (un write minúsculo). Al cierre, un agregador produce **una prueba Groth16 (con agregación/recursión)** de que "N votos válidos de N votantes elegibles distintos se contaron bien y el resultado es T", y **Soroban la verifica UNA vez** con BN254 nativo. Costo on-chain por votante = O(1) barato; costo criptográfico pesado = amortizado en una sola prueba final. **El blockchain sostiene el sistema** porque Stellar es el sustrato correcto para "muchos escritos pequeños + una verificación pesada".
- **Auditable para siempre:** todo (raíz del padrón, commitments, set de nullifiers, prueba de tally) queda en el ledger append-only y público. Tras la elección, **cualquiera descarga los datos y re-verifica matemáticamente** que nadie votó dos veces, que todos eran elegibles y que el conteo cuadra — **sin aprender jamás quién votó qué**. El perdedor ya no grita fraude: o lo prueba con matemática, o se calla.
- **Palanca nueva que exhibe:** **agregación / pruebas recursivas** (el tier "wild" de la convocatoria, que casi nadie tocará) + **el ledger de alto throughput de Stellar como substrato escalable** + nullifier Poseidon + ASP (padrón certificado) + Passkeys para emitir el voto con biometría.
- **Dinero/acción real:** modelo SaaS — las organizaciones **pagan por elección en USDC**; el resultado certificado gatilla la acción gobernada (publica el veredicto, habilita la directiva electa, valida el mandato). Stellar es load-bearing como **infraestructura** (throughput + auditoría), no solo como rail de pago.
- **Stack mínimo:** circuito membership + nullifier + voto comprometido (Circom/Groth16); contrato Soroban que registra commitments/nullifiers y verifica **una** prueba de tally al cierre; padrón Merkle-Poseidon firmado por la org. **MVP honesto de 4 días:** *batch verification* (verificar k pruebas en una llamada) en vez de recursión completa; demuestra la escala con un lote grande y documenta la recursión como roadmap. **Mock honesto:** el padrón firmado y el proveedor de ID.
- **Ratio valor/complejidad:** ⭐⭐⭐ — la más ambiciosa del portafolio junto con Payroll (la agregación añade trabajo), pero es **exactamente** la pieza que la convocatoria pone en su tier más alto, y la que convierte "otra app de voto" en "infraestructura electoral seria". Recórtala con batching honesto y es entregable.
- **Gancho comercial:** B2B/B2G de alto valor — **sindicatos** (voto secreto legalmente obligatorio para directiva/huelga), **empresas** (juntas de accionistas, comités de empresa), **municipios y comunidades** (referendos, juntas de vecinos). Reemplaza notarios y firmas administradoras caras con garantías criptográficas; al que pierde no le queda dónde agarrarse para impugnar.

**Diferencia con #5 (ZK-FairVote):** FairVote es *presupuesto participativo* donde el punto es **el dinero que sale** al proyecto ganador. ZK-Quorum es *urna institucional* donde el punto es **la integridad, el secreto y la escala del voto mismo** (sindicatos/empresas/municipios), auditable y resistente a volumen. Comparten el núcleo (membership + nullifier), pero #7 añade la capa de **agregación para escalar** y **auditoría permanente**; #5 añade el **desembolso**.

---

## 8. Matriz de decisión

Rúbrica /30 (mayor = mejor): **ZK-LB** (¿se rompe sin ZK?) · **Compl⁻¹** (simplicidad operativa — tu filtro clave) · **$STR** (mueve dinero real) · **NewTech** (exhibe palanca nueva de Stellar) · **Demo** (historia de una frase) · **Comercial** (viabilidad concreta).

| # | Idea | ZK-LB | Compl⁻¹ | $STR | NewTech | Demo | Comercial | **/30** |
|---|------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| 4 | **ZK-ProofOfRight** (subsidios) | 5 | 5 | 5 | 4 | 5 | 5 | **29** |
| 1 | **ZK-Terroir** (fair-trade) | 5 | 5 | 5 | 4 | 5 | 4 | **28** |
| 3 | **VeritasVoice** (feedback + IA) | 5 | 4 | 4 | 5 | 5 | 5 | **28** |
| 2 | **ZK-FoodWaste-Tax** (donación) | 5 | 5 | 4 | 4 | 4 | 5 | **27** |
| 6 | **ZK-ConfidentialPayroll** (nómina) | 5 | 3 | 5 | 5 | 4 | 5 | **27** |
| 5 | **ZK-FairVote** (presupuesto) | 5 | 4 | 5 | 4 | 4 | 4 | **26** |
| 7 | **ZK-Quorum** (urna institucional) | 5 | 3 | 4 | 5 | 4 | 5 | **26** |

*Empates resueltos por simplicidad: con 4 días, `Compl⁻¹` pesa doble en la práctica. Para #7, `$STR` mide el rol **infraestructural** de Stellar (throughput + auditoría + SaaS en USDC), no solo el flujo de dinero — es el caso que más estira el ledger de Stellar como producto.*

---

## 9. Recomendación

**Si quieres asegurar podio con mínima varianza → ZK-ProofOfRight o ZK-Terroir.** Son el circuito más simple (un range/membership), dinero real cross-border (lo que Stellar evangeliza) y una historia de una frase que un jurado cansado recuerda. ProofOfRight tiene la superficie comercial más amplia; Terroir tiene la narrativa foodtech más bonita y tu credibilidad de premio de innovación alimentaria detrás.

**Si quieres la idea más *tuya* y diferenciada → VeritasVoice.** Es la que sugieres, está en la convocatoria (bajo riesgo de encaje), y el **agente de IA anonimizador es un diferenciador que casi nadie tendrá**: combina ZK (identidad) + IA (contenido) para cerrar la re-identificación completa. Encaja con tu premio de innovación social.

**Si quieres impresionar al jurado técnico de la SDF → ZK-ConfidentialPayroll.** Es la única que exhibe el estándar Confidential Token más la divulgación selectiva; es B2B, recurrente y obvio. Mayor complejidad, mayor techo técnico.

**Si quieres la apuesta de infraestructura más ambiciosa (B2B/sindical de alto valor) → ZK-Quorum.** Es la única que toca el tier "wild" (agregación/recursión) y usa el ledger de Stellar como **substrato escalable y auditable**, no solo como rail de pago. Mayor ambición, pero es justo lo que separa "una app de voto" de "infraestructura electoral seria". Recórtala con *batch verification* honesto (no recursión completa) para que sea entregable en 4 días, y deja la escala a millones documentada como roadmap.

> **El movimiento inteligente:** las 7 comparten el esqueleto `commitment Poseidon → prueba Groth16 → nullifier → pago SEP-41` (el #7 lo extiende: en vez de un SNARK por voto, **commitments baratos por voto + una sola prueba de tally agregada**). Construye **ese núcleo común + Passkey** en los días 1–2, mide el costo de verificación on-chain con una prueba dummy el día 1 (único riesgo que hunde el proyecto), y **decide la "personalidad" el día 2** sin perder trabajo. Mi apuesta de mayor valor esperado: **ProofOfRight** como base segura, con la opción de vestirla de **Terroir** o **VeritasVoice** según cuál narres con más fuego en el video — y **ZK-Quorum** si te atrae el techo de infraestructura y aceptas su mayor varianza.

---

## 10. De-risking (lecciones validadas de los docs previos)

1. **Día 1: spike del costo de verificación.** Despliega el verificador Groth16 (`soroban-examples/groth16_verifier`) y verifica una prueba dummy on-chain. Es lo único que puede hundir el proyecto. Si usaras Noir/UltraHonk y no cabe → cae a Circom/Groth16 (más barato).
2. **Custodia del secreto = elección de stack.** En las 7, el secreto lo custodia el *usuario* → proving en navegador → **Circom/Groth16** (snarkjs WASM), **no** RISC Zero (no prueba en navegador). En #7 el *tally* agregado puede probarlo un agregador en servidor; el voto individual se prueba en el navegador del votante.
3. **Poseidon nativo + BN254 nativo, explícitos.** Construye commitments con `poseidon` del host y verifica con `pairing_check` BN254. Es más barato *y* es el guiño que el jurado quiere ver.
4. **Mock honesto y dilo en el README.** La convocatoria premia el "honest work-in-progress" sobre el "polished mystery". Mockea las firmas de emisores; deja real el circuito, la verificación y el pago.
5. **Vertical slice brutal.** Un solo subsidio / un solo lote / un solo grupo. Resiste generalizar; la generalidad se cuenta, no se construye.
