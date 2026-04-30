#!/bin/bash
set -euo pipefail

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE_DIR="${REPO_ROOT}/web"
TARGET_DIR="${REPO_ROOT}/android/app/src/main/assets/site"
SVG_SOURCE="${SOURCE_DIR}/mazocarta.svg"
ANDROID_RES_DIR="${REPO_ROOT}/android/app/src/main/res"

mkdir -p "${TARGET_DIR}"
rm -rf "${TARGET_DIR}"
mkdir -p "${TARGET_DIR}"

cp -R "${SOURCE_DIR}/." "${TARGET_DIR}/"
rm -f "${TARGET_DIR}/.debug-mode.json"

"${REPO_ROOT}/scripts/render-android-icons.sh" "${SVG_SOURCE}" "${ANDROID_RES_DIR}"
