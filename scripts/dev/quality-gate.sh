#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_ROOT="$(cd -- "${SCRIPT_DIRECTORY}/../.." && pwd)"

cd -- "${REPOSITORY_ROOT}"
cargo fmt --check
cargo arc check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

trunk_arguments=(index.html --release --locked)
if [[ -n "${PUBLIC_URL:-}" ]]; then
    trunk_arguments+=(--public-url "${PUBLIC_URL}")
fi

cd -- "${REPOSITORY_ROOT}/crates/web"
NO_COLOR=true trunk build "${trunk_arguments[@]}"
