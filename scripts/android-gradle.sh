#!/bin/bash
set -euo pipefail

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
GRADLE_VERSION="${MAZOCARTA_GRADLE_VERSION:-8.10.2}"
GRADLE_BOOTSTRAP_ROOT="${MAZOCARTA_GRADLE_BOOTSTRAP_ROOT:-${REPO_ROOT}/.gradle-bootstrap}"
GRADLE_HOME="${MAZOCARTA_GRADLE_HOME:-${GRADLE_BOOTSTRAP_ROOT}/gradle-${GRADLE_VERSION}}"
GRADLE_BIN="${GRADLE_HOME}/bin/gradle"
GRADLE_ARCHIVE="${GRADLE_BOOTSTRAP_ROOT}/gradle-${GRADLE_VERSION}-bin.zip"
GRADLE_URL="${MAZOCARTA_GRADLE_URL:-https://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip}"

if [[ ! -x "${GRADLE_BIN}" ]]; then
  if [[ -n "${MAZOCARTA_GRADLE_HOME:-}" ]]; then
    echo "Gradle not found at MAZOCARTA_GRADLE_HOME: ${MAZOCARTA_GRADLE_HOME}" >&2
    exit 1
  fi

  TMP_DIR="${GRADLE_BOOTSTRAP_ROOT}/tmp"
  mkdir -p "${GRADLE_BOOTSTRAP_ROOT}"
  if [[ ! -f "${GRADLE_ARCHIVE}" ]]; then
    curl -L "${GRADLE_URL}" -o "${GRADLE_ARCHIVE}.tmp"
    mv "${GRADLE_ARCHIVE}.tmp" "${GRADLE_ARCHIVE}"
  fi

  rm -rf "${TMP_DIR}"
  mkdir -p "${TMP_DIR}"
  unzip -q "${GRADLE_ARCHIVE}" -d "${TMP_DIR}"
  rm -rf "${GRADLE_HOME}"
  mv "${TMP_DIR}/gradle-${GRADLE_VERSION}" "${GRADLE_HOME}"
  rm -rf "${TMP_DIR}"
fi

case "${1:-}" in
  --version|-v)
    ;;
  *)
    source "${REPO_ROOT}/scripts/android-env.sh"
    ;;
esac

exec "${GRADLE_BIN}" -p "${REPO_ROOT}/android" "$@"
