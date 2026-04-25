#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 SVG_ICON ICON_DIR APPLE_ICON" >&2
  exit 1
fi

SVG_ICON="$1"
ICON_DIR="$2"
APPLE_ICON="$3"
ICON_BACKGROUND="#000000"
PWA_ICON_INSET="36"
PWA_ICON_SIZE="184"
RESVG_BIN="${RESVG_BIN:-resvg}"
TEMP_DIR="$(mktemp -d)"
PWA_ICON_SVG="$TEMP_DIR/mazocarta-pwa.svg"

cleanup() {
  rm -rf "$TEMP_DIR"
}

trap cleanup EXIT INT TERM

if ! command -v "$RESVG_BIN" >/dev/null 2>&1; then
  echo "resvg is required to build opaque PWA icons." >&2
  exit 1
fi

export_png_icon() {
  local size="$1"
  local output="$2"

  "$RESVG_BIN" \
    --background "$ICON_BACKGROUND" \
    --width "$size" \
    --height "$size" \
    "$PWA_ICON_SVG" \
    "$output"
}

build_pwa_icon_svg() {
  {
    printf '%s\n' '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">'
    printf '  <rect width="256" height="256" fill="%s" />\n' "$ICON_BACKGROUND"
    perl -0pe \
      "s/<svg\\b/<svg x=\\\"$PWA_ICON_INSET\\\" y=\\\"$PWA_ICON_INSET\\\" width=\\\"$PWA_ICON_SIZE\\\" height=\\\"$PWA_ICON_SIZE\\\" preserveAspectRatio=\\\"xMidYMid meet\\\"/" \
      "$SVG_ICON"
    printf '%s\n' '</svg>'
  } >"$PWA_ICON_SVG"
}

mkdir -p "$ICON_DIR"
build_pwa_icon_svg
export_png_icon 192 "$ICON_DIR/icon-192.png"
export_png_icon 512 "$ICON_DIR/icon-512.png"
export_png_icon 180 "$APPLE_ICON"
