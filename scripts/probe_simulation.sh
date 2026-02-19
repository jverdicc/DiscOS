#!/usr/bin/env bash
set -euo pipefail

ENDPOINT="http://127.0.0.1:50051"
CLAIMS=200
UNIQUE_HASHES=200
TOPICS=10
SEED=424242
ARTIFACT_DIR="artifacts/probe-sim"
REQUIRE_CONTROLS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --endpoint)
      ENDPOINT="$2"; shift 2 ;;
    --claims)
      CLAIMS="$2"; shift 2 ;;
    --unique-hashes)
      UNIQUE_HASHES="$2"; shift 2 ;;
    --topics)
      TOPICS="$2"; shift 2 ;;
    --seed)
      SEED="$2"; shift 2 ;;
    --artifact-dir)
      ARTIFACT_DIR="$2"; shift 2 ;;
    --require-controls)
      REQUIRE_CONTROLS=1; shift ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2 ;;
  esac
done

mkdir -p "$ARTIFACT_DIR"
SUMMARY_JSON="$ARTIFACT_DIR/probe_simulation_summary.json"
REQUESTS_JSONL="$ARTIFACT_DIR/probe_simulation_requests.jsonl"
HUMAN_TXT="$ARTIFACT_DIR/probe_simulation_human.txt"
: > "$REQUESTS_JSONL"

SUCCESS_COUNT=0
THROTTLE_AT="null"
ESCALATE_AT="null"
FREEZE_AT="null"
FIRST_FAILURE_AT="null"

for ((i=0; i<CLAIMS; i++)); do
  claim_name=$(python - <<'PY' "$SEED" "$i" "$UNIQUE_HASHES"
import hashlib,sys
seed=int(sys.argv[1]); idx=int(sys.argv[2]); uniq=max(1,int(sys.argv[3]))
slot=idx % uniq
raw=f"probe-{seed}-{slot}".encode()
print(hashlib.sha256(raw).hexdigest()[:24])
PY
)
  topic_idx=$((i % TOPICS))
  lane="probe-lane-$topic_idx"
  output_schema_id="cbrn-sc.v1"

  create_output=$(cargo run --quiet -p discos-cli -- --endpoint "$ENDPOINT" claim create \
    --claim-name "probe-${claim_name}-${i}" --alpha-micros 50000 --lane "$lane" \
    --epoch-config-ref "epoch/probe-$topic_idx" --output-schema-id "$output_schema_id" \
    --holdout-ref "holdout/probe" --epoch-size 1 --oracle-num-symbols 4 --access-credit 1)

  claim_id=$(python - <<'PY' "$create_output"
import json,sys
print(json.loads(sys.argv[1])["claim_id"])
PY
)

  cargo run --quiet -p discos-cli -- --endpoint "$ENDPOINT" claim commit --claim-id "$claim_id" \
    --wasm ".discos/claims/probe-${claim_name}-${i}/wasm.bin" \
    --manifests ".discos/claims/probe-${claim_name}-${i}/alpha_hir.json" \
    --manifests ".discos/claims/probe-${claim_name}-${i}/phys_hir.json" \
    --manifests ".discos/claims/probe-${claim_name}-${i}/causal_dsl.json" >/dev/null

  cargo run --quiet -p discos-cli -- --endpoint "$ENDPOINT" claim freeze --claim-id "$claim_id" >/dev/null

  status="PASS"
  message=""
  code=""
  execute_out_file="$ARTIFACT_DIR/execute_${i}.json"
  if cargo run --quiet -p discos-cli -- --endpoint "$ENDPOINT" claim execute --claim-id "$claim_id" >"$execute_out_file" 2>"$ARTIFACT_DIR/execute_${i}.stderr"; then
    SUCCESS_COUNT=$((SUCCESS_COUNT+1))
  else
    FIRST_FAILURE_AT=${FIRST_FAILURE_AT:-$i}
    code=$(python - <<'PY' "$ARTIFACT_DIR/execute_${i}.stderr"
import pathlib,re,sys
text=pathlib.Path(sys.argv[1]).read_text(encoding='utf-8', errors='ignore')
match=re.search(r'status: ([A-Za-z]+)', text)
print(match.group(1) if match else '')
PY
)
    message=$(python - <<'PY' "$ARTIFACT_DIR/execute_${i}.stderr"
import pathlib,re,sys
text=pathlib.Path(sys.argv[1]).read_text(encoding='utf-8', errors='ignore')
match=re.search(r'message: "([^"]+)"', text)
print(match.group(1) if match else text.strip().splitlines()[-1] if text.strip() else '')
PY
)
    case "$code" in
      ResourceExhausted)
        status="THROTTLE"
        if [[ "$THROTTLE_AT" == "null" ]]; then THROTTLE_AT=$i; fi
        ;;
      FailedPrecondition)
        status="FROZEN"
        if [[ "$FREEZE_AT" == "null" ]]; then FREEZE_AT=$i; fi
        ;;
      PermissionDenied)
        status="ESCALATE"
        if [[ "$ESCALATE_AT" == "null" ]]; then ESCALATE_AT=$i; fi
        ;;
      *)
        status="REJECT"
        ;;
    esac
  fi

  printf '{"index":%d,"claim_name":"%s","claim_id":"%s","topic_bucket":%d,"status":"%s","grpc_code":"%s","message":%s}\n' \
    "$i" "probe-${claim_name}-${i}" "$claim_id" "$topic_idx" "$status" "$code" "$(python - <<'PY' "$message"
import json,sys
print(json.dumps(sys.argv[1]))
PY
)" >> "$REQUESTS_JSONL"
done

python - <<'PY' "$SUMMARY_JSON" "$CLAIMS" "$SUCCESS_COUNT" "$THROTTLE_AT" "$ESCALATE_AT" "$FREEZE_AT" "$ARTIFACT_DIR" "$SEED" "$UNIQUE_HASHES" "$TOPICS"
import json,pathlib,sys
summary={
  "claims_total": int(sys.argv[2]),
  "claims_succeeded": int(sys.argv[3]),
  "throttle_started_at": None if sys.argv[4]=="null" else int(sys.argv[4]),
  "escalation_started_at": None if sys.argv[5]=="null" else int(sys.argv[5]),
  "freeze_started_at": None if sys.argv[6]=="null" else int(sys.argv[6]),
  "artifact_dir": sys.argv[7],
  "seed": int(sys.argv[8]),
  "unique_hashes": int(sys.argv[9]),
  "topics": int(sys.argv[10]),
}
path=pathlib.Path(sys.argv[1])
path.write_text(json.dumps(summary, indent=2)+"\n", encoding='utf-8')
print(json.dumps(summary))
PY

{
  echo "Probe simulation summary"
  echo "- endpoint: $ENDPOINT"
  echo "- claims succeeded: $SUCCESS_COUNT / $CLAIMS"
  echo "- throttle started at: $THROTTLE_AT"
  echo "- escalation started at: $ESCALATE_AT"
  echo "- freeze started at: $FREEZE_AT"
  echo "- artifacts: $ARTIFACT_DIR"
} | tee "$HUMAN_TXT"

if [[ "$REQUIRE_CONTROLS" -eq 1 ]]; then
  if [[ "$THROTTLE_AT" == "null" && "$FREEZE_AT" == "null" && "$ESCALATE_AT" == "null" ]]; then
    echo "probe simulation did not observe THROTTLE/ESCALATE/FROZEN signal" >&2
    exit 1
  fi
fi
