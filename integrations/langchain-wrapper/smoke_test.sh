#!/usr/bin/env bash
set -euo pipefail
ADDR="${EVIDENCEOS_DAEMON_ADDR:-http://127.0.0.1:50051}"

cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" health >/tmp/discos_langchain_health.json
cargo run --quiet -p discos-cli -- --endpoint "${ADDR}" server-info >/tmp/discos_langchain_server_info.json

python - <<'PY'
import json
health = json.loads(open('/tmp/discos_langchain_health.json', encoding='utf-8').read())
info = json.loads(open('/tmp/discos_langchain_server_info.json', encoding='utf-8').read())
assert health.get('status') == 'ok'
assert info.get('protocol_package') == 'evidenceos.v1'
print('langchain wrapper smoke test passed')
PY
