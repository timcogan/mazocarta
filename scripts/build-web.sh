#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_TARGET="$ROOT_DIR/target/wasm32-unknown-unknown/release/mazocarta.wasm"
WEB_WASM="$ROOT_DIR/web/mazocarta.wasm"
ICON_DIR="$ROOT_DIR/web/icons"
SVG_ICON="$ROOT_DIR/web/mazocarta.svg"
APPLE_ICON="$ROOT_DIR/web/apple-touch-icon.png"
APP_CHANNEL="${MAZOCARTA_APP_CHANNEL:-preview}"
BUILD_TIMESTAMP_UTC="${MAZOCARTA_APP_BUILD_TIMESTAMP_UTC:-$(date -u +"%Y-%m-%dT%H:%MZ")}"
GIT_SHA_SHORT="${MAZOCARTA_APP_GIT_SHA_SHORT:-}"
CARGO_FEATURES="${MAZOCARTA_CARGO_FEATURES:-}"

ensure_node_deps() {
  local node_lock="$ROOT_DIR/node_modules/.package-lock.json"
  if [[ -d "$ROOT_DIR/node_modules/qrcode" &&
    -d "$ROOT_DIR/node_modules/jsqr" &&
    -d "$ROOT_DIR/node_modules/esbuild" &&
    -f "$node_lock" &&
    ! "$ROOT_DIR/package-lock.json" -nt "$node_lock" ]]; then
    return
  fi

  npm --prefix "$ROOT_DIR" ci --prefer-offline --no-audit --no-fund
}

if [[ -z "$GIT_SHA_SHORT" ]] && git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  GIT_SHA_SHORT="$(git -C "$ROOT_DIR" rev-parse --short=7 HEAD 2>/dev/null || true)"
fi

export MAZOCARTA_APP_CHANNEL="$APP_CHANNEL"
export MAZOCARTA_APP_BUILD_TIMESTAMP_UTC="$BUILD_TIMESTAMP_UTC"
if [[ -n "$GIT_SHA_SHORT" ]]; then
  export MAZOCARTA_APP_GIT_SHA_SHORT="$GIT_SHA_SHORT"
else
  unset MAZOCARTA_APP_GIT_SHA_SHORT || true
fi

ensure_node_deps
npm --prefix "$ROOT_DIR" run vendor:qr
cargo_args=(
  build
  --lib
  --release
  --target wasm32-unknown-unknown
  --manifest-path "$ROOT_DIR/Cargo.toml"
)
if [[ -n "$CARGO_FEATURES" ]]; then
  cargo_args+=(--features "$CARGO_FEATURES")
fi
cargo "${cargo_args[@]}"
cp "$WASM_TARGET" "$WEB_WASM"
"$ROOT_DIR/scripts/render-pwa-icons.sh" "$SVG_ICON" "$ICON_DIR" "$APPLE_ICON"
"$ROOT_DIR/scripts/render-combat-icons.sh" "$ICON_DIR/combat"
printf 'Copied %s -> %s\n' "$WASM_TARGET" "$WEB_WASM"
