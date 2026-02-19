#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_DIR="artifacts/ci"
mkdir -p "$ARTIFACT_DIR"

cargo fmt --all -- --check | tee "$ARTIFACT_DIR/discos_fmt_output.txt"
cargo clippy --workspace --all-targets --all-features -- -D warnings | tee "$ARTIFACT_DIR/discos_clippy_output.txt"
cargo test --workspace --all-targets --all-features | tee "$ARTIFACT_DIR/discos_test_output.txt"

if command -v cargo-llvm-cov >/dev/null 2>&1; then
  cargo llvm-cov --workspace --all-features --lcov --output-path "$ARTIFACT_DIR/lcov.info" --fail-under-lines 90 \
    | tee "$ARTIFACT_DIR/discos_coverage_output.txt"
else
  echo "cargo-llvm-cov not installed; skipping coverage gate" | tee "$ARTIFACT_DIR/discos_coverage_output.txt"
fi
