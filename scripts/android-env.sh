#!/bin/bash
set -euo pipefail

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if [[ -z "${JAVA_HOME:-}" ]] && command -v javac >/dev/null 2>&1; then
  JAVAC_PATH=$(readlink -f "$(command -v javac)")
  export JAVA_HOME=$(CDPATH= cd -- "$(dirname "${JAVAC_PATH}")/.." && pwd)
fi

if [[ -n "${ANDROID_SDK_ROOT:-}" && -d "${ANDROID_SDK_ROOT}" ]]; then
  SDK_ROOT="${ANDROID_SDK_ROOT}"
elif [[ -n "${ANDROID_HOME:-}" && -d "${ANDROID_HOME}" ]]; then
  SDK_ROOT="${ANDROID_HOME}"
elif [[ -d "${REPO_ROOT}/.android-sdk" ]]; then
  SDK_ROOT="${REPO_ROOT}/.android-sdk"
elif [[ -d "${HOME}/Android/Sdk" ]]; then
  SDK_ROOT="${HOME}/Android/Sdk"
elif [[ -d "${HOME}/Library/Android/sdk" ]]; then
  SDK_ROOT="${HOME}/Library/Android/sdk"
else
  echo "Android SDK not found. Set ANDROID_SDK_ROOT or run ./scripts/setup-android-sdk.sh" >&2
  exit 1
fi

export ANDROID_SDK_ROOT="${SDK_ROOT}"
export ANDROID_HOME="${SDK_ROOT}"
export PATH="${ANDROID_SDK_ROOT}/platform-tools:${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin:${PATH}"
