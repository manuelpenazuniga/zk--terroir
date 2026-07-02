#!/usr/bin/env bash
# ZK-Terroir — verificador público de solo lectura (T3D-verify, Ola 1).
#
# Dado un `lot_commit` (hex de 32 bytes), consulta `lot_status(lot_commit)` del
# contrato `terroir` en Testnet y dice si el lote fue certificado y su premium
# pagado. SOLO LECTURA: usa `--send=no` (simulación); NO firma ni muta estado,
# no mueve fondos, no requiere claves privadas con permisos de escritura.
#
# El contract id / red se leen de deployments/testnet.json (nunca hardcodeados):
# si ese archivo cambia, el verificador sigue el valor de ahí.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
DEPLOY="$ROOT/deployments/testnet.json"

NETWORK="${STELLAR_NETWORK:-testnet}"
# Identidad usada solo para la simulación (--send=no no firma ni paga fee).
# Pásala por env si no tienes la identidad local 'terroir':  STELLAR_SOURCE=<tu_id>
SOURCE="${STELLAR_SOURCE:-terroir}"

usage() {
  echo "uso: $0 <lot_commit_hex_64>" >&2
  echo "  ej: $0 2ceda2ee11f38491b484858a98c200d48c97ce21fdf8e9217a62634de6da6563" >&2
  exit 2
}

[ $# -eq 1 ] || usage
LOT_COMMIT="${1#0x}"
LOT_COMMIT="$(printf '%s' "$LOT_COMMIT" | tr 'A-F' 'a-f')"
if ! [[ "$LOT_COMMIT" =~ ^[0-9a-f]{64}$ ]]; then
  echo "error: lot_commit debe ser 64 chars hex (32 bytes)" >&2
  exit 2
fi

command -v stellar >/dev/null || { echo "error: falta el CLI 'stellar'" >&2; exit 1; }
command -v jq      >/dev/null || { echo "error: falta 'jq'" >&2; exit 1; }
[ -f "$DEPLOY" ] || { echo "error: no existe $DEPLOY" >&2; exit 1; }

CID="$(jq -r '.addresses.terroir_contract' "$DEPLOY")"
[ -n "$CID" ] && [ "$CID" != "null" ] || { echo "error: terroir_contract no está en $DEPLOY" >&2; exit 1; }

echo "contrato  : $CID  ($NETWORK)"
echo "lot_commit: $LOT_COMMIT"

OUT="$(stellar contract invoke --id "$CID" --network "$NETWORK" --source "$SOURCE" --send=no \
        -- lot_status --lot_commit "$LOT_COMMIT" 2>/dev/null || true)"
OUT="$(printf '%s' "$OUT" | tr -d '"[:space:]')"

if [[ "$OUT" =~ ^[0-9]+$ ]]; then
  WHEN="$(date -u -d "@$OUT" '+%Y-%m-%d %H:%M:%S UTC' 2>/dev/null || echo "ts=$OUT")"
  echo "✓ Lote certificado — premium pagado el $WHEN"
  exit 0            # encontrado
elif [ "$OUT" = "null" ] || [ -z "$OUT" ]; then
  echo "✗ No encontrado / no reclamado"
  exit 1            # no encontrado (respuesta válida, no error)
else
  echo "? respuesta inesperada del contrato: $OUT" >&2
  exit 3            # error real (red / CLI)
fi
