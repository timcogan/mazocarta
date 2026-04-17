#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mapfile -t tracked_files < <(git ls-files ':(exclude)scripts/publish-check.sh')
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

echo "==> bash -n scripts/build-web.sh scripts/package-pages.sh scripts/run-gitleaks.sh"
bash -n scripts/build-web.sh scripts/package-pages.sh scripts/run-gitleaks.sh

echo "==> node --check web/index.js"
node --check web/index.js

echo "==> node --check web/sw.js"
node --check web/sw.js

echo "==> inspect worktree status"
git status --short --ignored

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
  "web/icons/icon-192.png"
  "web/icons/icon-512.png"
)

for path in "${expected_ignored[@]}"; do
  if ! git check-ignore -q "$path"; then
    echo "Expected ignored path is not ignored: $path" >&2
    exit 1
  fi
done

echo "publish-check passed"
