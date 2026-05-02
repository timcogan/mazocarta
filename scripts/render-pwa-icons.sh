#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 SVG_ICON ICON_DIR APPLE_ICON" >&2
  exit 1
fi

SVG_ICON="$1"
ICON_DIR="$2"
APPLE_ICON="$3"
MASKABLE_ICON_BACKGROUND="#000000"
MASKABLE_ICON_INSET="60"
MASKABLE_ICON_SIZE="136"
ANY_ICON_INSET="28"
ANY_ICON_SIZE="200"
RESVG_BIN="${RESVG_BIN:-resvg}"
TEMP_DIR="$(mktemp -d)"
MASKABLE_ICON_SVG="$TEMP_DIR/mazocarta-pwa-maskable.svg"
ANY_ICON_SVG="$TEMP_DIR/mazocarta-pwa-any.svg"

cleanup() {
  rm -rf "$TEMP_DIR"
}

trap cleanup EXIT INT TERM

if ! command -v "$RESVG_BIN" >/dev/null 2>&1; then
  echo "resvg is required to build PWA icons." >&2
  exit 1
fi

export_png_icon() {
  local size="$1"
  local source="$2"
  local output="$3"

  "$RESVG_BIN" \
    --width "$size" \
    --height "$size" \
    "$source" \
    "$output"
}

export_opaque_png_icon() {
  local size="$1"
  local output="$2"

  "$RESVG_BIN" \
    --background "$MASKABLE_ICON_BACKGROUND" \
    --width "$size" \
    --height "$size" \
    "$MASKABLE_ICON_SVG" \
    "$output"
}

build_pwa_icon_svg() {
  local output="$1"
  local inset="$2"
  local icon_size="$3"
  local background="${4:-}"

  {
    printf '%s\n' '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">'
    if [[ -n "$background" ]]; then
      printf '  <rect width="256" height="256" fill="%s" />\n' "$background"
    fi
    perl -0pe \
      "s/<svg\\b/<svg x=\\\"$inset\\\" y=\\\"$inset\\\" width=\\\"$icon_size\\\" height=\\\"$icon_size\\\" preserveAspectRatio=\\\"xMidYMid meet\\\"/" \
      "$SVG_ICON"
    printf '%s\n' '</svg>'
  } >"$output"
}

mkdir -p "$ICON_DIR"
build_pwa_icon_svg "$MASKABLE_ICON_SVG" "$MASKABLE_ICON_INSET" "$MASKABLE_ICON_SIZE" "$MASKABLE_ICON_BACKGROUND"
build_pwa_icon_svg "$ANY_ICON_SVG" "$ANY_ICON_INSET" "$ANY_ICON_SIZE"
rm -f "$ICON_DIR/icon-192.png" "$ICON_DIR/icon-512.png"
export_opaque_png_icon 192 "$ICON_DIR/icon-maskable-192.png"
export_opaque_png_icon 512 "$ICON_DIR/icon-maskable-512.png"
export_png_icon 192 "$ANY_ICON_SVG" "$ICON_DIR/icon-any-192.png"
export_png_icon 512 "$ANY_ICON_SVG" "$ICON_DIR/icon-any-512.png"
export_opaque_png_icon 180 "$APPLE_ICON"
