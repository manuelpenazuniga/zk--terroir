// web/browser_test.mjs — Ola 5 / T5.1: smoke test del proving en navegador.
//
// Arranca Chrome headless, abre la página servida por `python3 -m http.server`
// en `web/`, pulsa #prove, espera el resultado y:
//   1) confirma que fullProve→verify dio OK (prueba válida contra la VK),
//   2) confirma que r_cert computado == sembrado on-chain (975646…302554),
//   3) confirma que la serialización BN254 tiene las longitudes esperadas
//      (proof.a=64B, proof.b=128B, proof.c=64B, 7×32B de pub_signals),
//   4) confirma que el QR se renderizó,
//   5) saca screenshot de la página final para inspección visual.
//
// Uso:
//   (cd web && python3 -m http.server 8099) &  // en otra terminal
//   node web/browser_test.mjs
import puppeteer from 'puppeteer-core';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname  = path.dirname(__filename);

const CHROME = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const URL    = 'http://localhost:8099/';
const SHOT   = path.join(__dirname, 'screenshot-final.png');

if (!fs.existsSync(CHROME)) {
  console.error('No se encontró Chrome en', CHROME);
  process.exit(2);
}

const b = await puppeteer.launch({
  executablePath: CHROME,
  headless: 'new',
  args: ['--no-sandbox', '--disable-setuid-sandbox'],
});
const page = await b.newPage();
await page.setViewport({ width: 1100, height: 1400, deviceScaleFactor: 2 });

const logs = [];
page.on('console', (m) => logs.push(m.type() + ': ' + m.text()));
page.on('pageerror', (e) => logs.push('PAGEERR: ' + e.message));

console.log('> goto', URL);
await page.goto(URL, { waitUntil: 'networkidle2', timeout: 60000 });

// smoke: el formulario existe y tiene los defaults
const formOk = await page.evaluate(() => {
  return {
    contractId: document.getElementById('contract-id').textContent,
    hasQR:      !!document.getElementById('qr'),
    hasCmd:     !!document.getElementById('cmd'),
    defaults: {
      pk0: document.getElementById('certifier_pk0').value,
      pp:  document.getElementById('price_paid').value,
      fl:  document.getElementById('floor_price').value,
    },
  };
});
console.log('> form sanity', JSON.stringify(formOk));
if (!formOk.contractId || formOk.contractId.length < 10) throw new Error('contract_id no cargado');
if (!formOk.hasQR)  throw new Error('falta #qr');
if (!formOk.hasCmd) throw new Error('falta #cmd');

console.log('> click #prove');
await page.click('#prove');

try {
  await page.waitForFunction('window.__ZKT_RESULT__ !== undefined', { timeout: 120000 });
} catch (e) {
  console.log('TIMEOUT esperando resultado');
}

const res = await page.evaluate(() => window.__ZKT_RESULT__);
console.log('> __ZKT_RESULT__', JSON.stringify(res, null, 2));

// aserciones duras
const failures = [];
if (!res || !res.ok)         failures.push('verify off-chain FALLÓ');
if (!res.serialized)         failures.push('falta serialized');
if (!res.serialized?.proof?.a || res.serialized.proof.a.length !== 64*2)
  failures.push('proof.a no es 64B hex');
if (!res.serialized?.proof?.b || res.serialized.proof.b.length !== 128*2)
  failures.push('proof.b no es 128B hex');
if (!res.serialized?.proof?.c || res.serialized.proof.c.length !== 64*2)
  failures.push('proof.c no es 64B hex');
if (!Array.isArray(res.serialized?.pub_signals) || res.serialized.pub_signals.length !== 7)
  failures.push('pub_signals != 7');
if (res.serialized?.pub_signals) {
  for (let i = 0; i < res.serialized.pub_signals.length; i++) {
    if (res.serialized.pub_signals[i].length !== 64) {
      failures.push(`pub_signals[${i}] no es 32B hex`);
      break;
    }
  }
}
const SEED = '975646771672022315473887566531216271187435207093346729074489866782278275453';
if (res.rCert !== SEED) failures.push(`r_cert no matchea seed on-chain: ${res.rCert?.slice(0,16)}…`);

// el QR debe estar renderizado
const qrOk = await page.evaluate(() => {
  const h = document.getElementById('qr');
  return { svg: !!h.querySelector('svg'), text: h.textContent.trim() };
});
console.log('> QR', JSON.stringify(qrOk));
if (!qrOk.svg) failures.push('QR no renderizó <svg>');

// screenshot para inspección
await page.screenshot({ path: SHOT, fullPage: true });
console.log('> screenshot', SHOT);

await b.close();

console.log('=== CONSOLE (browser) ===');
console.log(logs.join('\n'));

if (failures.length) {
  console.log('\n❌ FAILURES:');
  for (const f of failures) console.log('  -', f);
  process.exit(1);
}
console.log('\n✅ TODOS LOS CHECKS PASARON');
