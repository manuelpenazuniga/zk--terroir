#!/usr/bin/env node
// Para cada uno de los 7 públicos (Decisión A), corrompe public[i] (+1) y exige
// que snarkjs groth16 verify vuelva INVALID. Luego restaura.
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const snarkjs = require('/home/manuel/.nvm/versions/node/v24.17.0/lib/node_modules/snarkjs');

const dir = __dirname;
const vk = path.join(dir, 'verification_key.json');
const proof = path.join(dir, 'proof.json');
const pubFile = path.join(dir, 'public.json');

const original = JSON.parse(fs.readFileSync(pubFile, 'utf8'));
const names = ['r_cert','floor_price','lot_commit','premium_amount','payout_hi','payout_lo','nullifier_hash'];
if (original.length !== 7) throw new Error('public.json debe tener 7 señales, tiene ' + original.length);

(async () => {
  // sanity: la prueba inalterada verifica OK.
  let v = await snarkjs.groth16.verify(JSON.parse(fs.readFileSync(vk)), original, JSON.parse(fs.readFileSync(proof)));
  if (!v) throw new Error('la prueba ORIGINAL no verifica — abort');
  console.log('OK base: prueba original =', v);

  let allFail = true;
  for (let i = 0; i < 7; i++) {
    const tampered = original.slice();
    tampered[i] = (BigInt(original[i]) + 1n).toString();
    v = await snarkjs.groth16.verify(JSON.parse(fs.readFileSync(vk)), tampered, JSON.parse(fs.readFileSync(proof)));
    const passed = v; // queremos que sea false
    console.log(`${names[i].padEnd(14)} tamper(+1) -> verify = ${v}  ${v ? 'FAIL(test)' : 'ok(rechazo)'}`);
    if (v) allFail = false;
  }

  // sanity final: restaurar
  fs.writeFileSync(pubFile, JSON.stringify(original, null, 1));
  if (!allFail) process.exit(1);
  console.log('\nRESULTADO: los 7 públicos rechazan tampering. PRUEBA T1 SOUND.');
  process.exit(0);
})();