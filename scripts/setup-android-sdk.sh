#!/bin/bash
set -euo pipefail

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SDK_ROOT="${ANDROID_SDK_ROOT:-${REPO_ROOT}/.android-sdk}"
TOOLS_VERSION="${ANDROID_CMDLINE_TOOLS_VERSION:-13114758}"

case "$(uname -s)" in
  Linux)
    TOOLS_URL="${ANDROID_CMDLINE_TOOLS_URL:-https://dl.google.com/android/repository/commandlinetools-linux-${TOOLS_VERSION}_latest.zip}"
    ;;
  Darwin)
    TOOLS_URL="${ANDROID_CMDLINE_TOOLS_URL:-https://dl.google.com/android/repository/commandlinetools-mac-${TOOLS_VERSION}_latest.zip}"
    ;;
  *)
    echo "Unsupported host OS for automatic Android SDK setup." >&2
    exit 1
    ;;
esac

TMP_DIR="${REPO_ROOT}/tmp/android-sdk"
ARCHIVE_PATH="${TMP_DIR}/cmdline-tools.zip"
TOOLS_ROOT="${SDK_ROOT}/cmdline-tools"
LATEST_DIR="${TOOLS_ROOT}/latest"

mkdir -p "${TMP_DIR}" "${TOOLS_ROOT}" "${SDK_ROOT}"

if [[ ! -x "${LATEST_DIR}/bin/sdkmanager" ]]; then
  rm -rf "${LATEST_DIR}"
  mkdir -p "${LATEST_DIR}"
  curl -L "${TOOLS_URL}" -o "${ARCHIVE_PATH}"
  unzip -q "${ARCHIVE_PATH}" -d "${TMP_DIR}/unzipped"
  rm -rf "${LATEST_DIR}"
  mv "${TMP_DIR}/unzipped/cmdline-tools" "${LATEST_DIR}"
fi

export ANDROID_SDK_ROOT="${SDK_ROOT}"
export ANDROID_HOME="${SDK_ROOT}"
export PATH="${LATEST_DIR}/bin:${SDK_ROOT}/platform-tools:${PATH}"

set +o pipefail
yes | sdkmanager --licenses >/dev/null
set -o pipefail
sdkmanager \
  "platform-tools" \
  "platforms;android-35" \
  "build-tools;35.0.0"

echo "ANDROID_SDK_ROOT=${ANDROID_SDK_ROOT}"
