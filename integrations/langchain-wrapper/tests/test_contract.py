import json
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer

import pytest

from langchain_evidenceos import EvidenceOSDecisionError, EvidenceOSGuardCallbackHandler


class _EvidenceOSContractHandler(BaseHTTPRequestHandler):
    requests = []

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("content-length", "0"))).decode("utf-8")
        payload = json.loads(body) if body else {}
        self.requests.append(
            {
                "headers": {k.lower(): v for k, v in self.headers.items()},
                "path": self.path,
                "body": payload,
            }
        )

        if self.path != "/v1/preflight_tool_call":
            self.send_response(404)
            self.send_header("content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"decision": "DENY", "reasonCode": "NotFound"}).encode("utf-8"))
            return

        if not self.headers.get("X-Request-Id"):
            self.send_response(400)
            self.send_header("content-type", "application/json")
            self.end_headers()
            self.wfile.write(
                json.dumps(
                    {
                        "decision": "DENY",
                        "reasonCode": "MissingRequestId",
                        "reasonDetail": "X-Request-Id header is required",
                    }
                ).encode("utf-8")
            )
            return

        tool_name = payload.get("toolName")
        if tool_name == "allow.tool":
            body = {"decision": "ALLOW", "reasonCode": "PolicyAllow", "reasonDetail": "ok"}
        elif tool_name == "deny.tool":
            body = {
                "decision": "DENY",
                "reasonCode": "PolicyDeny",
                "reasonDetail": "blocked by policy",
            }
        else:
            body = {
                "decision": "DOWNGRADE",
                "reasonCode": "PolicyDowngrade",
                "reasonDetail": "sanitized",
                "rewrittenParams": {"query": "public"},
                "budgetDelta": {"spent": 1, "remaining": 9},
            }

        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(body).encode("utf-8"))

    def log_message(self, _fmt, *_args):
        return


@pytest.fixture
def contract_server():
    _EvidenceOSContractHandler.requests = []
    server = HTTPServer(("127.0.0.1", 0), _EvidenceOSContractHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield server
    finally:
        server.shutdown()
        thread.join(timeout=2)


def _guard(port: int) -> EvidenceOSGuardCallbackHandler:
    return EvidenceOSGuardCallbackHandler(
        evidenceos_url=f"http://127.0.0.1:{port}",
        session_id="sess-1",
        agent_id="agent-1",
    )


def test_missing_x_request_id_returns_structured_decision_error(monkeypatch, contract_server):
    monkeypatch.setattr("langchain_evidenceos.guard.uuid.uuid4", lambda: "")

    with pytest.raises(EvidenceOSDecisionError) as excinfo:
        _guard(contract_server.server_port).guard_tool_call(
            tool_name="allow.tool", tool_input={"query": "hello"}
        )

    assert excinfo.value.receipt.reason_code == "MissingRequestId"
    assert "X-Request-Id header is required" in (excinfo.value.receipt.reason_detail or "")


def test_with_request_id_header_supports_allow_deny_and_downgrade(contract_server):
    guard = _guard(contract_server.server_port)

    allow = guard.guard_tool_call(tool_name="allow.tool", tool_input={"query": "ok"})
    assert allow == {"query": "ok"}

    with pytest.raises(EvidenceOSDecisionError) as excinfo:
        guard.guard_tool_call(tool_name="deny.tool", tool_input={"query": "nope"})
    assert excinfo.value.receipt.reason_code == "PolicyDeny"

    downgraded = guard.guard_tool_call(tool_name="downgrade.tool", tool_input={"query": "secret"})
    assert downgraded == {"query": "public"}

    for request in _EvidenceOSContractHandler.requests:
        assert request["headers"]["x-request-id"]


def test_response_schema_camel_case_is_consumed(contract_server):
    guard = _guard(contract_server.server_port)
    with pytest.raises(EvidenceOSDecisionError) as excinfo:
        guard.guard_tool_call(tool_name="deny.tool", tool_input={"query": "hello"})

    assert excinfo.value.receipt.reason_code == "PolicyDeny"
    assert excinfo.value.receipt.reason_detail == "blocked by policy"
