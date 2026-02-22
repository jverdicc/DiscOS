#!/usr/bin/env bash
set -euo pipefail

ADDR="${EVIDENCEOS_DAEMON_ADDR:-http://127.0.0.1:50051}"
BIN="${EVIDENCEOS_DAEMON_BIN:-evidenceos-daemon}"
LISTEN="${ADDR#http://}"
REV="${EVIDENCEOS_REV:-4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af}"
REPO="${EVIDENCEOS_REPO:-https://github.com/EvidenceOS/evidenceos.git}"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT_DIR="artifacts/system-test/${TS}"
DATA_DIR="${OUT_DIR}/data"
LOG_FILE="${OUT_DIR}/daemon.log"

mkdir -p "${DATA_DIR}" "${OUT_DIR}"

resolve_daemon_bin() {
  if [[ -x "${BIN}" ]]; then
    echo "${BIN}"
    return
  fi

  if command -v "${BIN}" >/dev/null 2>&1; then
    command -v "${BIN}"
    return
  fi

  local vendor_dir="${OUT_DIR}/EvidenceOS"
  echo "evidenceos-daemon not found; cloning ${REPO}@${REV} for system test" >&2
  git clone --quiet "${REPO}" "${vendor_dir}"
  git -C "${vendor_dir}" checkout --quiet "${REV}"
  cargo build --manifest-path "${vendor_dir}/Cargo.toml" --bin evidenceos-daemon --release >/dev/null
  echo "${vendor_dir}/target/release/evidenceos-daemon"
}

BIN="$(resolve_daemon_bin)"

run_json() {
  local outfile="$1"
  shift
  "$@" | tee "${outfile}" >/dev/null
}

"${BIN}" --listen "${LISTEN}" --data-dir "${DATA_DIR}" >"${LOG_FILE}" 2>&1 &
DAEMON_PID=$!
trap 'kill ${DAEMON_PID} >/dev/null 2>&1 || true; wait ${DAEMON_PID} >/dev/null 2>&1 || true' EXIT

for _ in $(seq 1 60); do
  if cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" health >"${OUT_DIR}/health.json" 2>&1; then
    break
  fi
  sleep 1
done

run_json "${OUT_DIR}/server_info.json" \
  cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" server-info

python - <<'PYINFO' "${OUT_DIR}/server_info.json" "${REV}"
import json, pathlib, sys
info = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding='utf-8'))
expected_rev = sys.argv[2]
assert info.get('expected_rev') == expected_rev, 'server-info expected_rev mismatch'
assert info.get('protocol_package') == 'evidenceos.v1', 'unexpected package name'
assert isinstance(info.get('key_ids'), list), 'key_ids should be a list'
print('server-info compatibility assertions passed')
PYINFO

run_json "${OUT_DIR}/validate_pass.json" \
  cargo run --quiet -p discos-cli -- claim validate-structured \
  --input test_vectors/structured_claims/valid/pass_max_boundaries.json

run_json "${OUT_DIR}/validate_heavy.json" \
  cargo run --quiet -p discos-cli -- claim validate-structured \
  --input test_vectors/structured_claims/valid/heavy_min.json

if cargo run --quiet -p discos-cli -- claim validate-structured \
  --input test_vectors/structured_claims/invalid/float_value_q.json >"${OUT_DIR}/validate_invalid_float.json" 2>&1; then
  echo "invalid float vector unexpectedly accepted" >&2
  exit 1
fi

if cargo run --quiet -p discos-cli -- claim validate-structured \
  --input test_vectors/structured_claims/invalid/heavy_missing_reason.json >"${OUT_DIR}/validate_invalid_heavy.json" 2>&1; then
  echo "invalid heavy vector unexpectedly accepted" >&2
  exit 1
fi

run_json "${OUT_DIR}/create_a.json" \
  cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim create \
  --claim-name claim-a --alpha-micros 50000 --lane high_assurance --epoch-config-ref epoch/default \
  --output-schema-id cbrn-sc.v1 --holdout-ref holdout/default --epoch-size 1 --oracle-num-symbols 4 --access-credit 1

CLAIM_A="$(python - <<'PY' "${OUT_DIR}/create_a.json"
import json,sys
print(json.loads(open(sys.argv[1], encoding='utf-8').read())['claim_id'])
PY
)"

run_json "${OUT_DIR}/commit_a.json" \
  cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim commit --claim-id "${CLAIM_A}" \
  --wasm .discos/claims/claim-a/wasm.bin --manifests .discos/claims/claim-a/alpha_hir.json .discos/claims/claim-a/phys_hir.json .discos/claims/claim-a/causal_dsl.json

run_json "${OUT_DIR}/freeze_a.json" \
  cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim freeze --claim-id "${CLAIM_A}"

run_json "${OUT_DIR}/execute_a.json" \
  cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim execute --claim-id "${CLAIM_A}"

run_json "${OUT_DIR}/seal_a.json" \
  cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim seal --claim-id "${CLAIM_A}"

run_json "${OUT_DIR}/fetch_a.json" \
  cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" claim fetch-capsule --claim-id "${CLAIM_A}" --verify-etl

python - <<'PY' "${OUT_DIR}/commit_a.json" "${OUT_DIR}/freeze_a.json" "${OUT_DIR}/execute_a.json" "${OUT_DIR}/seal_a.json" "${OUT_DIR}/fetch_a.json"
import json, pathlib, sys

commit = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding='utf-8'))
freeze = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding='utf-8'))
execute = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding='utf-8'))
seal = json.loads(pathlib.Path(sys.argv[4]).read_text(encoding='utf-8'))
fetch = json.loads(pathlib.Path(sys.argv[5]).read_text(encoding='utf-8'))

assert commit.get('accepted') is True, 'commit was not accepted'
assert freeze.get('frozen') is True, 'freeze failed'
assert isinstance(execute.get('e_value'), (int, float)), 'execute response missing e_value'
assert seal.get('sealed') is True, 'seal failed'
assert fetch.get('inclusion_ok') is True, 'inclusion proof verification failed'
assert isinstance(fetch.get('consistency_ok'), bool), 'consistency check flag missing'
print('system test assertions passed')
PY


./scripts/probe_simulation.sh --endpoint "${ADDR}" --claims 12 --unique-hashes 12 --topics 3 --artifact-dir "${OUT_DIR}/probe-sim" --require-controls

cargo run --quiet -p evidenceos-redteam -- --endpoint "${ADDR}" --rounds 12 --strict-pln true --production-mode true >"${OUT_DIR}/redteam_report.json"

cargo test -p discos-client --test e2e_against_daemon_v2 -- --ignored | tee "${OUT_DIR}/daemon_contract_test.log" >/dev/null
cargo test -p discos-builder --test evidenceos_vault_system -- --ignored | tee "${OUT_DIR}/builder_vault_system_test.log" >/dev/null

echo "system test outputs at ${OUT_DIR}" | tee "${OUT_DIR}/summary.txt" >/dev/null
