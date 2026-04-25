#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_ARG="${1:-}"
STABLE_ACCENT_COLOR="#3f6"
STABLE_ACCENT_COLOR_ALT="#33ff66"
PREVIEW_ACCENT_COLOR="#3df5ff"
PREVIEW_SHELL_THEME_COLOR="#000000"
PREVIEW_GAME_TITLE="Mazocarta Preview"
PREVIEW_SHORT_NAME="Mazo Preview"
RESVG_BIN="${RESVG_BIN:-resvg}"

if [[ -z "$OUT_ARG" ]]; then
  echo "usage: $0 OUT_DIR" >&2
  exit 1
fi

if [[ "$OUT_ARG" == "/" || "$OUT_ARG" == "." ]]; then
  echo "Refusing to package Pages artifact into '$OUT_ARG'." >&2
  exit 1
fi

if [[ "$OUT_ARG" = /* ]]; then
  OUT_DIR="$OUT_ARG"
else
  OUT_DIR="$PWD/$OUT_ARG"
fi

PREVIEW_PATH="${MAZOCARTA_PAGES_PREVIEW_PATH:-preview}"
PREVIEW_REF="${MAZOCARTA_PAGES_PREVIEW_REF:-HEAD}"
STABLE_REF="${MAZOCARTA_PAGES_STABLE_REF:-}"
BUILD_TIMESTAMP_UTC="${MAZOCARTA_PAGES_BUILD_TIMESTAMP_UTC:-$(date -u +%Y%m%d%H%M)}"

TEMP_ROOT="$(mktemp -d)"
declare -a WORKTREES=()

cleanup() {
  local status=$?
  local worktree
  for worktree in "${WORKTREES[@]:-}"; do
    git -C "$ROOT_DIR" worktree remove --force "$worktree" >/dev/null 2>&1 || true
    rm -rf "$worktree"
  done
  rm -rf "$TEMP_ROOT"
  exit "$status"
}

trap cleanup EXIT

resolve_latest_stable_tag() {
  local tag
  while IFS= read -r tag; do
    if [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
      printf '%s\n' "$tag"
      return 0
    fi
  done < <(git -C "$ROOT_DIR" tag --list 'v*' --sort=-version:refname)
  return 1
}

create_worktree() {
  local ref="$1"
  local worktree="$TEMP_ROOT/worktree-${#WORKTREES[@]}"
  git -C "$ROOT_DIR" worktree add --detach "$worktree" "$ref" >/dev/null
  WORKTREES+=("$worktree")
  printf '%s\n' "$worktree"
}

stamp_service_worker_version() {
  local destination="$1"
  local channel="$2"
  local short_sha="$3"
  local sw_path="$destination/sw.js"
  local sw_version="${channel}-${BUILD_TIMESTAMP_UTC}-${short_sha}"
  local expected_line="const CACHE_VERSION = \"${sw_version}\";"

  if [[ ! -f "$sw_path" ]]; then
    echo "Missing service worker to stamp: $sw_path" >&2
    return 1
  fi

  sed -i -E "s/^const CACHE_VERSION = \".*\";/const CACHE_VERSION = \"${sw_version}\";/" "$sw_path"

  if ! grep -qxF "$expected_line" "$sw_path"; then
    echo "Failed to stamp service worker version in $sw_path" >&2
    return 1
  fi

  if grep -q "__MAZOCARTA_SW_VERSION__" "$sw_path"; then
    echo "Service worker placeholder remained after stamping: $sw_path" >&2
    return 1
  fi
}

rewrite_manifest_branding() {
  local manifest_path="$1"
  local channel="$2"

  node --input-type=module -e '
    import fs from "node:fs";

    const [manifestPath, channel, previewTitle, previewShortName, previewThemeColor] = process.argv.slice(1);
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    manifest.id = "./";
    if (channel === "preview") {
      manifest.name = previewTitle;
      manifest.short_name = previewShortName;
      manifest.theme_color = previewThemeColor;
    }
    fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  ' "$manifest_path" "$channel" "$PREVIEW_GAME_TITLE" "$PREVIEW_SHORT_NAME" "$PREVIEW_SHELL_THEME_COLOR"
}

escape_grep_ere_literal() {
  printf '%s\n' "$1" | sed -e 's/[][\\.^$*+?(){}|]/\\&/g'
}

apply_preview_branding() {
  local destination="$1"
  local index_path="$destination/index.html"
  local manifest_path="$destination/manifest.webmanifest"
  local svg_path="$destination/mazocarta.svg"
  local icon_dir="$destination/icons"
  local apple_icon_path="$destination/apple-touch-icon.png"

  perl -0pi -e "s/${STABLE_ACCENT_COLOR}\\b/${PREVIEW_ACCENT_COLOR}/g; s/${STABLE_ACCENT_COLOR_ALT}\\b/${PREVIEW_ACCENT_COLOR}/g;" "$svg_path"
  perl -0pi -e 's/content="stable"/content="preview"/; s/content="Mazocarta"/content="Mazocarta Preview"/g; s#<title>Mazocarta</title>#<title>Mazocarta Preview</title>#;' "$index_path"
  rewrite_manifest_branding "$manifest_path" "preview"
  "$ROOT_DIR/scripts/render-pwa-icons.sh" "$svg_path" "$icon_dir" "$apple_icon_path"
}

verify_site_branding() {
  local destination="$1"
  local channel="$2"
  local stable_accent_pattern

  if [[ "$channel" == "preview" ]]; then
    stable_accent_pattern="$(escape_grep_ere_literal "$STABLE_ACCENT_COLOR")|$(escape_grep_ere_literal "$STABLE_ACCENT_COLOR_ALT")"
    grep -q '<title>Mazocarta Preview</title>' "$destination/index.html"
    grep -q "<meta name=\"theme-color\" content=\"$PREVIEW_SHELL_THEME_COLOR\"" "$destination/index.html"
    grep -q '"name": "Mazocarta Preview"' "$destination/manifest.webmanifest"
    grep -q '"short_name": "Mazo Preview"' "$destination/manifest.webmanifest"
    grep -q "\"theme_color\": \"$PREVIEW_SHELL_THEME_COLOR\"" "$destination/manifest.webmanifest"
    grep -Fq -- "$PREVIEW_ACCENT_COLOR" "$destination/mazocarta.svg"
    if grep -Eq "${stable_accent_pattern}\\b" "$destination/mazocarta.svg"; then
      echo "Preview SVG still contains the stable green accent." >&2
      return 1
    fi
    return 0
  fi

  grep -q '<title>Mazocarta</title>' "$destination/index.html"
  grep -q '"name": "Mazocarta"' "$destination/manifest.webmanifest"
}

build_legacy_site_with_resvg() {
  local worktree="$1"
  local channel="$2"
  local short_sha="$3"
  local wasm_target="$worktree/target/wasm32-unknown-unknown/release/mazocarta.wasm"
  local web_wasm="$worktree/web/mazocarta.wasm"
  local icon_dir="$worktree/web/icons"
  local svg_icon="$worktree/web/mazocarta.svg"
  local apple_icon="$worktree/web/apple-touch-icon.png"

  if ! command -v "$RESVG_BIN" >/dev/null 2>&1; then
    echo "resvg is required to build legacy Pages icons." >&2
    exit 1
  fi

  # Legacy release tags hardcode Inkscape in scripts/build-web.sh. Rebuild their
  # icons with the shared PWA icon compositor so Pages packaging stays compatible
  # without that dependency and keeps the modern opaque background/inset treatment.
  (
    cd "$worktree"
    MAZOCARTA_APP_CHANNEL="$channel" \
      MAZOCARTA_APP_BUILD_TIMESTAMP_UTC="$BUILD_TIMESTAMP_UTC" \
      MAZOCARTA_APP_GIT_SHA_SHORT="$short_sha" \
      cargo build --lib --release --target wasm32-unknown-unknown --manifest-path "$worktree/Cargo.toml"
  )

  cp "$wasm_target" "$web_wasm"
  mkdir -p "$icon_dir"
  RESVG_BIN="$RESVG_BIN" \
    "$ROOT_DIR/scripts/render-pwa-icons.sh" "$svg_icon" "$icon_dir" "$apple_icon"
  printf 'Copied %s -> %s\n' "$wasm_target" "$web_wasm"
}

build_worktree_site() {
  local worktree="$1"
  local channel="$2"
  local short_sha="$3"

  if [[ -f "$worktree/scripts/render-pwa-icons.sh" ]]; then
    install -m 0755 "$ROOT_DIR/scripts/render-pwa-icons.sh" "$worktree/scripts/render-pwa-icons.sh"
    install -m 0755 "$ROOT_DIR/scripts/render-combat-icons.sh" "$worktree/scripts/render-combat-icons.sh"
    (
      cd "$worktree"
      MAZOCARTA_APP_CHANNEL="$channel" \
        MAZOCARTA_APP_BUILD_TIMESTAMP_UTC="$BUILD_TIMESTAMP_UTC" \
        MAZOCARTA_APP_GIT_SHA_SHORT="$short_sha" \
        ./scripts/build-web.sh
    )
    return 0
  fi

  build_legacy_site_with_resvg "$worktree" "$channel" "$short_sha"
}

build_site() {
  local ref="$1"
  local channel="$2"
  local destination="$3"
  local worktree
  local short_sha

  worktree="$(create_worktree "$ref")"
  short_sha="$(git -C "$worktree" rev-parse --short=7 HEAD)"

  echo "==> building ${channel} site from ${ref}"
  build_worktree_site "$worktree" "$channel" "$short_sha"

  mkdir -p "$destination"
  cp -R "$worktree/web/." "$destination/"
  rm -f "$destination/.debug-mode.json"
  rewrite_manifest_branding "$destination/manifest.webmanifest" "$channel"
  if [[ "$channel" == "preview" ]]; then
    apply_preview_branding "$destination"
  fi
  stamp_service_worker_version "$destination" "$channel" "$short_sha"
  verify_site_branding "$destination" "$channel"
}

build_current_site() {
  local channel="$1"
  local destination="$2"
  local short_sha

  short_sha="$(git -C "$ROOT_DIR" rev-parse --short=7 HEAD)"

  echo "==> building ${channel} site from current checkout"
  (
    cd "$ROOT_DIR"
    MAZOCARTA_APP_CHANNEL="$channel" \
      MAZOCARTA_APP_BUILD_TIMESTAMP_UTC="$BUILD_TIMESTAMP_UTC" \
      MAZOCARTA_APP_GIT_SHA_SHORT="$short_sha" \
      ./scripts/build-web.sh
  )

  mkdir -p "$destination"
  cp -R "$ROOT_DIR/web/." "$destination/"
  rm -f "$destination/.debug-mode.json"
  rewrite_manifest_branding "$destination/manifest.webmanifest" "$channel"
  if [[ "$channel" == "preview" ]]; then
    apply_preview_branding "$destination"
  fi
  stamp_service_worker_version "$destination" "$channel" "$short_sha"
  verify_site_branding "$destination" "$channel"
}

write_root_redirect() {
  local preview_path="$1"

  cat >"$OUT_DIR/index.html" <<EOF
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="refresh" content="0; url=./${preview_path}/" />
    <title>Mazocarta Preview</title>
    <link rel="canonical" href="./${preview_path}/" />
  </head>
  <body>
    <p>Redirecting to <a href="./${preview_path}/">${preview_path}</a>...</p>
  </body>
</html>
EOF
}

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
touch "$OUT_DIR/.nojekyll"

if [[ "$PREVIEW_REF" == "HEAD" ]]; then
  build_current_site "preview" "$OUT_DIR/$PREVIEW_PATH"
else
  build_site "$PREVIEW_REF" "preview" "$OUT_DIR/$PREVIEW_PATH"
fi

if [[ -n "$STABLE_REF" ]]; then
  build_site "$STABLE_REF" "stable" "$OUT_DIR"
elif stable_tag="$(resolve_latest_stable_tag)"; then
  build_site "$stable_tag" "stable" "$OUT_DIR"
else
  echo "==> no stable tag found; writing root redirect to /${PREVIEW_PATH}/"
  write_root_redirect "$PREVIEW_PATH"
fi

echo "Pages artifact ready at $OUT_DIR"
