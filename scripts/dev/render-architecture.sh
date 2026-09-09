#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_ROOT="$(cd -- "${SCRIPT_DIRECTORY}/../.." && pwd)"

output_path="${ARCHITECTURE_OUTPUT:-target/apollo18/architecture/dependencies.svg}"
if [[ "${output_path}" != /* ]]; then
    output_path="${REPOSITORY_ROOT}/${output_path}"
fi
readonly output_path

if ! cargo arc --version >/dev/null 2>&1; then
    echo "error: cargo-arc 0.3.1 is required; install it with:" >&2
    echo "  cargo install cargo-arc --version 0.3.1 --locked" >&2
    exit 1
fi

mkdir -p -- "$(dirname -- "${output_path}")"
cd -- "${REPOSITORY_ROOT}"
cargo arc --expand-level 1 --output "${output_path}" "$@"

printf 'Architecture diagram written to %s\n' "${output_path}"

open -- "${output_path}"
