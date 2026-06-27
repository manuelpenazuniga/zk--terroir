#!/usr/bin/env bash
# T5 — Deploy del contrato escrow `terroir` (T3 entrega el .wasm).
# Reusa:
#   - spike/contract/Makefile + stellar contract build (mismo toolchain)
#   - Decisión F: VK del circuito de 3 eslabones HORNEADA en el contrato
#
# Salidas (deployments/testnet.json):
#   addresses.terroir_contract      (C... id del contrato desplegado)
#   addresses.admin                 (G... admin)
#   wasm.hash                       (sha256 del .wasm desplegado, para auditoría)
#   addresses.tusdc_sac              (NO lo crea; lo toma de testnet.json si ya existe)
#
# Si T3 aún no compiló, podés pasar DEPLOY_WASM=/ruta/al.wasm y el script sólo deploya.
# Por defecto intenta build desde contracts/terroir/.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

CONTRACT_CRATE_DIR="${CONTRACT_CRATE_DIR:-${REPO_ROOT}/contracts/terroir}"
DEPLOY_WASM="${DEPLOY_WASM:-}"              # override: saltar build
INIT_ARGS="${INIT_ARGS:-}"                  # args para `init`, ej: "--admin G... --token C..."

require_tools
init_json

# 1. wasm: build si hace falta
WASM_PATH="${DEPLOY_WASM}"
if [[ -z "${WASM_PATH}" ]]; then
  [[ -d "${CONTRACT_CRATE_DIR}" ]] \
    || die "crate no existe: ${CONTRACT_CRATE_DIR} (pasá DEPLOY_WASM=... si T3 aún no terminó)"
  log "compilando ${CONTRACT_CRATE_DIR} (stellar contract build)…"
  ( cd "${CONTRACT_CRATE_DIR}" && stellar contract build ) >&2
  # el nombre del .wasm sigue convención soroban: <crate_name>.wasm
  CRATE_NAME="$(grep -m1 '^name' "${CONTRACT_CRATE_DIR}/Cargo.toml" \
    | sed -E 's/^name\s*=\s*"([^"]+)".*/\1/' | tr '-' '_')"
  WASM_PATH="${CONTRACT_CRATE_DIR}/target/wasm32v1-none/release/${CRATE_NAME}.wasm"
fi
[[ -f "${WASM_PATH}" ]] || die "wasm no encontrado: ${WASM_PATH}"
WASM_HASH="$(sha256sum "${WASM_PATH}" | awk '{print $1}')"
log "wasm listo: ${WASM_PATH}  (sha256=${WASM_HASH:0:16}…)"

# 2. id del admin (la misma identidad SOURCE_IDENTITY por defecto; el contrato exige
#    que se pase al init)
ADMIN_PK="$(pubkey_of "${SOURCE_IDENTITY}")"
log "admin = ${SOURCE_IDENTITY} (${ADMIN_PK})"

# 3. deploy
log "deployando contrato a ${NETWORK}…"
ALIAS="terroir_${NETWORK}"
DEPLOY_OUT="$(
  stellar contract deploy \
    --source "${SOURCE_IDENTITY}" \
    --wasm "${WASM_PATH}" \
    --alias "${ALIAS}" \
    --network "${NETWORK}" 2>&1
)"
# La última línea con un C... es el id; filtramos los logs.
CONTRACT_ID="$(printf '%s\n' "${DEPLOY_OUT}" | grep -E '^C[A-Z0-9]{55}$' | tail -n1)"
[[ -n "${CONTRACT_ID}" ]] || die "deploy no devolvió contract id. output:
${DEPLOY_OUT}"
log "contrato desplegado: ${CONTRACT_ID}"

json_set addresses.terroir_contract "${CONTRACT_ID}"
json_set addresses.admin             "${ADMIN_PK}"
json_set wasm.hash                   "${WASM_HASH}"
json_set_obj wasm.size_bytes "$(jq -n --argjson v "$(stat -c %s "${WASM_PATH}")" '$v')"

# 4. init (si INIT_ARGS provisto). Decisión G: contrato necesita (admin, token) en init.
#    Si no, asumimos que la init ya la hizo T4 e2e o se hace en otro paso.
if [[ -n "${INIT_ARGS}" ]]; then
  log "invocando init ${INIT_ARGS}…"
  # shellcheck disable=SC2086
  stellar contract invoke \
    --id "${CONTRACT_ID}" \
    --source "${SOURCE_IDENTITY}" \
    --network "${NETWORK}" -- \
    init ${INIT_ARGS} >/dev/null
  log "init OK"
else
  warn "INIT_ARGS vacío: contrato desplegado pero NO inicializado. " \
       "Llamá 'init' desde tu script e2e o pasá INIT_ARGS='--admin G… --token C…'."
fi

# 5. resumen
cat <<EOF
=== terroir deploy OK (${NETWORK}) ===
  contract : ${CONTRACT_ID}
  admin    : ${ADMIN_PK}
  wasm     : ${WASM_PATH}
  sha256   : ${WASM_HASH}
  alias    : ${ALIAS}
EOF
