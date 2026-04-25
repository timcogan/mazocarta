#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?usage: install-resvg.sh VERSION}"
tmp_dir="$(mktemp -d)"
archive_path=""

cleanup() {
  rm -rf "$tmp_dir"
}

trap cleanup EXIT

detect_arch_suffix() {
  case "$(uname -m)" in
    x86_64 | amd64)
      printf '%s\n' "linux-x86_64"
      ;;
    aarch64 | arm64)
      printf '%s\n' "linux-aarch64"
      ;;
    *)
      echo "Unsupported resvg runner architecture: $(uname -m)" >&2
      return 1
      ;;
  esac
}

resvg_sha256_for() {
  case "${1}:${2}" in
    "v0.47.0:linux-x86_64")
      printf '%s\n' "5c84dcbcd032fe7e8d96e616fd6807a2f9df6561d2e6582b37e91e63c6cb4fe7"
      ;;
    *)
      return 1
      ;;
  esac
}

arch_suffix="$(detect_arch_suffix)"
asset_name="resvg-${arch_suffix}.tar.gz"
download_url="https://github.com/linebender/resvg/releases/download/${VERSION}/${asset_name}"
archive_path="$tmp_dir/$asset_name"

if ! expected_sha256="$(resvg_sha256_for "$VERSION" "$arch_suffix")"; then
  echo "No SHA256 configured for resvg ${VERSION} (${arch_suffix})." >&2
  exit 1
fi

curl -fsSL "$download_url" -o "$archive_path"
printf '%s  %s\n' "$expected_sha256" "$archive_path" | sha256sum -c -
tar -xzf "$archive_path" -C "$tmp_dir"
mkdir -p "$HOME/.local/bin"
install -m 0755 "$tmp_dir/resvg" "$HOME/.local/bin/resvg"
echo "$HOME/.local/bin" >> "$GITHUB_PATH"
"$HOME/.local/bin/resvg" --version
