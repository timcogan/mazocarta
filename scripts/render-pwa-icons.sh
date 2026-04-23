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
PWA_ICON_INSET="30.72"
PWA_ICON_SIZE="194.56"
TEMP_DIR="$(mktemp -d)"
PWA_ICON_SVG="$TEMP_DIR/mazocarta-pwa.svg"
declare -a INKSCAPE_COLOR_MODE_ARGS=()

cleanup() {
  rm -rf "$TEMP_DIR"
}

trap cleanup EXIT INT TERM

if ! command -v inkscape >/dev/null 2>&1; then
  echo "Inkscape is required to build opaque PWA icons." >&2
  exit 1
fi

inkscape_version="$(inkscape --version 2>/dev/null | sed -nE 's/.* ([0-9]+)\.([0-9]+)(\.[0-9]+)?.*/\1 \2/p' | head -n 1)"
if [[ -n "$inkscape_version" ]]; then
  read -r inkscape_major inkscape_minor <<<"$inkscape_version"
  if (( inkscape_major > 1 || (inkscape_major == 1 && inkscape_minor >= 1) )); then
    INKSCAPE_COLOR_MODE_ARGS+=(--export-png-color-mode=RGB_8)
  fi
fi

export_png_icon() {
  local size="$1"
  local output="$2"
  local stderr_file
  stderr_file="$(mktemp)"

  if ! inkscape "$PWA_ICON_SVG" \
    --export-type=png \
    --export-filename="$output" \
    --export-width="$size" \
    --export-height="$size" \
    --export-background="$ICON_BACKGROUND" \
    --export-background-opacity=1 \
    "${INKSCAPE_COLOR_MODE_ARGS[@]}" \
    >/dev/null 2>"$stderr_file"; then
    cat "$stderr_file" >&2
    rm -f "$stderr_file"
    return 1
  fi

  # Some headless Inkscape builds emit a benign GtkRecentManager warning on stderr
  # even when export succeeds. Keep all other stderr so real export problems stay visible.
  sed '/GtkRecentManager/d' "$stderr_file" >&2
  rm -f "$stderr_file"
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
