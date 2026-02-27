import json
import os
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib import request as urllib_request

import pytest

from langchain_evidenceos import EvidenceOSDecisionError, EvidenceOSGuardCallbackHandler


EVIDENCEOS_URL = os.getenv("EVIDENCEOS_PREFLIGHT_URL")


class _ForwardProxy(BaseHTTPRequestHandler):
    target = ""
    request_ids: list[str] = []

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("content-length", "0")))
        request_id = self.headers.get("X-Request-Id")
        if request_id:
            self.request_ids.append(request_id)

        upstream_req = urllib_request.Request(
            url=f"{self.target}{self.path}",
            data=body,
            method="POST",
            headers={k: v for k, v in self.headers.items()},
        )

        try:
            with urllib_request.urlopen(upstream_req, timeout=5) as upstream_resp:
                response_body = upstream_resp.read()
                self.send_response(upstream_resp.status)
                for key, value in upstream_resp.headers.items():
                    self.send_header(key, value)
                self.end_headers()
                self.wfile.write(response_body)
        except Exception as exc:  # pragma: no cover - exercised in CI only
            self.send_response(502)
            self.send_header("content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"error": str(exc)}).encode("utf-8"))

    def log_message(self, _fmt, *_args):
        return


@pytest.mark.skipif(not EVIDENCEOS_URL, reason="EVIDENCEOS_PREFLIGHT_URL not set")
def test_e2e_preflight_contract_against_daemon():
    _ForwardProxy.target = EVIDENCEOS_URL
    _ForwardProxy.request_ids = []

    server = HTTPServer(("127.0.0.1", 0), _ForwardProxy)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        guard = EvidenceOSGuardCallbackHandler(
            evidenceos_url=f"http://127.0.0.1:{server.server_port}",
            session_id="e2e-session",
            agent_id="e2e-agent",
            timeout_ms=5000,
            max_retries=0,
        )

        candidates = [
            ("search.web", {"query": "internal secrets"}),
            ("fs.delete_tree", {"path": "/tmp/demo"}),
            ("exec", {"cmd": "cat /etc/shadow"}),
        ]

        observed_rewrite = False
        requests_made = 0

        for tool_name, params in candidates[:2]:
            result = guard.preflight_tool_call(tool_name=tool_name, tool_input=params)
            requests_made += 1
            if result.receipt.decision == "DOWNGRADE" and result.params != params:
                observed_rewrite = True

        exec_name, exec_params = candidates[2]
        for _ in range(64):
            if observed_rewrite:
                break
            try:
                result = guard.preflight_tool_call(tool_name=exec_name, tool_input=exec_params)
                requests_made += 1
                if result.receipt.decision == "DOWNGRADE" and result.params != exec_params:
                    observed_rewrite = True
            except EvidenceOSDecisionError as exc:
                raise AssertionError(
                    "exec was denied before observing DOWNGRADE rewrite; threshold drift or downgrade regression"
                ) from exc

        assert len(_ForwardProxy.request_ids) >= requests_made
        assert all(request_id for request_id in _ForwardProxy.request_ids)
        assert observed_rewrite, "expected at least one DOWNGRADE rewrite from daemon preflight"
    finally:
        server.shutdown()
        thread.join(timeout=2)
