#!/usr/bin/env bash
set -euo pipefail

ADDR="${EVIDENCEOS_DAEMON_ADDR:-http://127.0.0.1:50051}"
BIN="${EVIDENCEOS_DAEMON_BIN:-evidenceos-daemon}"
LISTEN="${ADDR#http://}"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT_DIR="artifacts/system-test/${TS}"
DATA_DIR="${OUT_DIR}/data"
LOG_FILE="${OUT_DIR}/daemon.log"
TEST_LOG="${OUT_DIR}/test.log"
READY_LOG="${OUT_DIR}/readiness.log"
SUMMARY="${OUT_DIR}/summary.txt"

mkdir -p "${DATA_DIR}"

echo "[system-test] daemon binary: ${BIN}"
echo "[system-test] daemon addr:   ${ADDR}"
echo "[system-test] output dir:    ${OUT_DIR}"

"${BIN}" --listen "${LISTEN}" --etl-path "${DATA_DIR}/etl.log" >"${LOG_FILE}" 2>&1 &
DAEMON_PID=$!

cleanup() {
  if kill -0 "${DAEMON_PID}" 2>/dev/null; then
    kill "${DAEMON_PID}" || true
    wait "${DAEMON_PID}" || true
  fi
}
trap cleanup EXIT

READY=0
for i in $(seq 1 60); do
  if cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" health >"${READY_LOG}" 2>&1; then
    READY=1
    break
  fi
  sleep 1
done

if [[ "${READY}" -ne 1 ]]; then
  echo "[system-test] ERROR: daemon did not become ready at ${ADDR} within timeout" | tee -a "${SUMMARY}"
  echo "[system-test] readiness output:" | tee -a "${SUMMARY}"
  cat "${READY_LOG}" | tee -a "${SUMMARY}"
  exit 1
fi

set +e
cargo test -p discos-client --test e2e_against_daemon_v2 -- --ignored | tee "${TEST_LOG}"
TEST_RC=${PIPESTATUS[0]}
set -e

if [[ "${TEST_RC}" -ne 0 ]]; then
  echo "[system-test] FAIL: ignored daemon e2e test failed (exit=${TEST_RC})" | tee "${SUMMARY}"
else
  echo "[system-test] PASS: ignored daemon e2e test passed" | tee "${SUMMARY}"
fi

echo "[system-test] logs: ${OUT_DIR}" | tee -a "${SUMMARY}"

exit "${TEST_RC}"
