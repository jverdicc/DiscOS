#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_DIR="artifacts/ci"
COVERAGE_THRESHOLD_LINES=95
CI_STRICT="${DISCOS_CI_STRICT:-${CI:-0}}"
SKIP_COVERAGE="${SKIP_COVERAGE:-0}"
SKIP_FUZZ="${SKIP_FUZZ:-0}"

mkdir -p "${ARTIFACT_DIR}"

# Ensure llvm-tools for cargo-llvm-cov
if command -v rustup >/dev/null 2>&1; then
  if ! rustup component list --installed | grep -q '^llvm-tools-preview'; then
    if ! rustup component add llvm-tools-preview; then
      if [[ "${CI_STRICT}" == "1" ]]; then
        echo "[FAIL] unable to install llvm-tools-preview" >&2
        exit 1
      fi
      echo "[WARN] unable to install llvm-tools-preview; skipping coverage in non-strict mode" >&2
      SKIP_COVERAGE=1
    fi
  fi
fi

if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! cargo llvm-cov --version >/dev/null 2>&1; then
  if [[ "${CI_STRICT}" == "1" ]]; then
    echo "[FAIL] cargo llvm-cov is not available" >&2
    exit 1
  fi
  echo "[WARN] cargo llvm-cov is not available; skipping coverage in non-strict mode" >&2
  SKIP_COVERAGE=1
fi

if ! cargo +nightly --version >/dev/null 2>&1; then
  if [[ "${CI_STRICT}" == "1" ]]; then
    echo "[FAIL] nightly toolchain is not available" >&2
    exit 1
  fi
  echo "[WARN] nightly toolchain not available; skipping fuzz in non-strict mode" >&2
  SKIP_FUZZ=1
fi

run_logged() {
  local outfile="$1"; shift
  {
    echo "## $(date -u +%Y-%m-%dT%H:%M:%SZ) :: $*"
  } > "${outfile}"
  if "$@" 2>&1 | tee -a "${outfile}"; then
    return 0
  fi
  if [[ "${CI_STRICT}" == "1" ]]; then
    return 1
  fi
  echo "[WARN] non-strict mode ignoring failure: $*" | tee -a "${outfile}"
  return 0
}

run_logged "${ARTIFACT_DIR}/discos_fmt_output.txt" \
  cargo fmt --all -- --check

run_logged "${ARTIFACT_DIR}/clippy-report.txt" \
  cargo clippy --workspace --all-targets --exclude discos-cli -- -D warnings

run_logged "${ARTIFACT_DIR}/implementation_honesty_gate.txt" \
  ./scripts/check_implementation_honesty.sh

run_logged "${ARTIFACT_DIR}/test_output.txt" \
  cargo test --workspace --exclude discos-cli

{
  echo "## $(date -u +%Y-%m-%dT%H:%M:%SZ) :: cargo llvm-cov ..."
} > "${ARTIFACT_DIR}/discos_coverage_output.txt"

if [[ "${SKIP_COVERAGE}" == "1" ]]; then
  echo "[WARN] coverage step skipped" | tee -a "${ARTIFACT_DIR}/discos_coverage_output.txt"
  echo "TN:skipped" > "${ARTIFACT_DIR}/coverage.lcov"
else
  if cargo llvm-cov \
    --workspace --exclude discos-cli \
    --lcov --output-path "${ARTIFACT_DIR}/coverage.lcov" \
    --fail-under-lines "${COVERAGE_THRESHOLD_LINES}" \
    2>&1 | tee -a "${ARTIFACT_DIR}/discos_coverage_output.txt"; then
    :
  elif [[ "${CI_STRICT}" == "1" ]]; then
    exit 1
  else
    echo "[WARN] non-strict mode ignoring coverage failure" | tee -a "${ARTIFACT_DIR}/discos_coverage_output.txt"
    echo "TN:skipped" > "${ARTIFACT_DIR}/coverage.lcov"
  fi
fi

if [[ "${SKIP_FUZZ}" == "1" ]]; then
  printf '[WARN] fuzz step skipped\n' > "${ARTIFACT_DIR}/fuzz_structured_claims_json.txt"
  printf '[WARN] fuzz step skipped\n' > "${ARTIFACT_DIR}/fuzz_structured_claims_canonical.txt"
  printf '[WARN] fuzz step skipped\n' > "${ARTIFACT_DIR}/fuzz_structured_claim_parse_canonicalize.txt"
else
  run_logged "${ARTIFACT_DIR}/fuzz_structured_claims_json.txt" \
    bash -lc 'cd fuzz && cargo +nightly fuzz run fuzz_structured_claims_json -- -max_total_time=20'

  run_logged "${ARTIFACT_DIR}/fuzz_structured_claims_canonical.txt" \
    bash -lc 'cd fuzz && cargo +nightly fuzz run fuzz_structured_claims_canonical -- -max_total_time=20'

  run_logged "${ARTIFACT_DIR}/fuzz_structured_claim_parse_canonicalize.txt" \
    bash -lc 'cd fuzz && cargo +nightly fuzz run fuzz_structured_claim_parse_canonicalize -- -max_total_time=10'
fi

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
