"""End-to-end local example for EvidenceOSRunnableAdapter.

Run:
    python integrations/langchain-wrapper/examples/e2e_preflight_adapter.py
"""

from __future__ import annotations

import json
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer

from langchain_evidenceos import EvidenceOSGuardCallbackHandler, EvidenceOSRunnableAdapter


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path != "/v1/preflight_tool_call":
            self.send_response(404)
            self.end_headers()
            return

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(
            json.dumps(
                {
                    "decision": "DOWNGRADE",
                    "reasonCode": "Sanitized",
                    "rewrittenParams": {"query": "public roadmap"},
                    "budgetDelta": {"spent": 1, "remaining": 4},
                }
            ).encode("utf-8")
        )

    def log_message(self, format, *args):
        return


def tool_call(params: dict[str, str]) -> dict[str, str]:
    return {"answer": f"search results for: {params['query']}"}


if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        guard = EvidenceOSGuardCallbackHandler(
            evidenceos_url=f"http://127.0.0.1:{server.server_port}",
            session_id="example-session",
            agent_id="example-agent",
            max_retries=1,
            timeout_ms=500,
        )

        adapter = EvidenceOSRunnableAdapter(
            tool_name="search.web",
            tool_func=tool_call,
            guard=guard,
        )

        result = adapter.invoke({"query": "internal strategy"})
        print(json.dumps({"output": result.output, "policy_receipt": result.policy_receipt.__dict__}, indent=2))
    finally:
        server.shutdown()
        thread.join(timeout=2)
