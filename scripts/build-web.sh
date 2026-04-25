#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_TARGET="$ROOT_DIR/target/wasm32-unknown-unknown/release/mazocarta.wasm"
WEB_WASM="$ROOT_DIR/web/mazocarta.wasm"
ICON_DIR="$ROOT_DIR/web/icons"
SVG_ICON="$ROOT_DIR/web/mazocarta.svg"
APPLE_ICON="$ROOT_DIR/web/apple-touch-icon.png"

cargo build --lib --release --target wasm32-unknown-unknown --manifest-path "$ROOT_DIR/Cargo.toml"
cp "$WASM_TARGET" "$WEB_WASM"
"$ROOT_DIR/scripts/render-pwa-icons.sh" "$SVG_ICON" "$ICON_DIR" "$APPLE_ICON"
"$ROOT_DIR/scripts/render-combat-icons.sh" "$ROOT_DIR/web/icons/combat"
printf 'Copied %s -> %s\n' "$WASM_TARGET" "$WEB_WASM"
