#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_ROOT="$(cd -- "${SCRIPT_DIRECTORY}/.." && pwd)"
readonly BROWSER_TEST_DIRECTORY="${REPOSITORY_ROOT}/crates/web/smoke-tests"

cd -- "${BROWSER_TEST_DIRECTORY}"
exec npm run test:performance -- "$@"
