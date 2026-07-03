// web/app.js — Ola 5 / T5.1: proving en navegador para ZK-Terroir.
//
// Responsabilidades (en este orden):
//   1. Inicializa Poseidon(BN254) con circomlibjs 0.1.7 (MISMO bundle que produjo la VK).
//   2. Lee los inputs editables de la UI (3 eslabones, precio, piso, lote).
//   3. Construye el witness input (mismo formato que circuits/gen_input.js + buildTree.js).
//   4. Compara r_cert calculado vs r_cert sembrado on-chain — si difiere, lo dice HONESTO
//      antes de probar (la prueba igual se intenta; en el peor caso, verify devuelve false).
//   5. Ejecuta snarkjs.groth16.fullProve en el navegador.
//   6. Verifica off-chain contra la VK desplegada (misma que el contrato).
//   7. Serializa proof + pubSignals al layout BN254/EIP-197 que espera Soroban.
//   8. Muestra el comando `stellar contract invoke ... claim_premium --proof ... --pub_signals ... --payout ...`.
//   9. Renderiza un QR con payload `zkterroir:verify?lot_commit=<hex>&contract=<CID>&network=testnet`.
//
// Decisiones:
//   - NO integramos wallet (la firma de la tx Stellar es plomería; la ZK ya ocurrió en el navegador).
//   - Default values = los mismos de circuits/gen_input.js => r_cert == on-chain seed.
//   - lot_secret/season_id/lot_id se exponen como "avanzado" (defaults fijos del demo).
//   - Sin frameworks: vanilla JS, sin React/Vue.
//   - UI en inglés; el pipeline se muestra como stepper de 5 pasos + log con aria-live.

(function () {
  'use strict';

  // --- DOM helpers ---
  const $ = (id) => document.getElementById(id);

  // --- Config (constantes de Ola 3) ---
  const ROLE_FINCA    = 1n;
  const ROLE_COOP     = 2n;
  const ROLE_TOSTADOR = 3n;
  const LEVELS        = 10;
  const NLEAVES       = 1 << LEVELS;

  // r_cert sembrado on-chain (el demo usa los defaults de gen_input.js).
  // Si el usuario edita los inputs a valores fuera del set acreditado, este check
  // avisa honestamente. La prueba igual se genera — y al verificar off-chain verá
  // true (porque la VK no conoce la raíz, solo la firma del prover); pero al
  // llegar al contrato, el `root == stored_r_cert` lo va a rechazar.
  const SEED_R_CERT_HEX = '975646771672022315473887566531216271187435207093346729074489866782278275453';

  // Defaults idénticos a circuits/gen_input.js / buildTree.js — datos del demo.
  const DEFAULTS = {
    certifier_pk0: '11',     // COOP (eslabón 0)
    certifier_pk1: '22',     // FINCA (eslabón 1)
    certifier_pk2: '33',     // TOSTADOR (eslabón 2)
    attest_data0:  '101',    // FINCA attestation
    attest_data1:  '202',    // TOSTADOR attestation
    price_paid:    '187500', // 1875.00 USDC (centavos)
    floor_price:   '150000', // 1500.00 USDC
    lot_id:        '7777777777777777',
    season_id:     '20262027',
    lot_secret:    '9999999999999999000000000000000000',
    // payout: pubkey ed25519 demo (32B hex) — la de gen_input.js / verify.sh
    payout_hex:    '3c0b8a02e3f16b9c4d7e5a3b0c0d6e1f4a2b3c4d5e6f7081920a3b4c5d6e7f81',
  };

  // --- App state ---
  let _pose = null;       // pose(arr) -> bigint  (initialized lazily)
  let _F     = null;
  let _builtInput = null; // last input.json sent to snarkjs
  let _proof = null;
  let _pubSignals = null;
  let _serialized = null; // { proof: {a,b,c}, pub_signals: [hex,...] }
  let _contract = null;   // from deployments
  let _network  = 'testnet';

  // ============================================================
  // Status log / stepper / feedback helpers
  // ============================================================
  const STEP_ORDER = ['setup', 'witness', 'prove', 'verify', 'serialize'];

  function setStep(name, state) {
    const el = document.querySelector('.step[data-step="' + name + '"]');
    if (!el) return;
    el.classList.remove('active', 'done', 'error');
    if (state) el.classList.add(state);
    const dot = el.querySelector('.step-dot');
    dot.textContent = state === 'done' ? '✓' : state === 'error' ? '✕' : String(STEP_ORDER.indexOf(name) + 1);
    if (state === 'active') el.setAttribute('aria-current', 'step');
    else el.removeAttribute('aria-current');
  }
  function resetSteps() { STEP_ORDER.forEach((s) => setStep(s, null)); }

  function logReset() { $('status').textContent = ''; }
  function logLine(msg, cls) {
    const s = $('status');
    const d = document.createElement('div');
    d.className = 'line' + (cls ? ' ' + cls : '');
    d.textContent = msg;
    s.appendChild(d);
    s.scrollTop = s.scrollHeight;
    console.log('[zkt]', msg);
  }

  function setBanner(msg, cls) {
    const b = $('banner');
    b.textContent = msg;
    b.className = 'banner ' + (cls || '');
    b.style.display = msg ? 'block' : 'none';
  }
  function setProofBadge(state, text) {
    const b = $('proof-badge');
    b.className = 'badge ' + (state || '');
    b.textContent = text || '';
  }
  function setSection(name, show) {
    const el = $('section-' + name);
    if (el) el.style.display = show ? 'block' : 'none';
  }
  function setBusy(on) {
    $('prove').disabled = on;
    $('spinner').style.display = on ? 'inline-block' : 'none';
    $('prove-label').textContent = on ? 'Proving…' : 'Generate proof';
  }

  // ============================================================
  // Live form feedback (premium hint + payout inline validation)
  // ============================================================
  function updatePremiumHint() {
    const el = $('premium-hint');
    try {
      const p = BigInt($('price_paid').value.trim());
      const f = BigInt($('floor_price').value.trim());
      const d = p - f;
      if (d < 0n) {
        el.textContent = 'Price paid is below the floor — the premium would be negative.';
        el.className = 'hint err';
      } else {
        const usdc = (Number(d) / 100).toLocaleString('en-US',
          { minimumFractionDigits: 2, maximumFractionDigits: 2 });
        el.textContent = 'Premium = price paid − floor = ' + d.toString() + ' cents (' + usdc + ' USDC).';
        el.className = 'hint';
      }
    } catch (e) {
      el.textContent = 'Enter integer amounts in USDC cents.';
      el.className = 'hint err';
    }
  }

  function validatePayout(showError) {
    const v = $('payout_hex').value.trim().toLowerCase().replace(/^0x/, '');
    const ok = /^[0-9a-f]{64}$/.test(v);
    $('payout_hex').setAttribute('aria-invalid', ok ? 'false' : 'true');
    $('payout-err').style.display = ok || !showError ? 'none' : 'block';
    return ok;
  }

  // ============================================================
  // Poseidon init (circomlibjs 0.1.7 — MISMO bundle que el circuito)
  // ============================================================
  async function ensurePoseidon() {
    if (_pose) return;
    if (!window.__cljs || typeof window.__cljs.buildPoseidon !== 'function') {
      throw new Error('circomlibjs not loaded: window.__cljs.buildPoseidon missing');
    }
    const p = await window.__cljs.buildPoseidon();
    _F     = p.F;
    const o = (x) => _F.toObject(x);
    _pose  = (arr) => o(p(arr.map(BigInt)));
  }

  // ============================================================
  // Build witness input (mismas reglas que circuits/gen_input.js)
  // ============================================================
  function readForm() {
    const get = (k) => $(k).value.trim();
    const payoutHex = get('payout_hex').toLowerCase().replace(/^0x/, '');
    // Regex (no solo length): parseInt de un char no-hex da NaN -> byte 0 silencioso -> payout en ceros.
    if (!/^[0-9a-f]{64}$/.test(payoutHex)) {
      throw new Error('payout public key must be exactly 64 hex characters (32 bytes)');
    }
    return {
      certifier_pk: [get('certifier_pk0'), get('certifier_pk1'), get('certifier_pk2')],
      attest_data:  [get('attest_data0'),  get('attest_data1')],
      price_paid:   get('price_paid'),
      floor_price:  get('floor_price'),
      lot_id:       get('lot_id'),
      season_id:    get('season_id'),
      lot_secret:   get('lot_secret'),
      payout_hex:   payoutHex,
    };
  }

  async function buildWitnessInput(f) {
    const lot_id     = BigInt(f.lot_id);
    const season_id  = BigInt(f.season_id);
    const lot_secret = BigInt(f.lot_secret);
    const price_paid = BigInt(f.price_paid);
    const floor_price= BigInt(f.floor_price);
    if (price_paid < floor_price) {
      throw new Error('price_paid is below floor_price (negative premium) — fix the pricing fields');
    }
    const premium_amount = price_paid - floor_price;

    const lot_commit     = _pose([lot_id, season_id]);
    const nullifier_hash = _pose([lot_secret, season_id]);

    // payout_hi/payout_lo: ed25519 pubkey 32B -> 2×16B BE
    const pub32 = hexToBytes(f.payout_hex);
    if (pub32.length !== 32) throw new Error('payout_hex: expected 32 bytes');
    const payout_hi = bytesToBigIntBE(pub32.slice(0, 16));
    const payout_lo = bytesToBigIntBE(pub32.slice(16, 32));

    const certifier_pk = f.certifier_pk.map(BigInt);
    const attest_data  = f.attest_data.map(BigInt);
    if (certifier_pk.length !== 3) throw new Error('certifier_pk must have 3 entries');
    if (attest_data.length  !== 2) throw new Error('attest_data must have 2 entries');

    // Ola 3 (role-tag): cada leaf con su constante de rol
    const leaves = [
      _pose([certifier_pk[0], ROLE_COOP,     lot_id, season_id, price_paid, lot_secret]),
      _pose([certifier_pk[1], ROLE_FINCA,    lot_id, attest_data[0]]),
      _pose([certifier_pk[2], ROLE_TOSTADOR, lot_id, attest_data[1]]),
    ];

    // Árbol: 3 hojas en índices 0,1,2; resto = 0n (BN254 zero)
    const idxs = [0, 1, 2];
    let level = new Array(NLEAVES).fill(0n);
    for (let k = 0; k < idxs.length; k++) level[idxs[k]] = leaves[k];

    // Merkle path por hoja: (pathElements, pathIndices) con la convención
    // del circuito: pathIndices[d]=0 => cur LEFT => hash(cur, sib)
    function merklePath(index) {
      const pathElements = [];
      const pathIndices  = [];
      let cur = level.slice();
      let ix  = index;
      for (let d = 0; d < LEVELS; d++) {
        const sibIx = ix ^ 1;
        pathElements.push(cur[sibIx].toString());
        pathIndices.push(ix & 1);
        const next = new Array(cur.length >> 1);
        for (let j = 0; j < next.length; j++) {
          next[j] = _pose([cur[2 * j], cur[2 * j + 1]]);
        }
        cur = next;
        ix >>= 1;
      }
      return { pathElements, pathIndices };
    }

    // raíz
    let cur = level.slice();
    while (cur.length > 1) {
      const next = new Array(cur.length >> 1);
      for (let j = 0; j < next.length; j++) next[j] = _pose([cur[2 * j], cur[2 * j + 1]]);
      cur = next;
    }
    const r_cert = cur[0];

    // Sanity: cada path debe re-derivar la raíz in-código (anti-tampering)
    for (let i = 0; i < 3; i++) {
      const { pathElements, pathIndices } = merklePath(idxs[i]);
      let c = leaves[i];
      for (let d = 0; d < LEVELS; d++) {
        const sib = BigInt(pathElements[d]);
        c = pathIndices[d] === 0 ? _pose([c, sib]) : _pose([sib, c]);
      }
      if (c !== r_cert) throw new Error('Merkle path ' + i + ' does not recompute the root');
    }

    const paths = idxs.map(merklePath);

    return {
      // públicos (orden Decisión A, congelado)
      r_cert:         r_cert.toString(),
      floor_price:    floor_price.toString(),
      lot_commit:     lot_commit.toString(),
      premium_amount: premium_amount.toString(),
      payout_hi:      payout_hi.toString(),
      payout_lo:      payout_lo.toString(),
      nullifier_hash: nullifier_hash.toString(),
      // privados
      lot_id:     lot_id.toString(),
      season_id:  season_id.toString(),
      lot_secret: lot_secret.toString(),
      price_paid: price_paid.toString(),
      certifier_pk: certifier_pk.map((x) => x.toString()),
      attest_data:  attest_data.map((x) => x.toString()),
      pathElements: paths.map((p) => p.pathElements),
      pathIndices:  paths.map((p) => p.pathIndices),
    };
  }

  // ============================================================
  // Serialization: snarkjs Groth16 -> BN254/EIP-197 layout (Soroban).
  // Idéntico a circuits/serialize.js pero sin fs.
  //   G1  : be32(x) || be32(y)                       (64 bytes)
  //   G2  : Fp2(x) || Fp2(y),  Fp2(c) = be32(c1) || be32(c0)   (128 bytes)  <- SWAP
  //   Fr  : be32(value)                              (32 bytes)
  // ============================================================
  function be32(decStr) {
    let h = BigInt(decStr).toString(16);
    if (h.length > 64) throw new Error('field element overflow: ' + decStr);
    return h.padStart(64, '0');
  }
  const g1  = (p) => be32(p[0]) + be32(p[1]);
  const fp2 = (c) => be32(c[1]) + be32(c[0]); // SWAP
  const g2  = (p) => fp2(p[0]) + fp2(p[1]);

  function serialize(proof, pubSignals) {
    return {
      proof: { a: g1(proof.pi_a), b: g2(proof.pi_b), c: g1(proof.pi_c) },
      pub_signals: pubSignals.map(be32),
    };
  }

  // ============================================================
  // QR rendering (qrcode-generator UMD: window.qrcode)
  // ============================================================
  function renderQR(canvas, text) {
    if (!window.qrcode) throw new Error('qrcode-generator not loaded');
    // typeNumber=0 => auto-detect size; errorCorrectionLevel='M' is fine for short URLs.
    const qr = window.qrcode(0, 'M');
    qr.addData(text);
    qr.make();
    // Use createSvgTag (string) instead of canvas to avoid canvas pollution in headless tests.
    const cellSize = 4, margin = 2;
    return qr.createSvgTag({ cellSize, margin, scalable: true });
  }

  // ============================================================
  // Stellar command (claim_premium) — read-only, copy-paste.
  // El proving —lo ZK— ya ocurrió en el navegador. La firma es plomería.
  // ============================================================
  function buildClaimCommand(contract, payoutG, pubSignals, ser) {
    // Formato EXACTO que espera `claim_premium` (verificado on-chain):
    //   --proof       objeto  { "a": <64B hex>, "b": <128B hex>, "c": <64B hex> }
    //   --pub_signals Array<u256> en DECIMAL (los pubSignals crudos de snarkjs)
    // (Los bytes hex son solo para el proof; los public inputs van en decimal.)
    const proofObj = JSON.stringify({ a: ser.proof.a, b: ser.proof.b, c: ser.proof.c });
    const pubDec   = JSON.stringify(pubSignals.map((x) => String(x)));
    const args = [
      '--proof',       "'" + proofObj + "'",
      '--pub_signals', "'" + pubDec + "'",
      '--payout',      payoutG,
    ];
    const head = `stellar contract invoke --id ${contract} --network ${_network} --source <YOUR_ID> --send=yes -- claim_premium`;
    return head + ' ' + args.map((a, i) => i % 2 === 0 ? '\n  ' + a : ' ' + a).join('');
  }

  function buildVerifyPayload(contract, lotCommitHex) {
    // Mismo formato que verify/gen_qr.sh: zkterroir:verify?lot_commit=...&contract=...&network=...
    return `zkterroir:verify?lot_commit=${lotCommitHex}&contract=${contract}&network=${_network}`;
  }

  // ============================================================
  // Main flow
  // ============================================================
  async function loadContract() {
    try {
      const r = await fetch('./public/deployment.json');
      if (!r.ok) throw new Error('HTTP ' + r.status);
      const j = await r.json();
      _contract = j.terroir_contract;
      _network  = j.network || 'testnet';
    } catch (e) {
      console.warn('Could not read deployment.json:', e.message, '— using static fallback');
    }
    if (!_contract) {
      // Fallback: lo que hay en deployments/testnet.json. Honesto si el archivo
      // no se sirve (la página es 100% estática y se monta en `web/`).
      _contract = 'CDECOLH6DVMVRLZV4ECNL7ZT4XDAGNJJBP4RXSLGNN4UTSVVYN7SH4O7';
      _network  = 'testnet';
    }
    $('contract-id').textContent = _contract;
    $('contract-network').textContent = _network;
  }

  function setFormDefaults() {
    $('certifier_pk0').value = DEFAULTS.certifier_pk0;
    $('certifier_pk1').value = DEFAULTS.certifier_pk1;
    $('certifier_pk2').value = DEFAULTS.certifier_pk2;
    $('attest_data0').value  = DEFAULTS.attest_data0;
    $('attest_data1').value  = DEFAULTS.attest_data1;
    $('price_paid').value    = DEFAULTS.price_paid;
    $('floor_price').value   = DEFAULTS.floor_price;
    $('lot_id').value        = DEFAULTS.lot_id;
    $('season_id').value     = DEFAULTS.season_id;
    $('lot_secret').value    = DEFAULTS.lot_secret;
    $('payout_hex').value    = DEFAULTS.payout_hex;
    updatePremiumHint();
    validatePayout(false);
  }

  const fmtUSDC = (cents) => (Number(cents) / 100).toLocaleString('en-US',
    { minimumFractionDigits: 2, maximumFractionDigits: 2 }) + ' USDC';

  function showResults({ input, proof, pubSignals, ser }) {
    // pubSignals: Decisión A
    const [rCert, floorPrice, lotCommit, premium, payHi, payLo, nullHash] = pubSignals;
    const rCertHex    = BigInt(rCert).toString(16).padStart(64, '0');
    const lotCommitHex= BigInt(lotCommit).toString(16).padStart(64, '0');
    const nullHashHex = BigInt(nullHash).toString(16).padStart(64, '0');

    $('out-r_cert').textContent       = rCertHex;
    $('out-lot_commit').textContent   = lotCommitHex;
    $('out-nullifier').textContent    = nullHashHex;
    $('out-premium').textContent      = fmtUSDC(premium);
    $('out-premium-cents').textContent= premium;
    $('out-floor').textContent        = fmtUSDC(floorPrice);
    $('out-payout-hi').textContent    = BigInt(payHi).toString(16).padStart(32, '0');
    $('out-payout-lo').textContent    = BigInt(payLo).toString(16).padStart(32, '0');

    // Stellar command
    const cmd = buildClaimCommand(_contract, '<G...PayoutPubkey>', pubSignals, ser);
    $('cmd').textContent = cmd;
    $('cmd-hex').textContent = JSON.stringify({
      proof: { a: ser.proof.a, b: ser.proof.b, c: ser.proof.c },
      pub_signals: ser.pub_signals,
    }, null, 2);

    // QR
    const payload = buildVerifyPayload(_contract, lotCommitHex);
    $('qr-payload').textContent = payload;
    const qrHost = $('qr');
    qrHost.innerHTML = renderQR(null, payload);
    setSection('result', true);
  }

  async function onProve() {
    setBusy(true);
    setSection('result', false);
    setProofBadge('', '');
    setBanner('', '');
    resetSteps();
    logReset();
    let cur = null;
    const step = (name) => { setStep(name, 'active'); cur = name; };
    const done = () => { if (cur) setStep(cur, 'done'); cur = null; };
    try {
      step('setup');
      logLine('→ Initializing Poseidon (BN254) — circomlibjs 0.1.7, the same bundle that produced the VK (~2 MB)…', 'work');
      await ensurePoseidon();
      logLine('✓ Poseidon ready — constants identical to the circuit.', 'ok');
      done();

      step('witness');
      logLine('→ Building the witness input from the form…', 'work');
      const form = readForm();
      const input = await buildWitnessInput(form);
      _builtInput = input;
      logLine('  r_cert  = ' + input.r_cert.slice(0, 24) + '…', 'dim');
      logLine('  premium = ' + input.premium_amount + ' cents', 'dim');
      done();

      // Honest check: ¿el r_cert del usuario coincide con el sembrado on-chain?
      const seedRcert = BigInt(SEED_R_CERT_HEX);
      const userRcert = BigInt(input.r_cert);
      if (userRcert !== seedRcert) {
        setBanner(
          'Warning — outside the accredited set: your inputs produce r_cert ' +
          input.r_cert.slice(0, 20) + '…, but the on-chain seed is ' +
          SEED_R_CERT_HEX.slice(0, 20) + '…. The proof will still be generated, but the ' +
          'contract will REJECT it at the root == stored check.',
          'warn'
        );
      } else {
        setBanner('Inputs match the accredited set — the proof will match the r_cert seeded on-chain.', 'ok');
      }

      step('prove');
      logLine('→ Generating the witness + Groth16 proof in your browser (wasm 3.4 MB + zkey 9 MB)…', 'work');
      logLine('  Private inputs never leave this device.', 'dim');
      const t0 = performance.now();
      const r = await snarkjs.groth16.fullProve(
        input, './public/terroir_chain.wasm', './public/terroir_chain_0001.zkey'
      );
      const dt = ((performance.now() - t0) / 1000).toFixed(1);
      _proof = r.proof;
      _pubSignals = r.publicSignals;
      logLine('✓ Proof generated in ' + dt + ' s.', 'ok');
      logLine('  r_cert    = ' + _pubSignals[0].slice(0, 24) + '…', 'dim');
      logLine('  premium   = ' + _pubSignals[3], 'dim');
      logLine('  nullifier = ' + _pubSignals[6].slice(0, 24) + '…', 'dim');
      done();

      step('verify');
      logLine('→ Verifying off-chain against the deployed verification key (same VK as the contract)…', 'work');
      const vk = await (await fetch('./public/verification_key.json')).json();
      const ok = await snarkjs.groth16.verify(vk, _pubSignals, _proof);
      logLine(ok ? '✓ VERIFY OK — the proof is valid against the deployed VK.'
                 : '✗ VERIFY FAILED off-chain — the contract would reject this proof.',
              ok ? 'ok' : 'err');
      setProofBadge(ok ? 'ok' : 'err', ok ? 'Proof valid (off-chain)' : 'Proof invalid');
      setStep('verify', ok ? 'done' : 'error');
      cur = null;

      step('serialize');
      logLine('→ Encoding proof + public signals into the BN254 layout Soroban expects…', 'work');
      _serialized = serialize(_proof, _pubSignals);
      logLine('  proof.a     = ' + _serialized.proof.a.slice(0, 32) + '…  (64 bytes)', 'dim');
      logLine('  proof.b     = ' + _serialized.proof.b.slice(0, 32) + '…  (128 bytes)', 'dim');
      logLine('  proof.c     = ' + _serialized.proof.c.slice(0, 32) + '…  (64 bytes)', 'dim');
      logLine('  pub_signals = 7×32 B (Decision A)', 'dim');
      done();

      logLine('✓ Done — paste the command below into your shell to submit the claim transaction.', 'ok');
      showResults({ input, proof: _proof, pubSignals: _pubSignals, ser: _serialized });

      // Sonda para el browser test (Puppeteer): exponer lo esencial.
      window.__ZKT_RESULT__ = {
        ok,
        dtSeconds: Number(dt),
        rCert:    _pubSignals[0],
        lotCommit:_pubSignals[2],
        premium:  _pubSignals[3],
        nullifier:_pubSignals[6],
        pubSignals: _pubSignals,
        serialized: _serialized,
      };
    } catch (e) {
      console.error(e);
      if (cur) setStep(cur, 'error');
      logLine('✗ Error: ' + (e && e.message ? e.message : e), 'err');
      setProofBadge('err', 'Pipeline failed');
      window.__ZKT_RESULT__ = { ok: false, error: String(e && e.message || e) };
    } finally {
      setBusy(false);
    }
  }

  // ============================================================
  // Utils
  // ============================================================
  function hexToBytes(hex) {
    if (hex.length % 2 !== 0) throw new Error('odd hex length');
    const out = new Uint8Array(hex.length / 2);
    for (let i = 0; i < out.length; i++) {
      out[i] = parseInt(hex.substr(i * 2, 2), 16);
    }
    return out;
  }
  function bytesToBigIntBE(b) {
    let s = '';
    for (let i = 0; i < b.length; i++) s += b[i].toString(16).padStart(2, '0');
    return BigInt('0x' + s);
  }

  // ============================================================
  // Copy buttons
  // ============================================================
  function wireCopyButton(btnId, srcId) {
    const b = $(btnId);
    const label = b.querySelector('.copy-text');
    b.addEventListener('click', async () => {
      const text = $(srcId).textContent;
      try {
        await navigator.clipboard.writeText(text);
      } catch (e) {
        // Fallback: select+execCommand
        const r = document.createRange();
        r.selectNode($(srcId));
        const sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(r);
        try { document.execCommand('copy'); } catch (_) {}
      }
      label.textContent = 'Copied ✓';
      b.classList.add('copied');
      setTimeout(() => { label.textContent = 'Copy'; b.classList.remove('copied'); }, 1400);
    });
  }

  // ============================================================
  // Boot
  // ============================================================
  window.addEventListener('DOMContentLoaded', async () => {
    setFormDefaults();
    await loadContract();
    $('prove').addEventListener('click', onProve);
    $('reset').addEventListener('click', () => {
      setFormDefaults();
      resetSteps();
      logReset();
      logLine('Form restored to the demo defaults (matches the r_cert seeded on-chain).');
      setSection('result', false);
      setProofBadge('', '');
      setBanner('', '');
    });
    $('price_paid').addEventListener('input', updatePremiumHint);
    $('floor_price').addEventListener('input', updatePremiumHint);
    $('payout_hex').addEventListener('blur', () => validatePayout(true));
    $('payout_hex').addEventListener('input', () => validatePayout(false));
    wireCopyButton('copy-cmd', 'cmd');
    wireCopyButton('copy-qr-payload', 'qr-payload');
    wireCopyButton('copy-hex', 'cmd-hex');
    logLine('Ready — click "Generate proof" to run the full pipeline locally (~1 s in Chrome).');
  });
})();
