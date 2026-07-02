# Stellar Hacks: ZK en el Mundo Real

## ZK en el Mundo Real sobre Stellar

Este hackathon es completamente abierto: construye lo que quieras usando conocimiento cero (ZK) sobre Stellar. Grupos de privacidad (privacy pools), pagos privados, tokens confidenciales, pruebas de identidad y cumplimiento normativo, cómputo demostrable, datos verificables — si usa ZK y corre sobre Stellar, cuenta. Puedes moverte a lo largo de todo el espectro: desde lo moderado (un verificador limpio como prueba de concepto) hasta lo ambicioso (una aplicación completa de pagos blindados).

Stellar es conocida principalmente por mover dinero real en el mundo real — stablecoins, pagos transfronterizos, activos del mundo real tokenizados y liquidación institucional. Por eso, los proyectos que llevan ZK a ese tipo de casos de uso del mundo real son un encaje natural y son especialmente bienvenidos. Pero eso es una sugerencia, no un requisito. Una demostración ZK ingeniosa, una herramienta de privacidad de nicho, o un experimento que simplemente te despierte curiosidad son igual de válidos, siempre que ZK esté haciendo trabajo real (no solo mencionado en el README).

Lanzamos esto ahora porque Stellar ha pasado las últimas versiones de protocolo construyendo la base criptográfica que los sistemas ZK modernos necesitan. El Protocolo 25 ("X-Ray") introdujo funciones nativas del host para primitivas favorables a ZK — operaciones de curva elíptica BN254 y hashing Poseidon/Poseidon2 — y el Protocolo 26 ("Yardstick") amplió eso con nueve funciones de host BN254 adicionales (multiplicación multi-escalar, aritmética de campo escalar y verificaciones de pertenencia a la curva), llevando la matemática pesada de ZK a la capa del host y haciendo que la verificación de pruebas — incluyendo pruebas de NoirLang — sea significativamente más barata de ejecutar en la cadena. Combinado con BLS12-381 de protocolos anteriores, Stellar ahora cuenta con los bloques de construcción on-chain para verificar pruebas zk-SNARK de manera eficiente y asequible.

Una nota para establecer expectativas: estas primitivas son bloques de construcción. Por sí solas, no te dan pagos privados de extremo a extremo listos para usar — generas pruebas fuera de la cadena con un sistema de nivel superior (Noir, Circom, un programa zkVM de RISC Zero, etc.) y despliegas un contrato verificador en Stellar para verificarlas. Esa brecha entre "primitivas poderosas" y "producto terminado" es exactamente donde viven los proyectos interesantes del hackathon. La pestaña de Recursos tiene todo lo que necesitas para cerrarla.

## Introducción al Hackathon

Twitter Space — Introducción al Hackathon

Debes incluir alguna forma de tecnología de conocimiento cero e integrarla en Stellar, es decir, verificando las pruebas dentro de un contrato inteligente de Stellar. Actualmente existen tres opciones probadas para esto en Stellar:

* **RISC Zero** es para ejecutar código en una máquina virtual remota y luego demostrar que se ejecutó correctamente.
* **Noir** es un lenguaje elegante basado en Rust para crear circuitos de conocimiento cero.
* **Circom** ofrece circuitos de restricciones de bajo nivel que son más difíciles de entender pero más baratos de verificar.

### RISC Zero

RISC Zero proporciona un entorno de ejecución donde podemos calcular grandes volúmenes de datos fuera de la cadena y luego verificar la salida en un contrato inteligente de Stellar.

* Documentación de RISC Zero: [https://dev.risczero.com/](https://dev.risczero.com/)
* Verificador RISC Zero: [https://github.com/NethermindEth/stellar-risc0-verifier/](https://github.com/NethermindEth/stellar-risc0-verifier/)
* Tutorial de extremo a extremo: [https://jamesbachini.com/stellar-risc-zero-games/](https://jamesbachini.com/stellar-risc-zero-games/)

### Circom

Un lenguaje de dominio específico (DSL) para construir circuitos de conocimiento cero. Circom 1.0 era muy matemático y complejo de entender; la versión 2.0 es mucho más accesible y las herramientas de IA lo hacen más fácil que nunca.

Verifica las pruebas Groth16 dentro de contratos inteligentes de Stellar.

* Documentación de Circom: [https://docs.circom.io/](https://docs.circom.io/)
* Contratos Verificadores Groth16: [https://github.com/stellar/soroban-examples/tree/main/groth16_verifier](https://github.com/stellar/soroban-examples/tree/main/groth16_verifier)
* Tutorial de extremo a extremo: [https://jamesbachini.com/circom-on-stellar/](https://jamesbachini.com/circom-on-stellar/)

### Noir Lang

Un lenguaje de programación de dominio específico similar a Rust para crear circuitos de conocimiento cero. Simple de leer, entender y usar. La desventaja es que las pruebas UltraHonk son más grandes y cuestan más verificarlas on-chain.

* Documentación de Noir: [https://noir-lang.org/docs/](https://noir-lang.org/docs/)
* Verificador Noir: [https://github.com/yugocabrio/rs-soroban-ultrahonk](https://github.com/yugocabrio/rs-soroban-ultrahonk)
* Tutorial de extremo a extremo: [https://jamesbachini.com/noir-on-stellar/](https://jamesbachini.com/noir-on-stellar/)

## Recursos

Hemos reunido muchos recursos para ayudarte a construir — desde la documentación oficial de ZK y Privacidad hasta contratos verificadores listos para clonar y habilidades de desarrollo con IA. Visita la pestaña de Recursos.

## Requisitos de Envío

Mantenemos los requisitos deliberadamente ligeros. Para ser elegible, tu envío necesita:

* **Un repositorio de código abierto.** Un repositorio público en GitHub, GitLab o Bitbucket con tu código fuente completo y un README.md claro explicando qué construiste. Cuanto más detalle, mejor — y si algo está incompleto o usaste datos simulados en algún punto, indícalo en el README. Preferimos ver un trabajo en progreso honesto que un misterio pulido.
* **Un video de demostración corto.** Un recorrido de 2 a 3 minutos mostrando lo que construiste. No necesita ser muy técnico ni tener producción elaborada — solo muestra claramente el proyecto funcionando y explica qué hace ZK en él. No tienes que aparecer en el video.
* **ZK + Stellar.** Tu proyecto debe usar criptografía de conocimiento cero de manera significativa, y debe involucrar a Stellar — por ejemplo, verificando pruebas en un contrato, o de otra forma integrando la testnet o mainnet de Stellar. ZK debe ser esencial: potencia una parte real de cómo funciona el proyecto, en lugar de aparecer solo en una diapositiva.

Eso es todo. Sin marco de trabajo obligatorio, sin contrato plantilla requerido para llamar, sin una pista específica en la que encajar. Construye lo que encuentres interesante.

## Inspiración e Ideas

¿No sabes por dónde empezar? La pestaña de Inspiración e Ideas tiene una larga lista de direcciones ordenadas de moderado a ambicioso — todas construibles en Stellar, todas involucrando ZK.

## Premio Total de $10,000

Este hackathon presenta una única pista de innovación abierta con premios para los mejores proyectos:

* Primer Lugar: $5,000 en XLM
* Segundo Lugar: $2,000 en XLM
* Tercer Lugar: $1,250 en XLM
* Cuarto Lugar: $1,000 en XLM
* Quinto Lugar: $750 en XLM

## Fechas Clave

* Apertura de Envíos: 15 de junio, 12:00AM PST
* Fecha Límite de Envío: 29 de junio, 12:00PM PST

## Soporte del Hackathon

El equipo está aquí para ayudarte en cada paso del camino. Únete a cualquiera de estos canales para obtener asistencia:

* Discord de Desarrolladores de Stellar — canal #zk-chat [https://discord.gg/stellardev](https://discord.gg/stellardev)
* Grupo de Telegram de Stellar Hacks — [https://t.me/+e898qibDUVExODkx](https://t.me/+e898qibDUVExODkx)

Nota: Por favor, ten cuidado con las estafas por mensajes directos en ambas plataformas. El equipo nunca te contactará primero por DM pidiendo claves, frases semilla (seed phrases) o pagos.

---

## Enlaces y Soporte de DoraHacks

* Binance Live: [https://www.binance.com/en/live](https://www.binance.com/en/live)
* YouTube: [https://www.youtube.com/@DoraHacks](https://www.youtube.com/@DoraHacks)
* Telegram: [https://t.me/dorahacksofficial](https://t.me/dorahacksofficial)
* Discord: [https://discord.gg/xKJNFRz3bp](https://discord.gg/xKJNFRz3bp)

### 📌 Fuente de Extracción Original
* **URL de origen:** [https://dorahacks.io/hackathon/stellar-hacks-zk/detail](https://dorahacks.io/hackathon/stellar-hacks-zk/detail)

### 🎥 Videos en YouTube (Referenciados en el contexto del Hackathon)
La página menciona un video introductorio (Hackathon Primer) y un video de demostración sugerido. Además, incluye los enlaces directos a los perfiles sociales en el pie de página.
* **Video Demo Asociado (Confidential Token Association):** [Ver Video en YouTube](https://www.youtube.com/watch?v=6NnDqVQYOHM)
* **Canal Oficial de DoraHacks en YouTube:** [YouTube ↗](https://www.youtube.com/@DoraHacks)

### 📚 Documentación y Tutoriales de Tecnologías ZK

**RISC Zero:**
* **Documentación de RISC Zero:** [https://dev.risczero.com/](https://dev.risczero.com/)
* **Verificador RISC Zero (Nethermind):** [https://github.com/NethermindEth/stellar-risc0-verifier/](https://github.com/NethermindEth/stellar-risc0-verifier/)
* **Tutorial de Extremo a Extremo (RISC Zero en Stellar):** [https://jamesbachini.com/stellar-risc-zero-games/](https://jamesbachini.com/stellar-risc-zero-games/)

**Circom:**
* **Documentación de Circom:** [https://docs.circom.io/](https://docs.circom.io/)
* **Contratos Verificadores Groth16 (Ejemplos Soroban):** [https://github.com/stellar/soroban-examples/tree/main/groth16_verifier](https://github.com/stellar/soroban-examples/tree/main/groth16_verifier)
* **Tutorial de Extremo a Extremo (Circom en Stellar):** [https://jamesbachini.com/circom-on-stellar/](https://jamesbachini.com/circom-on-stellar/)

**Noir Lang:**
* **Documentación de Noir:** [https://noir-lang.org/docs/](https://noir-lang.org/docs/)
* **Verificador Noir (UltraHonk):** [https://github.com/yugocabrio/rs-soroban-ultrahonk](https://github.com/yugocabrio/rs-soroban-ultrahonk)
* **Tutorial de Extremo a Extremo (Noir en Stellar):** [https://jamesbachini.com/noir-on-stellar/](https://jamesbachini.com/noir-on-stellar/)

### 🤝 Soporte y Canales de la Comunidad
* **Discord de Desarrolladores de Stellar (Canal #zk-chat):** [https://discord.gg/stellardev](https://discord.gg/stellardev)
* **Grupo Oficial de Telegram (Stellar Hacks):** [https://t.me/+e898qibDUVExODkx](https://t.me/+e898qibDUVExODkx)




# RECURSOS

¿Eres nuevo en ZK sobre Stellar? Comienza con las dos páginas oficiales de documentación — Privacy on Stellar y ZK Proofs on Stellar — luego apunta tu agente de IA a las Stellar Skills que encontrarás más abajo.

## Por Dónde Empezar: ZK y Privacidad en Stellar

* ZK Proofs on Stellar (documentación): [https://developers.stellar.org/docs/build/apps/zk](https://developers.stellar.org/docs/build/apps/zk) — La referencia principal. Explica las funciones de host BN254 y Poseidon/Poseidon2, qué hacen y cómo funciona la verificación de pruebas en Stellar. Incluye ejemplos de código y enlaces a herramientas de circuitos.
* Privacy on Stellar (documentación): [https://developers.stellar.org/docs/build/apps/privacy](https://developers.stellar.org/docs/build/apps/privacy) — Panorama general del stack de privacidad: Privacy Pools, Tokens Confidenciales, verificadores ZK on-chain y las primitivas criptográficas subyacentes. El mejor mapa de todo el ecosistema.
* Anuncio de Stellar X-Ray (Protocolo 25): [https://stellar.org/blog/developers/announcing-stellar-x-ray-protocol-25](https://stellar.org/blog/developers/announcing-stellar-x-ray-protocol-25) — Contexto sobre por qué se añadieron estas primitivas y la estrategia de privacidad a largo plazo detrás de ellas.
* Guía de actualización Yardstick (Protocolo 26): [https://stellar.org/blog/foundation-news/stellar-yardstick-protocol-26-upgrade-guide](https://stellar.org/blog/foundation-news/stellar-yardstick-protocol-26-upgrade-guide) — Qué agregó el Protocolo 26 para los constructores de ZK y por qué la verificación de pruebas se abarató.

## Asistencia para Desarrollo con IA

Asumimos que la mayoría de ustedes construirá con un agente de IA. Primero dale el contexto correcto de Stellar — mejora drásticamente el código que genera.

* Stellar Skills (empieza aquí): [https://skills.stellar.org/](https://skills.stellar.org/) — Documentación legible por agentes para construir en Stellar, con skills dedicadas para Soroban, dApps/wallets, activos, datos/APIs, pagos agentivos y pruebas ZK. Funciona con cualquier agente de IA. El camino más simple: dile a tu agente "Lee skills.stellar.org antes de empezar a construir en Stellar."
* Skill ZK Proofs (directa): [https://skills.stellar.org/skills/zk-proofs/SKILL.md](https://skills.stellar.org/skills/zk-proofs/SKILL.md) — Verifica pruebas Groth16 de conocimiento cero en Stellar usando las primitivas BLS12-381, BN254 y Poseidon.
* Stellar Dev Skill (repositorio): [https://github.com/stellar/stellar-dev-skill](https://github.com/stellar/stellar-dev-skill) — El repositorio de skill open-source subyacente. Cubre Soroban, SDKs, RPC, integración de wallets, passkeys y patrones de seguridad.
  * Instalar en Claude Code: `/plugin marketplace add stellar/stellar-dev-skill` luego `/plugin install stellar-dev@stellar-dev`
  * Cursor: añadir `stellar/stellar-dev-skill`
  * Codex: `git clone https://github.com/stellar/stellar-dev-skill ~/.codex/skills/stellar-dev-skill`
* stellar-build: [https://github.com/kaankacar/stellar-build](https://github.com/kaankacar/stellar-build) — Instalador del recorrido de desarrollo en Stellar con 42 skills que cubren el camino completo desde la idea hasta el despliegue en mainnet y la presentación de una beca SCF, con seis agentes de tipo DevRel.
* OpenZeppelin Skills: [https://github.com/OpenZeppelin/openzeppelin-skills](https://github.com/OpenZeppelin/openzeppelin-skills) — Skills de Claude Code para el desarrollo seguro de contratos en Stellar. Instalar: `/plugin marketplace add OpenZeppelin/openzeppelin-skills` y `/plugin install openzeppelin-skills`
* Construcción con IA (documentación): [https://developers.stellar.org/docs/build/building-with-ai](https://developers.stellar.org/docs/build/building-with-ai) — Guía de Stellar para el desarrollo asistido por IA.
* llms.txt: [https://developers.stellar.org/llms.txt](https://developers.stellar.org/llms.txt) — Resumen legible por máquinas de la documentación de Stellar, diseñado para alimentar cualquier LLM.

## Verificadores ZK On-Chain (Implementaciones de Referencia)

Estos son lo más parecido a "código de inicio" para este hackathon — contratos verificadores desplegables que puedes estudiar, bifurcar y sobre los cuales construir.

* Verificador RISC Zero (Groth16): [https://github.com/NethermindEth/stellar-risc0-verifier](https://github.com/NethermindEth/stellar-risc0-verifier) — Verifica pruebas Groth16 creadas con la zkVM de RISC Zero (escribe tu programa demostrable en Rust). Construido por el equipo ZK de Nethermind.
  * Artículo complementario: [https://stellar.org/blog/developers/risc-zero-verifier](https://stellar.org/blog/developers/risc-zero-verifier)
* Verificador UltraHonk (Noir / Barretenberg): [https://github.com/yugocabrio/rs-soroban-ultrahonk](https://github.com/yugocabrio/rs-soroban-ultrahonk) / [https://github.com/indextree/ultrahonk_soroban_contract](https://github.com/indextree/ultrahonk_soroban_contract) — Verificador para circuitos construidos con el lenguaje Noir de Aztec. Un patrón limpio para demostrar soluciones o estados válidos sin revelarlos.
* Stellar Private Payments (PoC de Privacy Pools): [https://github.com/NethermindEth/stellar-private-payments](https://github.com/NethermindEth/stellar-private-payments) — Prueba de concepto de Privacy Pools de Nethermind usando circuitos Circom, pruebas Groth16 y contratos inteligentes de Stellar. Incluye un contrato de pool, un verificador Groth16 on-chain y contratos de membresía/no-membresía en el ASP (Proveedor de Conjunto de Asociación). Las pruebas se generan del lado del cliente en el navegador vía WebAssembly, por lo que los secretos nunca abandonan el dispositivo.
  * Documentación: [https://nethermindeth.github.io/stellar-private-payments/](https://nethermindeth.github.io/stellar-private-payments/)
  * Advertencia: prototipo de investigación, no auditado. No usar con activos reales.

## Herramientas de Circuitos ZK

* Noir (Aztec): [https://noir-lang.org/docs/](https://noir-lang.org/docs/) — Un DSL amigable, similar a Rust, para escribir circuitos ZK. Se combina con el verificador UltraHonk mencionado arriba. El Protocolo 26 abarató significativamente la verificación de pruebas Noir en Stellar.
* RISC Zero (zkVM): [https://dev.risczero.com/](https://dev.risczero.com/) — Escribe tu programa demostrable en Rust ordinario y demuestra su ejecución. Se combina con el verificador RISC Zero mencionado arriba.
* Circom: Usado por el PoC de Stellar Private Payments para circuitos Groth16. (Ver el repositorio de privacy-pools para un ejemplo detallado.)
* Soroban SDK — Documentación BN254: [https://docs.rs/soroban-sdk/latest/soroban_sdk/_migrating/v25_bn254/index.html](https://docs.rs/soroban-sdk/latest/soroban_sdk/_migrating/v25_bn254/index.html)
* Soroban SDK — Documentación Poseidon: [https://docs.rs/soroban-sdk/latest/soroban_sdk/_migrating/v25_poseidon/index.html](https://docs.rs/soroban-sdk/latest/soroban_sdk/_migrating/v25_poseidon/index.html)
* CAPs de protocolo (lectura profunda): BN254 — CAP-0074 · Poseidon/Poseidon2 — CAP-0075 · BLS12-381 — CAP-0059
* Ejemplos de Soroban P25 (preview): [https://github.com/jayz22/soroban-examples/tree/p25-preview/p25-preview](https://github.com/jayz22/soroban-examples/tree/p25-preview/p25-preview)

## Contexto Adicional de Privacidad

* Confidential Token Association: [https://www.confidentialtoken.org/](https://www.confidentialtoken.org/) — Estándar abierto (SDF, Nethermind, OpenZeppelin, Zama) para confidencialidad on-chain basada en cifrado, compatible con las interfaces de tokens existentes.
  * Video de presentación y demo: [https://www.youtube.com/watch?v=6NnDqVQYOHM](https://www.youtube.com/watch?v=6NnDqVQYOHM)
* Whitepaper de Privacy Pools (Buterin, Illum, Nadler, Schär, Soleimani): [https://privacypools.com/whitepaper.pdf](https://privacypools.com/whitepaper.pdf) — La base conceptual para privacy pools conformes con regulaciones (depósitos y retiros visibles, transferencias dentro del pool privadas, con listas de permisos y bloqueos del ASP).

## Herramientas Principales de Desarrollo en Stellar

* Documentación de Stellar: [https://developers.stellar.org/](https://developers.stellar.org/) — Documentación principal para construir en Stellar.
* SDKs: [https://developers.stellar.org/docs/tools/sdks](https://developers.stellar.org/docs/tools/sdks) — Librerías para interactuar con la red en tu lenguaje preferido. (Usa la versión más reciente del SDK para soporte del Protocolo 26.)
* CLI de Stellar: [https://developers.stellar.org/docs/tools/cli](https://developers.stellar.org/docs/tools/cli) — Construye, despliega e interactúa con contratos inteligentes Soroban desde la línea de comandos.
* Lab: [https://developers.stellar.org/docs/tools/lab](https://developers.stellar.org/docs/tools/lab) — Explora, prueba y experimenta con herramientas y APIs de Stellar en el navegador (también útil para generar y fondear cuentas en testnet).
* Quickstart: [https://developers.stellar.org/docs/tools/quickstart](https://developers.stellar.org/docs/tools/quickstart) — Ejecuta una red Stellar local vía Docker para desarrollo y pruebas.
* Scaffold Stellar: [https://scaffoldstellar.org](https://scaffoldstellar.org) — CLI para el ciclo de vida completo de desarrollo de aplicaciones Stellar: gestión de contratos, pruebas y despliegue con mejores prácticas integradas.
* Stellar Wallets Kit: [https://stellarwalletskit.dev/](https://stellarwalletskit.dev/) — Conexiones de wallet plug-and-play con una API unificada.
* OpenZeppelin en Stellar: [https://www.openzeppelin.com/networks/stellar](https://www.openzeppelin.com/networks/stellar) — Librería de contratos auditados, Contracts Wizard, servidor MCP para contratos, Relayer y detectores de seguridad para Soroban.

## Bloques de Construcción para Contratos Inteligentes (Documentación)

* Contratos Inteligentes — Primeros Pasos: [https://developers.stellar.org/docs/build/smart-contracts/getting-started](https://developers.stellar.org/docs/build/smart-contracts/getting-started)
* Autorización en Contratos: [https://developers.stellar.org/docs/build/guides/auth](https://developers.stellar.org/docs/build/guides/auth)
* Almacenamiento en Contratos: [https://developers.stellar.org/docs/build/guides/storage](https://developers.stellar.org/docs/build/guides/storage)
* Pruebas de Contratos: [https://developers.stellar.org/docs/build/guides/testing](https://developers.stellar.org/docs/build/guides/testing)

## Recursos de la Comunidad

* Recursos del Ecosistema Stellar: [https://github.com/stellar/ecosystem-resources/](https://github.com/stellar/ecosystem-resources/) — Activaciones de talleres y guías de referencia para Soroban, wallets, protocolos DeFi, OpenZeppelin, tokens y seguridad.
* FAQ del Hackathon de Stellar: [https://github.com/briwylde08/stellar-hackathon-faq](https://github.com/briwylde08/stellar-hackathon-faq) — FAQ compilada por la comunidad sobre el desarrollo en Stellar.
* Base de Datos del Ecosistema Stellar: [https://github.com/lumenloop/stellar-ecosystem-db](https://github.com/lumenloop/stellar-ecosystem-db) — Base de datos con búsqueda de proyectos en Stellar — útil para encontrar trabajo existente antes de construir desde cero.

---

# INSPIRACIÓN E IDEAS

¿Necesitas un punto de partida? Aquí tienes un espectro de ideas ZK que puedes construir en Stellar — desde lo moderado (un proyecto de fin de semana enfocado) hasta lo ambicioso (un proyecto de alto impacto). La fortaleza de Stellar es el movimiento de dinero real, por lo que varios de estos apuntan hacia pagos, identidad y activos del mundo real — pero construye lo que más te interese. Las únicas constantes: usa ZK y corre sobre Stellar.

## 🟢 Moderado — empieza aquí, muy construible

* Prueba de saldo / prueba de fondos. Demuestra que tu cuenta tiene al menos X USDC sin revelar el saldo exacto. La clásica "prueba de rango" (range proof) envuelta en un verificador de Stellar — un primer proyecto ZK perfecto.
* Verificación de edad / elegibilidad. Demuestra que eres mayor de 18 años (o que te encuentras en una región permitida) para acceder a un servicio, sin revelar tu fecha de nacimiento ni tu dirección.
* Membresía privada en lista de permisos (allowlist). Demuestra que estás en una lista aprobada (un airdrop, una beta, un DAO) sin revelar cuál miembro eres — una prueba de membresía Merkle verificada on-chain.
* Cómputo verificable fuera de la cadena. Ejecuta un cálculo off-chain (una calificación crediticia, una estimación de impuestos, el resultado de un juego) y publica una prueba sucinta de que se hizo correctamente, usando un circuito RISC Zero o Noir más un verificador en Stellar.
* Retroalimentación anónima / atestación. Permite que clientes o empleados verificados dejen comentarios que sean demostrablemente de un miembro real de un grupo, sin identificar quién.

## 🟡 Medio — un proyecto de fin de semana más sustancioso

* Pago privado / transferencia blindada. Un flujo simple de depósito y retiro donde los montos y contrapartes de las transferencias dentro del pool permanecen ocultos, construido sobre el PoC de Stellar Private Payments y el verificador Groth16.
* Nómina o facturación confidencial. Paga a un equipo en stablecoins donde los salarios o montos individuales permanecen privados on-chain, pero el empleador aún puede demostrar los totales a un auditor.
* Transferencia privada conforme con clave de visualización (view key). Mantén una transferencia privada del público, mientras permites que una parte autorizada (auditor, regulador) reconstruya los detalles con una clave de visualización — el patrón de "divulgación selectiva" sobre el que está construida la estrategia de privacidad de Stellar.
* Credencial privada / reputación. Emite una credencial verificable (KYC aprobado, inversor acreditado, profesional certificado) que el titular puede demostrar sin revelar los documentos subyacentes.
* Subasta de oferta sellada o votación. Las ofertas o votos se comprometen de forma privada, luego se revelan o demuestran como válidos en el momento de la liquidación, para que nadie pueda espiar ni cambiar su respuesta después del hecho.
* Prueba de reservas para un emisor. Un emisor de stablecoin o activo del mundo real (RWA) demuestra umbrales de respaldo y solvencia on-chain sin exponer el detalle a nivel de cuenta individual.

## 🟠 Picante — ambicioso, más partes en movimiento

* Pool de privacidad conforme con integración de ASP. Un privacy pool funcional con listas de permisos y bloqueos del Proveedor de Conjunto de Asociación (ASP), para que los usuarios legítimos transaccionen de forma privada mientras los actores maliciosos conocidos son excluidos — el punto óptimo de privacidad conforme para la adopción en el mundo real.
* Implementación de token confidencial. Oculta saldos y montos mientras mantiene públicas las direcciones del emisor y receptor — útil cuando las contrapartes son conocidas pero las cifras no deben serlo (liquidación B2B, flujos institucionales). Se alinea con el estándar emergente de Confidential Token.
* Liquidación privada de RWA. Activos del mundo real tokenizados (bonos del tesoro, facturas, crédito) que se liquidan en Stellar con montos y posiciones blindados, preservando al mismo tiempo la auditabilidad.
* Identidad on-chain que preserva la privacidad. Demuestra la propiedad y validez de un certificado del mundo real o documento de identidad oficial y vincúlalo a una dirección de Stellar — con un anulador (nullifier) para resistencia Sybil — sin revelar identificadores personales.
* Membresía y gobernanza privada en un DAO. Un DAO donde los miembros demuestran que pueden votar sin revelar quiénes son, y donde las reglas de gobernanza en sí pueden permanecer ocultas.

## 🔴 Ambicioso — proyectos de alto impacto, sin límites

* Wallet de stablecoin completamente blindada. Una wallet orientada al consumidor donde los pagos cotidianos en USDC son privados por defecto, con generación de pruebas del lado del cliente, divulgación conforme integrada y una UX que una persona común pueda realmente usar.
* Corredor de remesas transfronterizas privadas. De extremo a extremo: rampa de entrada de fiat → transferencia blindada a través de un corredor → rampa de salida de fiat, con montos privados en todo momento y pruebas de cumplimiento en los extremos. Los raíles de pagos del mundo real de Stellar, hechos confidenciales.
* DeFi confidencial impulsado por ZK. Un mercado de préstamos, DEX o bóveda de rendimiento en Stellar donde las posiciones y saldos son privados pero la solvencia y corrección son demostrables.
* Sistema de pago privado estilo UTXO. Un diseño de transferencia de valor privado desde cero (en el espíritu de investigaciones financiadas por SDF como Moonlight) que explora un modelo de privacidad diferente al de los pools basados en cuentas.
* Agregación de pruebas / pruebas recursivas en Stellar. Lleva las primitivas al límite: agrupa muchas pruebas en una sola, o construye verificación recursiva, para que las aplicaciones privadas sean más baratas y escalables on-chain.
* Puente privado entre cadenas (cross-chain). Usa la compatibilidad de Stellar con BN254 (que espeja las precompilaciones de Ethereum) para verificar pruebas originadas en otro ecosistema, habilitando flujos privados entre cadenas.

Recuerda: estas son sugerencias, no requisitos. Los proyectos "moderados" ganan hackathons todo el tiempo cuando son precisos y bien ejecutados. Elige algo que puedas realmente entregar en el tiempo disponible, haz que ZK sea genuinamente esencial y documéntalo claramente.

---

## Enlaces y Soporte de DoraHacks

* Binance Live: [https://www.binance.com/en/live](https://www.binance.com/en/live)
* YouTube: [https://www.youtube.com/@DoraHacks](https://www.youtube.com/@DoraHacks)
* Telegram: [https://t.me/dorahacksofficial](https://t.me/dorahacksofficial)
* Discord: [https://discord.gg/xKJNFRz3bp](https://discord.gg/xKJNFRz3bp)
