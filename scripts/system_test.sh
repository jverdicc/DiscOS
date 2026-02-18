#!/usr/bin/env bash
set -euo pipefail

ADDR="${EVIDENCEOS_DAEMON_ADDR:-http://127.0.0.1:50051}"
BIN="${EVIDENCEOS_DAEMON_BIN:-evidenceos-daemon}"
LISTEN="${ADDR#http://}"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT_DIR="artifacts/system-test/${TS}"
DATA_DIR="${OUT_DIR}/data"
LOG_FILE="${OUT_DIR}/daemon.log"

mkdir -p "${DATA_DIR}" "${OUT_DIR}"

"${BIN}" --listen "${LISTEN}" --data-dir "${DATA_DIR}" >"${LOG_FILE}" 2>&1 &
DAEMON_PID=$!
trap 'kill ${DAEMON_PID} >/dev/null 2>&1 || true; wait ${DAEMON_PID} >/dev/null 2>&1 || true' EXIT

for _ in $(seq 1 60); do
  if cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" health >"${OUT_DIR}/health.json" 2>&1; then
    break
  fi
  sleep 1
done

cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim create \
  --claim-name claim-a --alpha-micros 50000 --lane cbrn --epoch-config-ref epoch/v1 \
  --holdout-ref holdout/default --epoch-size 1 --oracle-num-symbols 1 --access-credit 1 \
  >"${OUT_DIR}/create_a.json"
CLAIM_A="$(python - <<'PY' "${OUT_DIR}/create_a.json"
import json,sys
print(json.loads(open(sys.argv[1]).read())['claim_id'])
PY
)"

cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim commit --claim-id "${CLAIM_A}" --wasm .discos/claims/claim-a/wasm.bin --manifests .discos/claims/claim-a/alpha_hir.json .discos/claims/claim-a/phys_hir.json .discos/claims/claim-a/causal_dsl.json >"${OUT_DIR}/commit_a.json"
cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim freeze --claim-id "${CLAIM_A}" >"${OUT_DIR}/freeze_a.json"
cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim seal --claim-id "${CLAIM_A}" >"${OUT_DIR}/seal_a.json"
cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim execute --claim-id "${CLAIM_A}" >"${OUT_DIR}/execute_a.json"
cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim fetch-capsule --claim-id "${CLAIM_A}" --verify-etl >"${OUT_DIR}/fetch_a.json"

cargo test -p discos-client --test e2e_against_daemon_v2 -- --ignored >"${OUT_DIR}/daemon_contract_test.log"

echo "system test outputs at ${OUT_DIR}" > "${OUT_DIR}/summary.txt"
