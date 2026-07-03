# ZK-Terroir · Frontend de proving en el navegador (Ola 5)

Página **estática, sin backend** que genera una prueba **Groth16/BN254 en el navegador** con
`snarkjs.groth16.fullProve`, la verifica off-chain contra la VK desplegada, la serializa al layout
BN254/EIP-197 de Soroban y muestra el comando `claim_premium` + un QR para el verificador público.

> **El momento que gana:** la prueba se genera en TU navegador — el `lot_secret` y los datos del lote
> nunca salen de tu equipo. El contrato solo ve la prueba.

## Correrlo

```bash
cd web
python3 -m http.server 8099
# abrí http://localhost:8099/  → "Generar prueba" (~1s en Chrome desktop)
```

Todo se sirve local; no hay CDNs externos. La tx on-chain (opcional) se hace pegando el comando
`stellar contract invoke ... claim_premium` que muestra la página (la firma es plomería — el proving,
lo ZK, ya ocurrió en el navegador).

## Estructura

```
index.html          UI (una página, vanilla, sin frameworks)
app.js              lógica: Poseidon(circomlibjs) → input role-tag → fullProve → verify → serialize → cmd + QR
public/
  terroir_chain.wasm            generador de witness del circuito (3.4 MB)
  terroir_chain_0001.zkey       proving key (9 MB) — PRESERVADO (setup no reproducible; único link a la VK desplegada)
  verification_key.json         VK (para verify off-chain en el browser)
  input.json                    ejemplo (el default se reconstruye en el browser desde el formulario)
  deployment.json               contrato/red (snapshot de deployments/testnet.json)
  vendor/                       snarkjs.min.js · circomlibjs-poseidon.min.js · qrcode.min.js (vendorizados)
build_vendor.mjs    cómo se generaron los bundles de public/vendor/ (npm + copia)
browser_test.mjs    smoke test headless (puppeteer-core): corre fullProve y asserta verify OK
```

## Verificado (Paso 5 del orquestador)

- `fullProve` corre en **Chrome real** (headless), ~0.7s → prueba válida.
- El input construido en el navegador produce el **r_cert sembrado on-chain** (`975646…302554`).
- Una prueba **generada en el navegador** (nullifier fresco) → enviada al contrato → **verifica TRUE
  on-chain → paga 37500** (contrato demo `CDECOLH6…`, payout `zkq-t0`).
- Sin claves de escritura ni secretos embebidos.

## Nota de honestidad

circomlibjs debe ser **0.1.7** (mismo Poseidon que el circuito) o el `r_cert` no matchea. El trusted
setup del circuito **no es ceremonial** (Powers-of-Tau de juguete) — ver README raíz. El `zkey` está
**trackeado** (no gitignored) porque el setup no es reproducible: sin él es imposible probar contra la
VK desplegada.
