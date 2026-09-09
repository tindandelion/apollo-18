#!/usr/bin/env bash
# Temporary Ticket 19 measurement helper. Rebuilds the release Wasm, runs the
# staged browser profile, and prints one labelled result line.
set -euo pipefail

readonly LABEL="${1:?usage: measure.sh <label>}"
readonly REPOSITORY_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"

cd -- "${REPOSITORY_ROOT}/crates/web"
NO_COLOR=true trunk build index.html --release >/dev/null 2>"${REPOSITORY_ROOT}/.scratch/apollo-18/perf/build.log" || {
    echo "BUILD FAILED for ${LABEL}" >&2
    tail -30 "${REPOSITORY_ROOT}/.scratch/apollo-18/perf/build.log" >&2
    exit 1
}

cd -- "${REPOSITORY_ROOT}/crates/web/smoke-tests"
result="$(npx playwright test zz-profile.spec.js 2>&1 | grep 'APOLLO18_PROFILE')"
printf '%s\t%s\n' "${LABEL}" "${result#APOLLO18_PROFILE }" \
    | tee -a "${REPOSITORY_ROOT}/.scratch/apollo-18/perf/results.tsv"
