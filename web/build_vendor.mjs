// web/build_vendor.mjs
// Bundles circomlibjs 0.1.7's `buildPoseidon` into a single IIFE that exposes
// `window.circomlibjs.buildPoseidon` (and a few siblings we don't need but they
// cost ~zero). Uses the SAME circomlibjs that produced the deployed VK
// (`circuits/node_modules/circomlibjs`, version 0.1.7) and the SAME browser
// build of ffjavascript already vendored in `web/node_modules/ffjavascript/build/browser.esm.js`.
//
// Why an IIFE: simplest possible browser loader, no module shenanigans,
// no <script type="module"> in the demo. ~2MB output (poseidon_constants_opt
// is 1.7MB; ffjavascript is ~570KB; poseidon_wasm itself is tiny).
//
// Run: node web/build_vendor.mjs

import { build } from 'esbuild';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname  = path.dirname(__filename);
const ROOT       = path.resolve(__dirname, '..');

const circuitsNM  = path.join(ROOT, 'circuits', 'node_modules');
const webNM       = path.join(ROOT, 'web', 'node_modules');
const outFile     = path.join(__dirname, 'public', 'vendor', 'circomlibjs-poseidon.min.js');

if (!fs.existsSync(path.join(circuitsNM, 'circomlibjs'))) {
  console.error('ERROR: circuits/node_modules/circomlibjs missing — run `npm ci` in circuits/.');
  process.exit(1);
}
const cljsVer = JSON.parse(fs.readFileSync(path.join(circuitsNM, 'circomlibjs', 'package.json'), 'utf8')).version;
const ffjVer  = JSON.parse(fs.readFileSync(path.join(webNM,     'ffjavascript',    'package.json'), 'utf8')).version;
console.log(`circomlibjs=${cljsVer}  ffjavascript=${ffjVer}`);

// Entry: a tiny shim that imports ONLY buildPoseidon from the specific
// poseidon_wasm.js file (NOT from the main.js barrel, which would pull in
// eddsa/mimc7/smt and their CJS deps). esbuild will inline the constants
// JSON, the poseidon_wasm logic, and ffjavascript's browser ESM build, then
// produce a single self-contained IIFE.
const cljsPoseidonPath = path.join(circuitsNM, 'circomlibjs', 'src', 'poseidon_wasm.js');
const shimSrc = `
export { buildPoseidon } from ${JSON.stringify(cljsPoseidonPath)};
`;

const shimPath = path.join(__dirname, '.vendor-shim.mjs');
fs.writeFileSync(shimPath, shimSrc);

try {
  await build({
    entryPoints: [shimPath],
    bundle: true,
    minify: true,
    format: 'iife',
    target: ['es2020'],
    platform: 'browser',
    globalName: '__cljs',
    // Resolve circomlibjs from the circuits node_modules so we use 0.1.7
    // (not whatever might be in web/node_modules).
    nodePaths: [circuitsNM],
    alias: {
      // Force the browser build of ffjavascript (web NM has the esm build).
      // We point at the explicit file to avoid the package.json "exports"
      // resolution landing on the CJS variant.
      'ffjavascript': path.join(webNM, 'ffjavascript', 'build', 'browser.esm.js'),
    },
    define: { 'process.env.NODE_ENV': '"production"' },
    outfile: outFile,
    logLevel: 'info',
  });
} finally {
  fs.unlinkSync(shimPath);
}

const size = fs.statSync(outFile).size;
console.log(`wrote ${path.relative(ROOT, outFile)}  (${(size/1024).toFixed(0)} KB)`);
