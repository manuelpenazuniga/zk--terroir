// Para cada uno de los 7 públicos: corrompe +1 y afirma que verify() = false.
const {snarkjs} = (() => { try { return {snarkjs: require('/home/manuel/.nvm/versions/node/v24.17.0/lib/node_modules/snarkjs')}; } catch(e){ return {}; } })();
const snarkjs = require('snarkjs') || (globalThis.snarkjs);
