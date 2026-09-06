#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_ROOT="$(cd -- "${SCRIPT_DIRECTORY}/../.." && pwd)"

cd -- "${REPOSITORY_ROOT}"
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cd -- "${REPOSITORY_ROOT}/crates/web"
NO_COLOR=true trunk build index.html --release
