#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 COMBAT_ICON_DIR" >&2
  exit 1
fi

ICON_DIR="$1"
RESVG_BIN="${RESVG_BIN:-resvg}"

if ! command -v "$RESVG_BIN" >/dev/null 2>&1; then
  echo "resvg is required to build combat icon PNGs." >&2
  exit 1
fi

shopt -s nullglob
svg_files=("$ICON_DIR"/*.svg)
shopt -u nullglob

if [[ ${#svg_files[@]} -eq 0 ]]; then
  echo "No combat SVG icons found in $ICON_DIR" >&2
  exit 1
fi

for svg_path in "${svg_files[@]}"; do
  png_path="${svg_path%.svg}.png"
  "$RESVG_BIN" --zoom 4 "$svg_path" "$png_path"
done
