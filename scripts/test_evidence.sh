#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_DIR="artifacts/ci"
COVERAGE_THRESHOLD_LINES=95

mkdir -p "${ARTIFACT_DIR}"

# Ensure llvm-tools for cargo-llvm-cov
if command -v rustup >/dev/null 2>&1; then
  if ! rustup component list --installed | grep -q '^llvm-tools-preview'; then
    rustup component add llvm-tools-preview
  fi
fi

run_logged() {
  local outfile="$1"; shift
  {
    echo "## $(date -u +%Y-%m-%dT%H:%M:%SZ) :: $*"
  } > "${outfile}"
  "$@" 2>&1 | tee -a "${outfile}"
}

run_logged "${ARTIFACT_DIR}/discos_fmt_output.txt" \
  cargo fmt --all -- --check

run_logged "${ARTIFACT_DIR}/clippy-report.txt" \
  cargo clippy --workspace --all-targets --all-features -- -D warnings

run_logged "${ARTIFACT_DIR}/implementation_honesty_gate.txt" \
  ./scripts/check_implementation_honesty.sh

run_logged "${ARTIFACT_DIR}/test_output.txt" \
  cargo test --workspace --all-targets --all-features

{
  echo "## $(date -u +%Y-%m-%dT%H:%M:%SZ) :: cargo llvm-cov ..."
} > "${ARTIFACT_DIR}/discos_coverage_output.txt"

cargo llvm-cov \
  --workspace --all-features \
  --lcov --output-path "${ARTIFACT_DIR}/coverage.lcov" \
  --fail-under-lines "${COVERAGE_THRESHOLD_LINES}" \
  2>&1 | tee -a "${ARTIFACT_DIR}/discos_coverage_output.txt"

run_logged "${ARTIFACT_DIR}/fuzz_structured_claims_json.txt" \
  bash -lc 'cd fuzz && cargo +nightly fuzz run fuzz_structured_claims_json -- -max_total_time=20'

run_logged "${ARTIFACT_DIR}/fuzz_structured_claims_canonical.txt" \
  bash -lc 'cd fuzz && cargo +nightly fuzz run fuzz_structured_claims_canonical -- -max_total_time=20'

run_logged "${ARTIFACT_DIR}/fuzz_structured_claim_parse_canonicalize.txt" \
  bash -lc 'cd fuzz && cargo +nightly fuzz run fuzz_structured_claim_parse_canonicalize -- -max_total_time=10'

required_files=(
  "${ARTIFACT_DIR}/discos_fmt_output.txt"
  "${ARTIFACT_DIR}/clippy-report.txt"
  "${ARTIFACT_DIR}/implementation_honesty_gate.txt"
  "${ARTIFACT_DIR}/test_output.txt"
  "${ARTIFACT_DIR}/discos_coverage_output.txt"
  "${ARTIFACT_DIR}/coverage.lcov"
  "${ARTIFACT_DIR}/fuzz_structured_claims_json.txt"
  "${ARTIFACT_DIR}/fuzz_structured_claims_canonical.txt"
  "${ARTIFACT_DIR}/fuzz_structured_claim_parse_canonicalize.txt"
)

for f in "${required_files[@]}"; do
  if [[ ! -f "${f}" ]]; then
    echo "[FAIL] Missing required CI artifact: ${f}" >&2
    exit 1
  fi
done

if [[ ! -s "${ARTIFACT_DIR}/coverage.lcov" ]]; then
  echo "[FAIL] coverage.lcov was empty" >&2
  exit 1
fi

echo "[OK] CI evidence artifacts produced in ${ARTIFACT_DIR}"
