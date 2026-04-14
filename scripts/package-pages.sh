#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_ARG="${1:-}"

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

build_site() {
  local ref="$1"
  local channel="$2"
  local destination="$3"
  local worktree
  local short_sha

  worktree="$(create_worktree "$ref")"
  short_sha="$(git -C "$worktree" rev-parse --short=7 HEAD)"

  echo "==> building ${channel} site from ${ref}"
  (
    cd "$worktree"
    MAZOCARTA_APP_CHANNEL="$channel" \
      MAZOCARTA_APP_BUILD_TIMESTAMP_UTC="$BUILD_TIMESTAMP_UTC" \
      MAZOCARTA_APP_GIT_SHA_SHORT="$short_sha" \
      ./scripts/build-web.sh
  )

  mkdir -p "$destination"
  cp -R "$worktree/web/." "$destination/"
  rm -f "$destination/.debug-mode.json"
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
