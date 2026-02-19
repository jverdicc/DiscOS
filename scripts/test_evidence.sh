#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_DIR="artifacts/ci"
mkdir -p "$ARTIFACT_DIR"

cargo fmt --all -- --check | tee "$ARTIFACT_DIR/discos_fmt_output.txt"
cargo clippy --workspace --all-targets --all-features -- -D warnings | tee "$ARTIFACT_DIR/discos_clippy_output.txt"
cargo test --workspace --all-targets --all-features | tee "$ARTIFACT_DIR/test_output.txt"

cargo llvm-cov --workspace --all-features --lcov --output-path "$ARTIFACT_DIR/coverage.lcov" --fail-under-lines 95 \
  | tee "$ARTIFACT_DIR/discos_coverage_output.txt"

(
  cd fuzz
  cargo fuzz run fuzz_structured_claims_json -- -max_total_time=20
) | tee "$ARTIFACT_DIR/fuzz_structured_claims_json.txt"
(
  cd fuzz
  cargo fuzz run fuzz_structured_claims_canonical -- -max_total_time=20
) | tee "$ARTIFACT_DIR/fuzz_structured_claims_canonical.txt"
(
  cd fuzz
  cargo fuzz run fuzz_structured_claim_parse_canonicalize -- -max_total_time=10
) | tee "$ARTIFACT_DIR/fuzz_structured_claim_parse_canonicalize.txt"

