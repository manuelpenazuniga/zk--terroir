#!/usr/bin/env bash
# ZK-Terroir — genera un QR de verificación para un lot_commit (T3D-verify).
#
# El QR codifica un payload de texto autoexplicativo que apunta a la consulta
# read-only del lote:
#   zkterroir:verify?lot_commit=<hex>&contract=<CID>&network=<red>
# Un consumidor/juez lo escanea y corre `verify.sh <lot_commit>` (o una página
# que llame a lot_status). Si 'qrencode' está instalado produce un PNG; si no,
# imprime el payload + instrucciones (sin dependencias pesadas).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
DEPLOY="$ROOT/deployments/testnet.json"
NETWORK="${STELLAR_NETWORK:-testnet}"

[ $# -ge 1 ] || { echo "uso: $0 <lot_commit_hex_64> [salida.png]" >&2; exit 2; }
LOT_COMMIT="${1#0x}"
LOT_COMMIT="$(printf '%s' "$LOT_COMMIT" | tr 'A-F' 'a-f')"
[[ "$LOT_COMMIT" =~ ^[0-9a-f]{64}$ ]] || { echo "error: lot_commit debe ser 64 hex (32 bytes)" >&2; exit 2; }

command -v jq >/dev/null || { echo "error: falta 'jq'" >&2; exit 1; }
[ -f "$DEPLOY" ] || { echo "error: no existe $DEPLOY" >&2; exit 1; }
CID="$(jq -r '.addresses.terroir_contract' "$DEPLOY")"

OUT_PNG="${2:-$HERE/lot-${LOT_COMMIT:0:12}.png}"
PAYLOAD="zkterroir:verify?lot_commit=$LOT_COMMIT&contract=$CID&network=$NETWORK"

echo "payload QR: $PAYLOAD"
if command -v qrencode >/dev/null; then
  qrencode -o "$OUT_PNG" "$PAYLOAD"
  echo "QR escrito en: $OUT_PNG"
else
  echo "(qrencode no instalado — el payload de arriba es lo que va en el QR)"
  echo "para generar el PNG:"
  echo "  sudo apt install qrencode     # o en macOS: brew install qrencode"
  echo "  qrencode -o lote.png '$PAYLOAD'"
  echo "  # o pega el payload en cualquier generador de QR."
fi
