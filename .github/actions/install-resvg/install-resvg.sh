#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?usage: install-resvg.sh VERSION}"
tmp_dir="$(mktemp -d)"

cleanup() {
  rm -rf "$tmp_dir"
}

trap cleanup EXIT

curl -fsSL "https://github.com/linebender/resvg/releases/download/${VERSION}/resvg-linux-x86_64.tar.gz" \
  | tar -xz -C "$tmp_dir"
mkdir -p "$HOME/.local/bin"
install -m 0755 "$tmp_dir/resvg" "$HOME/.local/bin/resvg"
echo "$HOME/.local/bin" >> "$GITHUB_PATH"
"$HOME/.local/bin/resvg" --version
