#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mapfile -t tracked_files < <(
  git ls-files ':(exclude)scripts/publish-check.sh' |
    while IFS= read -r path; do
      if [ -e "$path" ]; then
        printf '%s\n' "$path"
      fi
    done
)
mapfile -t history_revisions < <(git rev-list --all)
history_pathspecs=(':(exclude)scripts/publish-check.sh')

if [ "${#tracked_files[@]}" -eq 0 ]; then
  echo "No tracked files found to scan." >&2
  exit 1
fi

if [ "${#history_revisions[@]}" -eq 0 ]; then
  echo "No git history found to scan." >&2
  exit 1
fi

scan_tracked() {
  local label="$1"
  local pattern="$2"
  local failure_message="$3"
  local allow_pattern="${4:-}"

  echo "==> ${label}"

  local matches=""
  if matches="$(grep -nHE -- "$pattern" "${tracked_files[@]}")"; then
    if [ -n "$allow_pattern" ]; then
      local filtered_matches=""
      if filtered_matches="$(printf '%s\n' "$matches" | grep -nEv -- "$allow_pattern")"; then
        printf '%s\n' "$filtered_matches"
        echo "$failure_message" >&2
        exit 1
      fi
      local filtered_status=$?
      if [ "$filtered_status" -eq 1 ]; then
        return
      fi
      echo "Scan failed while checking tracked files for ${label}." >&2
      exit 1
    fi

    printf '%s\n' "$matches"
    echo "$failure_message" >&2
    exit 1
  else
    local status=$?
    if [ "$status" -ne 1 ]; then
      echo "Scan failed while checking tracked files for ${label}." >&2
      exit 1
    fi
  fi
}

scan_history() {
  local label="$1"
  local pattern="$2"
  local failure_message="$3"
  local allow_pattern="${4:-}"

  echo "==> ${label}"

  local matches=""
  if matches="$(git grep -nIE -- "$pattern" "${history_revisions[@]}" -- "${history_pathspecs[@]}")"; then
    if [ -n "$allow_pattern" ]; then
      local filtered_matches=""
      if filtered_matches="$(printf '%s\n' "$matches" | grep -nEv -- "$allow_pattern")"; then
        printf '%s\n' "$filtered_matches"
        echo "$failure_message" >&2
        exit 1
      fi
      local filtered_status=$?
      if [ "$filtered_status" -eq 1 ]; then
        return
      fi
      echo "Scan failed while checking git history for ${label}." >&2
      exit 1
    fi

    printf '%s\n' "$matches"
    echo "$failure_message" >&2
    exit 1
  else
    local status=$?
    if [ "$status" -ne 1 ]; then
      echo "Scan failed while checking git history for ${label}." >&2
      exit 1
    fi
  fi
}

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo check"
cargo check

echo "==> cargo clippy -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> cargo test -q"
cargo test -q

echo "==> bash -n project scripts"
bash -n \
  scripts/android-env.sh \
  scripts/android-gradle.sh \
  scripts/android-sync-assets.sh \
  scripts/build-web.sh \
  scripts/package-pages.sh \
  scripts/render-android-icons.sh \
  scripts/render-combat-icons.sh \
  scripts/render-pwa-icons.sh \
  scripts/run-gitleaks.sh \
  scripts/setup-android-sdk.sh

echo "==> node --check web/index.js"
node --check web/index.js

echo "==> node --check web/e2e-harness.js"
node --check web/e2e-harness.js

echo "==> node --check web/multiplayer.js"
node --check web/multiplayer.js

echo "==> node --check scripts/vendor-qr-libs.mjs"
node --check scripts/vendor-qr-libs.mjs

echo "==> npm ci"
npm ci --prefer-offline --no-audit --no-fund

echo "==> npm run vendor:qr"
npm run vendor:qr

echo "==> node --check web/qrcode.bundle.mjs"
node --check web/qrcode.bundle.mjs

echo "==> node --check web/jsqr.js"
node --check web/jsqr.js

echo "==> node --check web/ui-kit.js"
node --check web/ui-kit.js

echo "==> node --check web/sw.js"
node --check web/sw.js

echo "==> npm run vendor:qr -- --check"
npm run vendor:qr -- --check

echo "==> inspect worktree status"
git status --short --ignored

tracked_android_jars="$(
  git ls-files android |
    while IFS= read -r path; do
      case "$path" in
        *.jar)
          if [ -e "$path" ]; then
            printf '%s\n' "$path"
          fi
          ;;
      esac
    done
)"
if [ -n "$tracked_android_jars" ]; then
  printf '%s\n' "$tracked_android_jars"
  echo "Found tracked Android JAR files; Android debug dependencies must be bootstrapped on demand." >&2
  exit 1
fi

scan_tracked \
  "scan tracked files for machine-specific paths" \
  "(/home/|/Users/|C:\\\\)" \
  "Found machine-specific paths in tracked content."

scan_tracked \
  "scan tracked files for obvious secrets" \
  "(AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-|AIza[0-9A-Za-z\\-_]{35}|sk-[A-Za-z0-9]{20,}|-----BEGIN (RSA|EC|OPENSSH|DSA|PRIVATE KEY)-----|aws_secret_access_key|aws_access_key_id)" \
  "Found secret-like material in tracked content."

scan_tracked \
  "scan tracked files for email addresses" \
  "[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}" \
  "Found email-like content in tracked files." \
  "@(example\\.com|example\\.org|example\\.net|localhost)\\b"

scan_history \
  "scan reachable git history for machine-specific paths" \
  "(/home/|/Users/|C:\\\\)" \
  "Found machine-specific paths in reachable git history."

scan_history \
  "scan reachable git history for obvious secrets" \
  "(AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-|AIza[0-9A-Za-z\\-_]{35}|sk-[A-Za-z0-9]{20,}|-----BEGIN (RSA|EC|OPENSSH|DSA|PRIVATE KEY)-----|aws_secret_access_key|aws_access_key_id)" \
  "Found secret-like material in reachable git history."

scan_history \
  "scan reachable git history for email addresses" \
  "[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}" \
  "Found email-like content in reachable git history." \
  "@(example\\.com|example\\.org|example\\.net|localhost)\\b"

echo "==> build web bundle"
./scripts/build-web.sh

echo "==> verify ignored generated artifacts"
expected_ignored=(
  "target/"
  "web/mazocarta.wasm"
  "web/.debug-mode.json"
  "web/apple-touch-icon.png"
  "web/jsqr.js"
  "web/qrcode.bundle.mjs"
  "web/icons/icon-any-192.png"
  "web/icons/icon-any-512.png"
  "web/icons/icon-maskable-192.png"
  "web/icons/icon-maskable-512.png"
  "web/icons/combat/heart.png"
  "web/icons/combat/shield.png"
  "web/icons/combat/energy.png"
  "web/icons/combat/deck.png"
  "web/icons/combat/arrow.png"
  "android/app/src/main/assets/site/"
  "android/app/src/main/res/drawable/ic_launcher.png"
  "android/app/src/main/res/drawable/ic_launcher_foreground.xml"
  "android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml"
  "android/app/src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml"
  ".gradle-bootstrap/"
)

for path in "${expected_ignored[@]}"; do
  if ! git check-ignore -q "$path"; then
    echo "Expected ignored path is not ignored: $path" >&2
    exit 1
  fi
done

echo "publish-check passed"
