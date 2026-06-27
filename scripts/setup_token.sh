#!/usr/bin/env bash
# T5 — Token TUSDC (SEP-41) para zk-terroir
#   1. crea el asset clásico TUSDC emitido por $SOURCE_IDENTITY
#   2. deploya el SAC (Stellar Asset Contract) sobre ese asset
#   3. mintea `MINT_AMOUNT` unidades a la dirección de escrow (env: ESCROW_ADDRESS)
#
# Reusa:
#   - Decisión G (PLAN-DIA-2 §2): TUSDC = SAC de testnet, minteado al escrow
#   - El admin del SAC es el issuer (= $SOURCE_IDENTITY en testnet)
#
# Idempotencia: si el asset ya existe, lo detecta por `addresses.tusdc_sac` en
# deployments/testnet.json y reusa el contrato. Si ya hay mint previo al escrow,
# lo respeta y sólo mintea el déficit (delta = MINT_AMOUNT - balance_actual).
#
# Salidas (deployments/testnet.json):
#   addresses.tusdc_issuer        (G... del issuer)
#   addresses.tusdc_asset         (canonical: "TUSDC:G...")
#   addresses.tusdc_sac           (C... contrato SAC desplegado)
#   amounts.tusdc_escrow_minted   (delta minteado, en unidades mínimas)
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

MINT_AMOUNT="${MINT_AMOUNT:-1000000000000}"   # 1e12 unidades, con 7 decimales ≈ 100k USDC
DECIMALS="${DECIMALS:-7}"                     # USDC estándar
ESCROW_ADDRESS="${ESCROW_ADDRESS:-${1:-}}"   # pasar como arg o env
DRY_RUN="${DRY_RUN:-0}"

require_tools
init_json

# 1. resolver addresses
ISSUER_PK="$(pubkey_of "${SOURCE_IDENTITY}")"
log "issuer = ${SOURCE_IDENTITY} (${ISSUER_PK})"

if [[ -z "${ESCROW_ADDRESS}" ]]; then
  # Conveniencia: si el contrato terroir ya está desplegado, úsalo como escrow.
  # Si no, usa la misma cuenta (sirve como test de "balance en self-custody").
  local_escrow="$(json_get addresses.terroir_contract)"
  if [[ -n "${local_escrow}" && "${local_escrow}" != "null" ]]; then
    ESCROW_ADDRESS="${local_escrow}"
  else
    ESCROW_ADDRESS="${ISSUER_PK}"
    warn "ESCROW_ADDRESS no provisto y 'terroir_contract' no desplegado; minteando a issuer (${ISSUER_PK})"
  fi
fi
log "escrow = ${ESCROW_ADDRESS}"
log "mint target = ${MINT_AMOUNT} (decimals=${DECIMALS})"

ASSET_STR="${ASSET_CODE}:${ISSUER_PK}"
json_set addresses.tusdc_issuer   "${ISSUER_PK}"
json_set addresses.tusdc_asset    "${ASSET_STR}"

# 2. deploy SAC (idempotente: si ya existe, no redeploya)
SAC_ID="$(json_get addresses.tusdc_sac || true)"
if [[ -z "${SAC_ID}" || "${SAC_ID}" == "null" ]]; then
  log "deployando SAC para ${ASSET_STR}…"
  if [[ "${DRY_RUN}" == "1" ]]; then
    die "DRY_RUN: nada que hacer, el SAC no existe aún"
  fi
  SAC_ID="$(
    stellar contract asset deploy \
      --source "${SOURCE_IDENTITY}" \
      --asset "${ASSET_STR}" \
      --alias "${ASSET_CODE}_${NETWORK}" \
      --network "${NETWORK}" 2>/dev/null \
    | tail -n1
  )"
  [[ "${SAC_ID}" =~ ^C[A-Z0-9]{55}$ ]] || die "SAC deploy no devolvió contract id: '${SAC_ID}'"
  log "SAC desplegado: ${SAC_ID}"
  json_set addresses.tusdc_sac "${SAC_ID}"
else
  log "reusando SAC existente: ${SAC_ID}"
fi

# 3. inicializar SAC si hace falta (decimals/name/symbol/admin). El CLI `asset deploy`
#    ya lo hace en su primera transacción si el asset se acaba de crear. Pero si el
#    asset ya existía, el SAC puede no estar inicializado: el primer `mint` lo hace
#    con side-effects. Verificamos con `decimals`.
log "verificando inicialización del SAC…"
DEC_OUT="$(
  stellar contract invoke \
    --id "${SAC_ID}" \
    --source "${SOURCE_IDENTITY}" \
    --send no --network "${NETWORK}" -- \
    decimals 2>/dev/null | tail -n1 | tr -d '"'
)" || DEC_OUT=""
if [[ "${DEC_OUT}" != "${DECIMALS}" ]]; then
  log "inicializando SAC (decimals=${DECIMALS}, name=${ASSET_CODE}, symbol=${ASSET_CODE})…"
  if [[ "${DRY_RUN}" != "1" ]]; then
    stellar contract invoke \
      --id "${SAC_ID}" \
      --source "${SOURCE_IDENTITY}" \
      --network "${NETWORK}" -- \
      initialize \
        --admin "${ISSUER_PK}" \
        --decimal "${DECIMALS}" \
        --name "Terroir USDC" \
        --symbol "${ASSET_CODE}" \
      >/dev/null
  fi
else
  log "SAC ya inicializado (decimals=${DEC_OUT})"
fi

# 4. calcular delta de mint (idempotente: no minteamos de más)
CURRENT_ESCROW_BAL="$(
  stellar contract invoke \
    --id "${SAC_ID}" \
    --source "${SOURCE_IDENTITY}" \
    --send no --network "${NETWORK}" -- \
    balance --id "${ESCROW_ADDRESS}" 2>/dev/null | tail -n1 | tr -d '"' \
    || echo 0
)"
[[ -z "${CURRENT_ESCROW_BAL}" || "${CURRENT_ESCROW_BAL}" == "null" ]] && CURRENT_ESCROW_BAL=0
log "balance actual del escrow en ${ASSET_CODE}: ${CURRENT_ESCROW_BAL}"

# delta = max(0, MINT_AMOUNT - current), saturado a i128::MAX para no romper la SAC.
I128_MAX=9223372036854775807
DELTA="$(awk -v cur="${CURRENT_ESCROW_BAL}" -v tgt="${MINT_AMOUNT}" -v cap="${I128_MAX}" \
  'BEGIN{ d = tgt - cur; if (d<0) d=0; if (d+cur>cap) d=cap-cur; if (d<0) d=0; print d }')"

if [[ "${DELTA}" == "0" ]]; then
  log "escrow ya tiene >= MINT_AMOUNT (o saturado a i128::MAX); nada que mintear"
else
  log "minteando delta=${DELTA} a ${ESCROW_ADDRESS}…"
  if [[ "${DRY_RUN}" != "1" ]]; then
    if ! stellar contract invoke \
        --id "${SAC_ID}" \
        --source "${SOURCE_IDENTITY}" \
        --network "${NETWORK}" -- \
        mint --to "${ESCROW_ADDRESS}" --amount "${DELTA}" \
        >/dev/null 2>&1; then
      warn "mint falló (probable saturación i128); el escrow ya tiene balance >= premium"
    fi
  fi
  json_set_obj amounts.tusdc_escrow_minted "$(jq -n --argjson v "${DELTA}" '$v')"
fi

# 5. verificación final (read-only)
FINAL_ESCROW_BAL="$(
  stellar contract invoke \
    --id "${SAC_ID}" \
    --source "${SOURCE_IDENTITY}" \
    --send no --network "${NETWORK}" -- \
    balance --id "${ESCROW_ADDRESS}" 2>/dev/null | tail -n1 | tr -d '"' \
    || echo 0
)"
log "balance final escrow = ${FINAL_ESCROW_BAL} (target >= ${MINT_AMOUNT})"

if (( FINAL_ESCROW_BAL < MINT_AMOUNT )); then
  die "balance final (${FINAL_ESCROW_BAL}) < MINT_AMOUNT (${MINT_AMOUNT})"
fi
log "OK: escrow puede recibir payouts TUSDC"

# 6. resumen
echo
cat <<EOF
=== TUSDC setup OK (${NETWORK}) ===
  asset  : ${ASSET_STR}
  SAC    : ${SAC_ID}
  escrow : ${ESCROW_ADDRESS}
  minted : ${DELTA} (balance final ${FINAL_ESCROW_BAL})
EOF
