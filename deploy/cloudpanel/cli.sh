#!/usr/bin/env bash
set -euo pipefail
APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
set -a
[[ -f "$APP_DIR/.env" ]] && source "$APP_DIR/.env"
set +a
BIN_PATH="${ARGWS_PONTO_MANAGER_BINARY:-$APP_DIR/bin/argws_ponto_manager}"
if [[ "$BIN_PATH" != /* ]]; then
  BIN_PATH="$APP_DIR/${BIN_PATH#./}"
fi
export ARGWS_PONTO_MANAGER_ENV_FILE="${ARGWS_PONTO_MANAGER_ENV_FILE:-$APP_DIR/.env}"
export ARGWS_PONTO_MANAGER_WEB_DIST_DIR="${ARGWS_PONTO_MANAGER_WEB_DIST_DIR:-$APP_DIR/dist}"
export ARGWS_PONTO_MANAGER_DATA_DIR="${ARGWS_PONTO_MANAGER_DATA_DIR:-$APP_DIR/data}"
"$BIN_PATH" --mode=cli --data-dir "$ARGWS_PONTO_MANAGER_DATA_DIR" "$@"
