#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets

# Coverage gate (requires llvm-tools + cargo-llvm-cov in CI image)
if command -v cargo-llvm-cov >/dev/null 2>&1; then
  cargo llvm-cov --workspace --lcov --output-path lcov.info --fail-under-lines 90
else
  echo "cargo-llvm-cov not installed; skipping coverage gate" >&2
fi
