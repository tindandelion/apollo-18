#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_ROOT="$(cd -- "${SCRIPT_DIRECTORY}/../.." && pwd)"
readonly WEB_DIRECTORY="${REPOSITORY_ROOT}/crates/web"
readonly BUILD_DIRECTORY="${REPOSITORY_ROOT}/target/apollo18/web-wasm-size"
readonly DIST_DIRECTORY="${BUILD_DIRECTORY}/dist"

command -v trunk >/dev/null 2>&1 || {
  echo "error: trunk is required (expected version 0.21.14)" >&2
  exit 1
}
command -v gzip >/dev/null 2>&1 || {
  echo "error: gzip is required" >&2
  exit 1
}

rm -rf -- "${DIST_DIRECTORY}"
mkdir -p -- "${BUILD_DIRECTORY}"

cd -- "${WEB_DIRECTORY}"
NO_COLOR=true trunk build --release --dist "${DIST_DIRECTORY}"

wasm_artifacts=("${DIST_DIRECTORY}"/*.wasm)
if [[ ! -f "${wasm_artifacts[0]}" || ${#wasm_artifacts[@]} -ne 1 ]]; then
  echo "error: expected exactly one top-level Wasm artifact in ${DIST_DIRECTORY}" >&2
  exit 1
fi

readonly WASM_ARTIFACT="${wasm_artifacts[0]}"
readonly RAW_BYTES="$(wc -c < "${WASM_ARTIFACT}" | tr -d '[:space:]')"
readonly GZIP_BYTES="$(gzip -9 -n -c -- "${WASM_ARTIFACT}" | wc -c | tr -d '[:space:]')"
readonly ARTIFACT_PATH="${WASM_ARTIFACT#"${REPOSITORY_ROOT}/"}"

printf '\nApollo 18 release Wasm size\n'
printf 'artifact: %s\n' "${ARTIFACT_PATH}"
printf 'raw_bytes: %s\n' "${RAW_BYTES}"
printf 'gzip_bytes: %s\n' "${GZIP_BYTES}"
