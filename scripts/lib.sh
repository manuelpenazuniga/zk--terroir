#!/usr/bin/env bash
# Shared helpers for scripts/setup_token.sh and scripts/deploy.sh.
# Reusa identidades/conocimientos del spike (PLAN-DIA-2 §2 Decisiones G, H, I).
# NO escribe lógica de dominio — sólo plomería de Stellar CLI + JSON.
set -euo pipefail

# ---------- paths / config ----------
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOY_DIR="${REPO_ROOT}/deployments"
DEPLOY_FILE="${DEPLOY_DIR}/testnet.json"
NETWORK="${NETWORK:-testnet}"
SOURCE_IDENTITY="${SOURCE_IDENTITY:-terroir}"   # admin/issuer; ya funded en testnet
ASSET_CODE="${ASSET_CODE:-TUSDC}"                # ≤12 chars, alph+num (Decisión G)

# Tasas razonables: el fee de testnet se mide en stroops.
INCLUSION_FEE="${INCLUSION_FEE:-100000}"   # 0.01 XLM, margen de seguridad
RESOURCE_FEE="${RESOURCE_FEE:-5000000}"    # 0.5 XLM, cubre el SAC + ops

mkdir -p "${DEPLOY_DIR}"

# ---------- pretty logging ----------
log()  { printf '\033[1;36m[setup]\033[0m %s\n' "$*" >&2; }
warn() { printf '\033[1;33m[warn ]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[fatal]\033[0m %s\n' "$*" >&2; exit 1; }

# ---------- stellar wrappers (network global) ----------
# Pasamos --network global; usan $SOURCE_IDENTITY por defecto si no se pasa --source.
scl() { stellar "$@" --network "${NETWORK}"; }

# Lee la public key (G...) de una identidad Stellar. Falla si no existe.
pubkey_of() {
  local ident="$1"
  stellar keys address "${ident}" 2>/dev/null \
    || die "identidad '${ident}' no existe (stellar keys ls)"
}

# ---------- json store ----------
# init_json: crea el archivo si no existe.
init_json() {
  if [[ ! -f "${DEPLOY_FILE}" ]]; then
    jq -n --arg net "${NETWORK}" '{network:$net, updated_at:null, addresses:{}}' \
      > "${DEPLOY_FILE}"
    log "creado ${DEPLOY_FILE}"
  fi
}

# Convierte "a.b.c" -> array JSON ["a","b","c"] para jq setpath/getpath.
_path_to_jq_array() {
  local IFS='.'
  local -a parts=( $1 )
  printf '%s\n' "${parts[@]}" | jq -R . | jq -s 'map(.|tostring)'
}

# json_set <key.path> <value>   value se trata como string (jq --arg).
json_set() {
  local path="$1" value="$2"
  local arr; arr="$(_path_to_jq_array "${path}")"
  local tmp; tmp="$(mktemp)"
  jq --arg v "${value}" --argjson p "${arr}" --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    'setpath($p; $v) | .updated_at=$ts' "${DEPLOY_FILE}" > "${tmp}"
  mv "${tmp}" "${DEPLOY_FILE}"
}

# json_set_obj <key.path> <json-string>
json_set_obj() {
  local path="$1" value="$2"
  local arr; arr="$(_path_to_jq_array "${path}")"
  local tmp; tmp="$(mktemp)"
  jq --argjson v "${value}" --argjson p "${arr}" --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    'setpath($p; $v) | .updated_at=$ts' "${DEPLOY_FILE}" > "${tmp}"
  mv "${tmp}" "${DEPLOY_FILE}"
}

# json_get <key.path> -> stdout
json_get() {
  local arr; arr="$(_path_to_jq_array "$1")"
  jq -r --argjson p "${arr}" 'getpath($p) // empty' "${DEPLOY_FILE}"
}

# ---------- balances (read-only) ----------
# Lee el balance nativo XLM y el balance del asset code (si existe) para una address.
# Imprime "XLM=..;ASSET=.." en stdout. Si no hay trustline de asset, ASSET=0.
balances_of() {
  local addr="$1"
  local url="https://horizon-testnet.stellar.org/accounts/${addr}"
  local json
  json="$(curl -fsS "${url}")" || die "Horizon no responde: ${url}"

  local xlm asset_bal
  xlm="$(jq -r '.balances[] | select(.asset_type=="native") | .balance' <<<"${json}" || echo 0)"
  asset_bal="$(jq -r --arg c "${ASSET_CODE}" \
    '.balances[]? | select(.asset_type!="native" and .asset_code==$c) | .balance // "0"' \
    <<<"${json}" | head -n1)"
  [[ -z "${asset_bal}" ]] && asset_bal=0
  printf 'XLM=%s;%s=%s\n' "${xlm}" "${ASSET_CODE}" "${asset_bal}"
}

# balance de un token SAC (smart contract) por address.
sac_balance() {
  local sac_id="$1" addr="$2"
  stellar contract invoke \
    --id "${sac_id}" \
    --source "${SOURCE_IDENTITY}" \
    --send no --network "${NETWORK}" -- \
    balance --id "${addr}" 2>/dev/null \
    | tail -n1 \
    | tr -d '"'
}

# ---------- fee / cost helpers ----------
cost_last_tx() {
  # Imprime los stroops de CPU+resources de la última tx submitted.
  # Útil para anexar a deployments/testnet.json.
  stellar --quiet --network "${NETWORK}" tx cost \
    "$(scl --help 2>/dev/null; echo)" 2>/dev/null || true
}

# ---------- require ----------
require_tools() {
  for t in stellar curl jq; do
    command -v "${t}" >/dev/null || die "falta tool: ${t}"
  done
  stellar network ls 2>/dev/null | grep -q "^${NETWORK}$" \
    || die "red '${NETWORK}' no configurada en stellar CLI"
  stellar keys ls 2>/dev/null | grep -q "^${SOURCE_IDENTITY}$" \
    || die "identidad '${SOURCE_IDENTITY}' no existe"
  log "tools OK (network=${NETWORK}, source=${SOURCE_IDENTITY})"
}
