#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_ROOT="$(cd -- "${SCRIPT_DIRECTORY}/../.." && pwd)"
readonly LCOV_REPORT="$(mktemp "${TMPDIR:-/tmp}/apollo18-workspace-lcov.XXXXXX")"

trap 'rm -f -- "${LCOV_REPORT}"' EXIT

cd -- "${REPOSITORY_ROOT}"
cargo fmt --check
cargo arc check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --locked --package apollo18-web --target wasm32-unknown-unknown
cargo llvm-cov --workspace --lcov --output-path "${LCOV_REPORT}"
cargo crap --package apollo18-renderer --lcov "${LCOV_REPORT}" --fail-above
