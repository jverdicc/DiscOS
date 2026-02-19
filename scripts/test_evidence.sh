#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_DIR="artifacts/ci"
COVERAGE_THRESHOLD_LINES=95
mkdir -p "$ARTIFACT_DIR"

cargo fmt --all -- --check | tee "$ARTIFACT_DIR/discos_fmt_output.txt"
cargo clippy --workspace --all-targets --all-features -- -D warnings | tee "$ARTIFACT_DIR/clippy-report.txt"
cargo test --workspace --all-targets --all-features | tee "$ARTIFACT_DIR/test_output.txt"

cargo llvm-cov --workspace --all-features --lcov --output-path "$ARTIFACT_DIR/coverage.lcov" --fail-under-lines "$COVERAGE_THRESHOLD_LINES" \
  | tee "$ARTIFACT_DIR/discos_coverage_output.txt"

(
  cd fuzz
  cargo +nightly fuzz run fuzz_structured_claims_json -- -max_total_time=20
) | tee "$ARTIFACT_DIR/fuzz_structured_claims_json.txt"
(
  cd fuzz
  cargo +nightly fuzz run fuzz_structured_claims_canonical -- -max_total_time=20
) | tee "$ARTIFACT_DIR/fuzz_structured_claims_canonical.txt"
(
  cd fuzz
  cargo +nightly fuzz run fuzz_structured_claim_parse_canonicalize -- -max_total_time=10
) | tee "$ARTIFACT_DIR/fuzz_structured_claim_parse_canonicalize.txt"

required_artifacts=(
  "$ARTIFACT_DIR/discos_fmt_output.txt"
  "$ARTIFACT_DIR/clippy-report.txt"
  "$ARTIFACT_DIR/test_output.txt"
  "$ARTIFACT_DIR/coverage.lcov"
  "$ARTIFACT_DIR/fuzz_structured_claims_json.txt"
  "$ARTIFACT_DIR/fuzz_structured_claims_canonical.txt"
  "$ARTIFACT_DIR/fuzz_structured_claim_parse_canonicalize.txt"
)

for artifact in "${required_artifacts[@]}"; do
  if [[ ! -s "$artifact" ]]; then
    echo "missing required CI artifact: $artifact" >&2
    exit 1
  fi
done
