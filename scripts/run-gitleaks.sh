#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GITLEAKS_VERSION="${GITLEAKS_VERSION:-8.30.0}"
GITLEAKS_REPO="https://github.com/gitleaks/gitleaks/releases/download/v${GITLEAKS_VERSION}"
CACHE_ROOT="${XDG_CACHE_HOME:-$HOME/.cache}/mazocarta-tools/gitleaks"

detect_archive_name() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64) printf 'gitleaks_%s_linux_x64.tar.gz\n' "$GITLEAKS_VERSION" ;;
        aarch64|arm64) printf 'gitleaks_%s_linux_arm64.tar.gz\n' "$GITLEAKS_VERSION" ;;
        *)
          echo "Unsupported Linux architecture for gitleaks: $arch" >&2
          return 1
          ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64) printf 'gitleaks_%s_darwin_x64.tar.gz\n' "$GITLEAKS_VERSION" ;;
        arm64) printf 'gitleaks_%s_darwin_arm64.tar.gz\n' "$GITLEAKS_VERSION" ;;
        *)
          echo "Unsupported macOS architecture for gitleaks: $arch" >&2
          return 1
          ;;
      esac
      ;;
    *)
      echo "Unsupported OS for gitleaks: $os" >&2
      return 1
      ;;
  esac
}

checksum_cmd() {
  if command -v sha256sum >/dev/null 2>&1; then
    printf 'sha256sum\n'
  elif command -v shasum >/dev/null 2>&1; then
    printf 'shasum -a 256\n'
  else
    echo "Missing checksum tool: need sha256sum or shasum" >&2
    return 1
  fi
}

ensure_gitleaks() {
  local archive_name install_dir bin_path tmp_dir checksum_tool
  archive_name="$(detect_archive_name)"
  install_dir="$CACHE_ROOT/$GITLEAKS_VERSION"
  bin_path="$install_dir/gitleaks"

  if [[ -x "$bin_path" ]]; then
    printf '%s\n' "$bin_path"
    return 0
  fi

  mkdir -p "$install_dir"
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' RETURN

  curl -fsSL -o "$tmp_dir/$archive_name" "$GITLEAKS_REPO/$archive_name"
  curl -fsSL -o "$tmp_dir/checksums.txt" "$GITLEAKS_REPO/gitleaks_${GITLEAKS_VERSION}_checksums.txt"

  grep " $archive_name\$" "$tmp_dir/checksums.txt" > "$tmp_dir/checksum-entry.txt"
  checksum_tool="$(checksum_cmd)"
  (
    cd "$tmp_dir"
    $checksum_tool -c checksum-entry.txt >&2
  )

  tar -xzf "$tmp_dir/$archive_name" -C "$tmp_dir"
  install -m 0755 "$tmp_dir/gitleaks" "$bin_path"

  printf '%s\n' "$bin_path"
}

main() {
  local gitleaks_bin
  gitleaks_bin="$(ensure_gitleaks)"

  echo "==> gitleaks git"
  "$gitleaks_bin" git --no-banner --redact "$ROOT_DIR"
}

main "$@"
