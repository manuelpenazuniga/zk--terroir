#!/usr/bin/env node
// Ataque de doble-cobro contra terroir_chain v3:
// Atacante quiere un nullifier DISTINTO para el MISMO leaf_0 acreditado, variando season_id.
// Recomputa lot_commit' y nullifier' para season'=season+1 (para pasar lc/nh),
// PERO mantiene r_cert y los Merkle paths reales. Si el binding de season_id en leaf_0
// funciona, la membership de leaf_0 debe romperse -> witness gen FALLA.
const fs = require('fs');
const { buildPoseidon } = require('/home/manuel/proyectos/zk-terroir/spike/node_modules/circomlibjs');
const DIR = '/home/manuel/proyectos/zk-terroir/circuits';

(async () => {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;
  const o = (x) => F.toObject(x);
  const pose = (a) => o(poseidon(a.map(BigInt)));

  const inp = JSON.parse(fs.readFileSync(DIR + '/input.json', 'utf8'));
  const season2 = (BigInt(inp.season_id) + 1n).toString();

  const attack = { ...inp,
    season_id: season2,
    // recomputa los públicos derivados para que lc/nh NO sean el punto de fallo:
    lot_commit: pose([BigInt(inp.lot_id), BigInt(season2)]).toString(),
    nullifier_hash: pose([BigInt(inp.lot_secret), BigInt(season2)]).toString(),
    // r_cert y pathElements/pathIndices SE MANTIENEN (el árbol real no cambia).
  };
  fs.writeFileSync(DIR + '/input_attack.json', JSON.stringify(attack, null, 2));
  console.log('attack: season', inp.season_id, '->', season2,
    '| r_cert y paths intactos | lot_commit/nullifier recomputados');
})();
