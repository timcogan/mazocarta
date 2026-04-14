#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_TARGET="$ROOT_DIR/target/wasm32-unknown-unknown/release/mazocarta.wasm"
WEB_WASM="$ROOT_DIR/web/mazocarta.wasm"
ICON_DIR="$ROOT_DIR/web/icons"
SVG_ICON="$ROOT_DIR/web/mazocarta.svg"
APPLE_ICON="$ROOT_DIR/web/apple-touch-icon.png"

export_png_icon() {
  local size="$1"
  local output="$2"
  inkscape "$SVG_ICON" \
    --export-type=png \
    --export-filename="$output" \
    --export-width="$size" \
    --export-height="$size" \
    --export-background-opacity=0 \
    --export-png-color-mode=RGBA_8 \
    >/dev/null
}

cargo build --release --target wasm32-unknown-unknown --manifest-path "$ROOT_DIR/Cargo.toml"
cp "$WASM_TARGET" "$WEB_WASM"
mkdir -p "$ICON_DIR"
if ! command -v inkscape >/dev/null 2>&1; then
  echo "Inkscape is required to build PWA icons with transparency." >&2
  exit 1
fi
export_png_icon 192 "$ICON_DIR/icon-192.png"
export_png_icon 512 "$ICON_DIR/icon-512.png"
export_png_icon 180 "$APPLE_ICON"
printf 'Copied %s -> %s\n' "$WASM_TARGET" "$WEB_WASM"
