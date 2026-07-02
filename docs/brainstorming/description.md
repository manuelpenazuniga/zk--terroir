# Stellar Hacks: Real-World ZK

## Real-World ZK on Stellar

This hackathon is wide open: build anything you want with zero-knowledge on Stellar. Privacy pools, private payments, confidential tokens, identity and compliance proofs, provable computation, verifiable data — if it uses ZK and runs on Stellar, it counts. You can go anywhere on the spectrum from mild (a clean proof-of-concept verifier) to wild (a full shielded payment app).

Stellar is best known for moving real money in the real world — stablecoins, cross-border payments, tokenized real-world assets, and institutional settlement. So projects that bring ZK to those kinds of real-world use cases are a natural fit and especially welcome. But that's a suggestion, not a requirement. A clever ZK demo, a niche privacy tool, or an experiment that just makes you curious is equally valid as long as ZK is doing real work in it (not just namechecked in the README).

We're running this now because Stellar has spent the last few protocol releases building out the cryptographic foundation that modern ZK systems need. Protocol 25 ("X-Ray") introduced native host functions for ZK-friendly primitives — BN254 elliptic-curve operations and Poseidon/Poseidon2 hashing — and Protocol 26 ("Yardstick") built on that with nine additional BN254 host functions (multi-scalar multiplication, scalar-field arithmetic, and curve-membership checks), moving heavy ZK math into the host layer and making proof verification — including NoirLang proofs — meaningfully cheaper to run on-chain. Combined with BLS12-381 from earlier protocols, Stellar now has the on-chain building blocks to verify zk-SNARK proofs efficiently and affordably.

A note worth setting expectations on: these primitives are building blocks. They don't, by themselves, give you end-to-end private payments out of the box — you generate proofs off-chain with a higher-level system (Noir, Circom, a RISC Zero zkVM program, etc.) and deploy a verifier contract on Stellar to check them. That gap between "powerful primitives" and "finished product" is exactly where the interesting hackathon projects live. The Resources tab has everything you need to close it.

## Hackathon Primer

Hackathon Primer Twitter Space

You need to include some form of zero-knowledge technology and have that integrated in Stellar i.e. verifying the proofs within a Stellar smart contract. There are three proven options for this currently on Stellar:

* **RISC Zero** is for executing code on a remote virtual machine and then proving it executed correctly.
* **Noir** is a beautiful Rust based language for creating zero-knowledge circuits.
* **Circom** offers lower level constraint based circuits which are harder to understand but cheaper to verify.

### RISC Zero

RISC Zero provides an execution environment where we can compute large amounts of data off-chain and then verify the output in a Stellar smart contract.

* RISC Zero Docs: [https://dev.risczero.com/](https://dev.risczero.com/)
* RISC Zero Verifier: [https://github.com/NethermindEth/stellar-risc0-verifier/](https://github.com/NethermindEth/stellar-risc0-verifier/)
* E2E Tutorial: [https://jamesbachini.com/stellar-risc-zero-games/](https://jamesbachini.com/stellar-risc-zero-games/)

### Circom

A domain specific langauge for building zero-knowledge circuits. Circom 1.0 was very mathematical and complex to understand, 2.0 is much more approachable and AI tools make it easier than ever.

Verify the Groth16 proofs within Stellar smart contracts.

* Circom Docs: [https://docs.circom.io/](https://docs.circom.io/)
* Groth16 Verifier Contracts: [https://github.com/stellar/soroban-examples/tree/main/groth16_verifier](https://github.com/stellar/soroban-examples/tree/main/groth16_verifier)
* E2E Tutorial: [https://jamesbachini.com/circom-on-stellar/](https://jamesbachini.com/circom-on-stellar/)

### Noir Lang

A Rust-like domain specific programming language for creating zero-knowledge circuits. Simple to read, understand and work with. The downside is the Ultrahonk proofs are larger and cost more to verify on-chain.

* Noir Docs: [https://noir-lang.org/docs/](https://noir-lang.org/docs/)
* Noir Verifier: [https://github.com/yugocabrio/rs-soroban-ultrahonk](https://github.com/yugocabrio/rs-soroban-ultrahonk)
* E2E Tutorial: [https://jamesbachini.com/noir-on-stellar/](https://jamesbachini.com/noir-on-stellar/)

## Resources

We've gathered a lot of resources to help you build — from the official ZK and Privacy docs to ready-to-clone verifier contracts and AI dev skills. Visit the Resources tab

## Submission Requirements

We're keeping requirements deliberately light. To be eligible, your submission needs:

* **An open-source repo.** A public GitHub, GitLab, or Bitbucket repository with your full source code and a clear README.md explaining what you built. The more detail, the better — and if something's unfinished or you used mock data in places, just say so in the README. We'd rather see an honest work-in-progress than a polished mystery.
* **A short demo video.** A 2–3 minute walkthrough showing what you built. It doesn't need to be heavily technical or produced — just clearly show the project working and explain what ZK is doing in it. You do not have to be in the video.
* **ZK + Stellar.** Your project should use zero-knowledge cryptography in a meaningful way, and it should touch Stellar — for example, verifying proofs in a contract, or otherwise integrating Stellar testnet or mainnet. The ZK should be load-bearing: it powers a real part of how the project works, rather than appearing only on a slide.

That's it. No mandatory framework, no required boilerplate contract to call, no specific track to fit into. Build what you find interesting.

## Inspiration & Ideas

Not sure where to start? The Inspiration & Ideas tab has a long list of directions sorted from mild to wild — all buildable on Stellar, all involving ZK.

## $10,000 Prize Pool

This hackathon features a single open innovation track with awards for the top projects:

* First Place: $5,000 in XLM
* Second Place: $2,000 in XLM
* Third Place: $1,250 in XLM
* Fourth Place: $1,000 in XLM
* Fifth Place: $750 in XLM

## Key Dates

* Submissions Open: June 15, 12:00AM PST
* Submission Deadline: June 29, 12:00PM PST

## Hackathon Support

The team is here to help you every step of the way. Drop into any of these channels for assistance:

* Stellar Dev Discord — #zk-chat channel [https://discord.gg/stellardev](https://discord.gg/stellardev)
* Stellar Hacks Telegram Group — [https://t.me/+e898qibDUVExODkx](https://t.me/+e898qibDUVExODkx)

Note: Please beware of scams via DM on both platforms. The team will never DM you first asking for keys, seed phrases, or payment.

---

## DoraHacks Links & Support

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
* **RISC Zero Docs:** [https://dev.risczero.com/](https://dev.risczero.com/)
* **RISC Zero Verifier (Nethermind):** [https://github.com/NethermindEth/stellar-risc0-verifier/](https://github.com/NethermindEth/stellar-risc0-verifier/)
* **Tutorial End-to-End (RISC Zero en Stellar):** [https://jamesbachini.com/stellar-risc-zero-games/](https://jamesbachini.com/stellar-risc-zero-games/)

**Circom:**
* **Circom Docs:** [https://docs.circom.io/](https://docs.circom.io/)
* **Contratos de Verificación Groth16 (Ejemplos Soroban):** [https://github.com/stellar/soroban-examples/tree/main/groth16_verifier](https://github.com/stellar/soroban-examples/tree/main/groth16_verifier)
* **Tutorial End-to-End (Circom en Stellar):** [https://jamesbachini.com/circom-on-stellar/](https://jamesbachini.com/circom-on-stellar/)

**Noir Lang:**
* **Noir Docs:** [https://noir-lang.org/docs/](https://noir-lang.org/docs/)
* **Noir Verifier (Ultrahonk):** [https://github.com/yugocabrio/rs-soroban-ultrahonk](https://github.com/yugocabrio/rs-soroban-ultrahonk)
* **Tutorial End-to-End (Noir en Stellar):** [https://jamesbachini.com/noir-on-stellar/](https://jamesbachini.com/noir-on-stellar/)

### 🤝 Soporte y Canales de la Comunidad
* **Stellar Dev Discord (Canal #zk-chat):** [https://discord.gg/stellardev](https://discord.gg/stellardev)
* **Grupo Oficial de Telegram (Stellar Hacks):** [https://t.me/+e898qibDUVExODkx](https://t.me/+e898qibDUVExODkx)




# RESOURCES

New to ZK on Stellar? Start with the two official docs pages — Privacy on Stellar and ZK Proofs on Stellar — then point your AI agent at the Stellar Skills below.

## Start Here: ZK & Privacy on Stellar

* ZK Proofs on Stellar (docs): [https://developers.stellar.org/docs/build/apps/zk](https://developers.stellar.org/docs/build/apps/zk) — The core reference. Explains the BN254 and Poseidon/Poseidon2 host functions, what they do, and how proof verification works on Stellar. Includes code examples and links to circuit tooling.
* Privacy on Stellar (docs): [https://developers.stellar.org/docs/build/apps/privacy](https://developers.stellar.org/docs/build/apps/privacy) — Overview of the privacy stack: Privacy Pools, Confidential Tokens, on-chain ZK verifiers, and the underlying cryptographic primitives. The best map of the whole landscape.
* Announcing Stellar X-Ray (Protocol 25): [https://stellar.org/blog/developers/announcing-stellar-x-ray-protocol-25](https://stellar.org/blog/developers/announcing-stellar-x-ray-protocol-25) — Background on why these primitives were added and the long-term privacy strategy behind them.
* Yardstick (Protocol 26) upgrade guide: [https://stellar.org/blog/foundation-news/stellar-yardstick-protocol-26-upgrade-guide](https://stellar.org/blog/foundation-news/stellar-yardstick-protocol-26-upgrade-guide) — What Protocol 26 added for ZK builders and why proof verification got cheaper.

## AI Development Assistance

We assume most of you will be building with an AI agent. Give it the right Stellar context first — it dramatically improves the code it writes.

* Stellar Skills (start here): [https://skills.stellar.org/](https://skills.stellar.org/) — Agent-readable documentation for building on Stellar, with dedicated skills for Soroban, dApps/wallets, assets, data/APIs, agentic payments, and ZK Proofs. Works with any AI agent. The simplest path: tell your agent "Read skills.stellar.org before you start building on Stellar."
* ZK Proofs skill (direct): [https://skills.stellar.org/skills/zk-proofs/SKILL.md](https://skills.stellar.org/skills/zk-proofs/SKILL.md) — Verify Groth16 zero-knowledge proofs on Stellar using BLS12-381, BN254, and Poseidon primitives.
* Stellar Dev Skill (repo): [https://github.com/stellar/stellar-dev-skill](https://github.com/stellar/stellar-dev-skill) — The underlying open-source skill repo. Covers Soroban, SDKs, RPC, wallet integration, passkeys, and security patterns.
  * Install in Claude Code: `/plugin marketplace add stellar/stellar-dev-skill` then `/plugin install stellar-dev@stellar-dev`
  * Cursor: add `stellar/stellar-dev-skill`
  * Codex: `git clone https://github.com/stellar/stellar-dev-skill ~/.codex/skills/stellar-dev-skill`
* stellar-build: [https://github.com/kaankacar/stellar-build](https://github.com/kaankacar/stellar-build) — Stellar development journey installer of 42 skills covering the full journey from idea to mainnet deploy and SCF grant submission, with six DevRel-persona agents.
* OpenZeppelin Skills: [https://github.com/OpenZeppelin/openzeppelin-skills](https://github.com/OpenZeppelin/openzeppelin-skills) — Claude Code skills for secure Stellar contract development. Install: `/plugin marketplace add OpenZeppelin/openzeppelin-skills` and `/plugin install openzeppelin-skills`
* Building with AI (docs): [https://developers.stellar.org/docs/build/building-with-ai](https://developers.stellar.org/docs/build/building-with-ai) — Stellar's guidance on AI-assisted development.
* llms.txt: [https://developers.stellar.org/llms.txt](https://developers.stellar.org/llms.txt) — Machine-readable digest of the Stellar docs, designed to be fed into any LLM.

## On-Chain ZK Verifiers (Reference Implementations)

These are the closest thing to "starter code" for this hackathon — deployable verifier contracts you can study, fork, and build on.

* RISC Zero (Groth16) verifier: [https://github.com/NethermindEth/stellar-risc0-verifier](https://github.com/NethermindEth/stellar-risc0-verifier) — Verifies Groth16 proofs created with RISC Zero's zkVM (write your provable program in Rust). Built by Nethermind's ZK team. 
  * Companion article: [https://stellar.org/blog/developers/risc-zero-verifier](https://stellar.org/blog/developers/risc-zero-verifier)
* UltraHonk verifier (Noir / Barretenberg): [https://github.com/yugocabrio/rs-soroban-ultrahonk](https://github.com/yugocabrio/rs-soroban-ultrahonk) / [https://github.com/indextree/ultrahonk_soroban_contract](https://github.com/indextree/ultrahonk_soroban_contract) — Verifier for circuits built with Aztec's Noir language. A clean pattern for proving valid solutions/state without revealing them.
* Stellar Private Payments (Privacy Pools PoC): [https://github.com/NethermindEth/stellar-private-payments](https://github.com/NethermindEth/stellar-private-payments) — Nethermind's proof-of-concept Privacy Pools implementation using Circom circuits, Groth16 proofs, and Stellar smart contracts. Includes a pool contract, on-chain Groth16 verifier, and ASP membership/non-membership contracts. Proofs are generated client-side in the browser via WebAssembly, so secrets never leave the device. 
  * Docs: [https://nethermindeth.github.io/stellar-private-payments/](https://nethermindeth.github.io/stellar-private-payments/) 
  * Caution: research prototype, not audited. Don't use with real assets.

## ZK Circuit Tooling

* Noir (Aztec): [https://noir-lang.org/docs/](https://noir-lang.org/docs/) — A friendly, Rust-like DSL for writing ZK circuits. Pairs with the UltraHonk verifier above. Protocol 26 made Noir proof verification on Stellar significantly cheaper.
* RISC Zero (zkVM): [https://dev.risczero.com/](https://dev.risczero.com/) — Write your provable program in ordinary Rust and prove its execution. Pairs with the RISC Zero verifier above.
* Circom: Used by the Stellar Private Payments PoC for Groth16 circuits. (See the privacy-pools repo for a worked example.)
* Soroban SDK — BN254 docs: [https://docs.rs/soroban-sdk/latest/soroban_sdk/_migrating/v25_bn254/index.html](https://docs.rs/soroban-sdk/latest/soroban_sdk/_migrating/v25_bn254/index.html)
* Soroban SDK — Poseidon docs: [https://docs.rs/soroban-sdk/latest/soroban_sdk/_migrating/v25_poseidon/index.html](https://docs.rs/soroban-sdk/latest/soroban_sdk/_migrating/v25_poseidon/index.html)
* Protocol CAPs (deep cuts): BN254 — CAP-0074 · Poseidon/Poseidon2 — CAP-0075 · BLS12-381 — CAP-0059
* Soroban P25 preview examples: [https://github.com/jayz22/soroban-examples/tree/p25-preview/p25-preview](https://github.com/jayz22/soroban-examples/tree/p25-preview/p25-preview)

## Further Privacy Context

* Confidential Token Association: [https://www.confidentialtoken.org/](https://www.confidentialtoken.org/) — Open standard (SDF, Nethermind, OpenZeppelin, Zama) for encryption-based on-chain confidentiality compatible with existing token interfaces. 
  * Overview/demo video: [https://www.youtube.com/watch?v=6NnDqVQYOHM](https://www.youtube.com/watch?v=6NnDqVQYOHM)
* Privacy Pools whitepaper (Buterin, Illum, Nadler, Schär, Soleimani): [https://privacypools.com/whitepaper.pdf](https://privacypools.com/whitepaper.pdf) — The conceptual basis for compliant privacy pools (deposits/withdrawals visible, in-pool transfers private, with ASP allow/deny lists).

## Core Stellar Dev Tools

* Stellar Docs: [https://developers.stellar.org/](https://developers.stellar.org/) — Core documentation for building on Stellar.
* SDKs: [https://developers.stellar.org/docs/tools/sdks](https://developers.stellar.org/docs/tools/sdks) — Libraries to interact with the network in your preferred language. (Use the latest SDK version for Protocol 26 support.)
* Stellar CLI: [https://developers.stellar.org/docs/tools/cli](https://developers.stellar.org/docs/tools/cli) — Build, deploy, and interact with Soroban smart contracts from the command line.
* Lab: [https://developers.stellar.org/docs/tools/lab](https://developers.stellar.org/docs/tools/lab) — Explore, test, and experiment with Stellar tools and APIs in the browser (also handy for generating + funding testnet accounts).
* Quickstart: [https://developers.stellar.org/docs/tools/quickstart](https://developers.stellar.org/docs/tools/quickstart) — Run a local Stellar network via Docker for development/testing.
* Scaffold Stellar: [https://scaffoldstellar.org](https://scaffoldstellar.org) — CLI for the full Stellar app development lifecycle: contract management, testing, and deployment with best practices baked in.
* Stellar Wallets Kit: [https://stellarwalletskit.dev/](https://stellarwalletskit.dev/) — Plug-and-play wallet connections with a unified API.
* OpenZeppelin on Stellar: [https://www.openzeppelin.com/networks/stellar](https://www.openzeppelin.com/networks/stellar) — Audited contract library, Contracts Wizard, Contracts MCP server, Relayer, and Soroban security detectors.

## Smart Contract Building Blocks (Docs)

* Smart Contracts — Getting Started: [https://developers.stellar.org/docs/build/smart-contracts/getting-started](https://developers.stellar.org/docs/build/smart-contracts/getting-started)
* Contract Authorization: [https://developers.stellar.org/docs/build/guides/auth](https://developers.stellar.org/docs/build/guides/auth)
* Contract Storage: [https://developers.stellar.org/docs/build/guides/storage](https://developers.stellar.org/docs/build/guides/storage)
* Contract Testing: [https://developers.stellar.org/docs/build/guides/testing](https://developers.stellar.org/docs/build/guides/testing)

## Community Resources

* Stellar Ecosystem Resources: [https://github.com/stellar/ecosystem-resources/](https://github.com/stellar/ecosystem-resources/) — Workshop activations and reference guides for Soroban, wallets, DeFi protocols, OpenZeppelin, tokens, and security.
* Stellar Hackathon FAQ: [https://github.com/briwylde08/stellar-hackathon-faq](https://github.com/briwylde08/stellar-hackathon-faq) — Community-compiled FAQ from building on Stellar.
* Stellar Ecosystem DB: [https://github.com/lumenloop/stellar-ecosystem-db](https://github.com/lumenloop/stellar-ecosystem-db) — Searchable database of Stellar projects — useful for finding existing work before building from scratch.

---

# INSPIRATION & IDEAS

Need a spark? Here's a spectrum of ZK ideas you can build on Stellar — from mild (a focused weekend build) to wild (an ambitious moonshot). Stellar's strength is real-world money movement, so several of these lean toward payments, identity, and real-world assets — but build whatever interests you. The only constants: it uses ZK, and it runs on Stellar.

## 🟢 Mild — start here, very buildable

* Proof-of-balance / proof-of-funds. Prove your account holds at least X USDC without revealing the exact balance. Classic "range proof" wrapped in a Stellar verifier — a perfect first ZK project.
* Age / eligibility check. Prove you're over 18 (or in an allowed region) to access a service, without disclosing your birthdate or address.
* Private allowlist membership. Prove you're on an approved list (an airdrop, a beta, a DAO) without revealing which member you are — a Merkle-membership proof verified on-chain.
* Verifiable off-chain computation. Run a calculation off-chain (a credit score, a tax estimate, a game result) and post a succinct proof that it was done correctly, using a RISC Zero or Noir circuit + a Stellar verifier.
* Anonymous feedback / attestation. Let verified customers or employees leave feedback that's provably from a real member of a group, without identifying who.

## 🟡 Medium — a meatier weekend project

* Private payment / shielded transfer. A simple deposit-and-withdraw flow where in-pool transfer amounts and counterparties stay hidden, building on the Stellar Private Payments PoC and the Groth16 verifier.
* Confidential payroll or invoicing. Pay a team in stablecoins where individual salaries/amounts stay private on-chain, but the employer can still prove totals to an auditor.
* Compliant private transfer with a view key. Keep a transfer private from the public, while letting an authorized party (auditor, regulator) reconstruct details with a view key — the "selective disclosure" pattern Stellar's privacy strategy is built around.
* Private credential / reputation. Issue a verifiable credential (KYC-passed, accredited investor, certified pro) that the holder can prove without revealing the underlying documents.
* Sealed-bid auction or vote. Bids/votes are committed privately, then revealed or proven valid at settlement, so no one can peek or change their answer after the fact.
* Proof-of-reserves for an issuer. A stablecoin or RWA issuer proves backing/solvency thresholds on-chain without exposing full account-level detail.

## 🟠 Spicy — ambitious, more moving parts

* Compliant privacy pool with ASP integration. A working privacy pool with Association Set Provider allow/deny lists, so legitimate users transact privately while known bad actors are excluded — the compliant-privacy sweet spot for real-world adoption.
* Confidential token implementation. Hide balances and amounts while keeping sender/receiver addresses public — useful where the counterparties are known but the numbers shouldn't be (B2B settlement, institutional flows). Aligns with the emerging Confidential Token standard.
* Private RWA settlement. Tokenized real-world assets (treasuries, invoices, credit) that settle on Stellar with amounts and positions shielded, while preserving auditability.
* Privacy-preserving on-chain identity. Prove ownership/validity of a real-world certificate or government ID and bind it to a Stellar address — with a nullifier for Sybil resistance — without revealing personal identifiers.
* Private DAO membership & governance. A DAO where members prove they can vote without revealing who they are, and where the governance rules themselves can stay hidden.

## 🔴 Wild — moonshots, go nuts

* Fully shielded stablecoin wallet. A consumer-grade wallet where everyday USDC payments are private by default, with client-side proof generation, compliant disclosure built in, and a UX an ordinary person could actually use.
* Private cross-border remittance corridor. End-to-end: fiat on-ramp → shielded transfer across a corridor → fiat off-ramp, with amounts private throughout and compliance proofs at the edges. Stellar's real-world payment rails, made confidential.
* ZK-powered confidential DeFi. A lending market, DEX, or yield vault on Stellar where positions and balances are private but solvency and correctness are provable.
* UTXO-style private payment system. A from-scratch private value-transfer design (in the spirit of SDF-funded research like Moonlight) exploring a different privacy model than account-based pools.
* Proof aggregation / recursive proofs on Stellar. Push the primitives: batch many proofs into one, or build recursive verification, to make private apps cheaper and more scalable on-chain.
* Cross-chain private bridge. Use Stellar's BN254 compatibility (it mirrors Ethereum's precompiles) to verify proofs originating on another ecosystem, enabling private cross-chain flows.

Remember: these are prompts, not requirements. "Mild" projects win hackathons all the time when they're sharp and well-executed. Pick something you can actually ship in the time you have, make the ZK genuinely essential, and document it clearly.

---

## DoraHacks Links & Support

* Binance Live: [https://www.binance.com/en/live](https://www.binance.com/en/live)
* YouTube: [https://www.youtube.com/@DoraHacks](https://www.youtube.com/@DoraHacks)
* Telegram: [https://t.me/dorahacksofficial](https://t.me/dorahacksofficial)
* Discord: [https://discord.gg/xKJNFRz3bp](https://discord.gg/xKJNFRz3bp)