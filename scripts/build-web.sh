#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_TARGET="$ROOT_DIR/target/wasm32-unknown-unknown/release/mazocarta.wasm"
WEB_WASM="$ROOT_DIR/web/mazocarta.wasm"
ICON_DIR="$ROOT_DIR/web/icons"
SVG_ICON="$ROOT_DIR/web/mazocarta.svg"
APPLE_ICON="$ROOT_DIR/web/apple-touch-icon.png"
declare -a INKSCAPE_COLOR_MODE_ARGS=()

if ! command -v inkscape >/dev/null 2>&1; then
  echo "Inkscape is required to build PWA icons with transparency." >&2
  exit 1
fi

inkscape_version="$(inkscape --version 2>/dev/null | sed -nE 's/.* ([0-9]+)\.([0-9]+)(\.[0-9]+)?.*/\1 \2/p' | head -n 1)"
if [[ -n "$inkscape_version" ]]; then
  read -r inkscape_major inkscape_minor <<<"$inkscape_version"
  if (( inkscape_major > 1 || (inkscape_major == 1 && inkscape_minor >= 1) )); then
    INKSCAPE_COLOR_MODE_ARGS+=(--export-png-color-mode=RGBA_8)
  fi
fi

export_png_icon() {
  local size="$1"
  local output="$2"
  local stderr_file cleanup_cmd
  stderr_file="$(mktemp)"
  printf -v cleanup_cmd 'rm -f %q' "$stderr_file"
  trap "$cleanup_cmd" RETURN EXIT INT TERM

  if ! inkscape "$SVG_ICON" \
    --export-type=png \
    --export-filename="$output" \
    --export-width="$size" \
    --export-height="$size" \
    --export-background-opacity=0 \
    "${INKSCAPE_COLOR_MODE_ARGS[@]}" \
    >/dev/null 2>"$stderr_file"; then
    cat "$stderr_file" >&2
    rm -f "$stderr_file"
    trap - RETURN EXIT INT TERM
    return 1
  fi

  # Some headless Inkscape builds emit a benign GtkRecentManager warning on stderr
  # even when export succeeds. Keep all other stderr so real export problems stay visible.
  sed '/GtkRecentManager/d' "$stderr_file" >&2
  rm -f "$stderr_file"
  trap - RETURN EXIT INT TERM
}

cargo build --release --target wasm32-unknown-unknown --manifest-path "$ROOT_DIR/Cargo.toml"
cp "$WASM_TARGET" "$WEB_WASM"
mkdir -p "$ICON_DIR"
export_png_icon 192 "$ICON_DIR/icon-192.png"
export_png_icon 512 "$ICON_DIR/icon-512.png"
export_png_icon 180 "$APPLE_ICON"
printf 'Copied %s -> %s\n' "$WASM_TARGET" "$WEB_WASM"
