#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python -m pip install -q -e "$ROOT_DIR/integrations/langchain-wrapper" --no-deps --no-build-isolation

python - <<'PY'
import json
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer

from langchain_evidenceos import EvidenceOSGuardCallbackHandler

class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path != "/v1/preflight_tool_call":
            self.send_response(404)
            self.end_headers()
            return
        length = int(self.headers.get("Content-Length", "0"))
        _payload = json.loads(self.rfile.read(length).decode("utf-8"))
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps({"decision": "ALLOW", "reasonCode": "SmokeAllow"}).encode("utf-8"))

    def log_message(self, format, *args):
        return

server = HTTPServer(("127.0.0.1", 0), Handler)
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()

try:
    base_url = f"http://127.0.0.1:{server.server_port}"
    guard = EvidenceOSGuardCallbackHandler(evidenceos_url=base_url, session_id="smoke", agent_id="smoke")
    result = guard.guard_tool_call(tool_name="read.docs", tool_input={"q": "ping"})
    assert result == {"q": "ping"}
    print("langchain wrapper smoke test passed")
finally:
    server.shutdown()
    thread.join(timeout=2)
PY
