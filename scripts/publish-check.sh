#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mapfile -t tracked_files < <(git ls-files ':(exclude)scripts/publish-check.sh')

if [ "${#tracked_files[@]}" -eq 0 ]; then
  echo "No tracked files found to scan." >&2
  exit 1
fi

scan_tracked() {
  local label="$1"
  local pattern="$2"
  local failure_message="$3"

  echo "==> ${label}"

  local status=0
  rg -n --hidden -S "$pattern" -- "${tracked_files[@]}" || status=$?

  if [ "$status" -eq 0 ]; then
    echo "$failure_message" >&2
    exit 1
  fi

  if [ "$status" -ne 1 ]; then
    echo "Scan failed while checking tracked files for ${label}." >&2
    exit 1
  fi
}

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo check"
cargo check

echo "==> cargo test -q"
cargo test -q

echo "==> bash -n scripts/build-web.sh scripts/package-pages.sh"
bash -n scripts/build-web.sh scripts/package-pages.sh

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
  "Found email-like content in tracked files."

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
