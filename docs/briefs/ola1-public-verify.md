# BRIEF Ola 1 — Verificador público (QR / lot_status) · WORKER (escribe)

> Antepón `docs/briefs/_wrapper.md`. Trabajas en el worktree que te pasa el orquestador. **NO tocas
> el contrato ni el circuito** (son solo-lectura para ti): solo lees on-chain. **No toca fondos.**

## Objetivo
Un verificador de **solo lectura** que permita a un consumidor (o un juez) comprobar, escaneando un
QR, que un lote de café fue **certificado y su premium pagado**, sin revelar la cadena de suministro.

## Qué construir (en `verify/`)
1. Un script/página que reciba un `lot_commit` (string hex de 32 bytes) y llame a
   **`lot_status(lot_commit) -> Option<u64>`** del contrato `terroir` en **Testnet**.
   - Contract ID, red y token: **léelos de `deployments/testnet.json`** (`addresses.terroir_contract`).
     No hardcodees direcciones; si el archivo cambia, el verificador debe seguir el valor de ahí.
   - Invocación de referencia (CLI):
     `stellar contract invoke --id <terroir_contract> --network testnet --source <cuenta> -- lot_status --lot_commit <hex>`
     (o `--send=no` / read-only si tu versión lo soporta; es una consulta, no muta estado).
   - Alternativa Node: usa `@stellar/stellar-sdk` para simular la invocación y leer el retorno.
2. Presentación mínima: si devuelve un timestamp → "✓ Lote certificado — premium pagado el
   <fecha del timestamp>"; si `None` → "✗ No encontrado / no reclamado".
3. Genera (o documenta cómo generar) un **QR** que codifique la URL/parámetro con el `lot_commit`
   (una lib de QR o incluso un `.md` con el comando; mantenlo simple y sin dependencias pesadas).

## Restricciones
- **Solo lectura on-chain.** Nada de claves privadas, nada de firmar, nada de mutar estado.
- Sin secretos en el repo. Si necesitas una cuenta para `--source`, usa una pública de solo-consulta
  o documenta que el usuario pase la suya por env var.
- Mantén el árbol limpio: todo bajo `verify/`; no toques `contracts/`, `circuits/`, `scripts/`.

## Criterios de aceptación
- Con el `lot_commit` del E2E real (existe en Testnet: `deployments/testnet.json` registra
  `e2e.lot_status_registered`), el verificador imprime el timestamp correcto.
- Con un `lot_commit` inventado → "No encontrado".
- README corto en `verify/README.md`: cómo correrlo y cómo se genera el QR.
- Corre en esta máquina (bash, node disponible) sin instalar toolchains nuevos pesados.

## Al terminar
Corre tu propio smoke-test contra Testnet, deja el árbol verde, **haz commit**, y resume qué hiciste
+ supuestos (marca `// TODO(verify)` lo que no pudiste confirmar). Termina con:
`VEREDICTO: LISTO — <una línea>` (esto lo revisa luego el auditor, no es tu autoaprobación).
</content>
