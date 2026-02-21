import json
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib import error as urllib_error

import pytest

from langchain_evidenceos import (
    EvidenceOSDecisionError,
    EvidenceOSGuardCallbackHandler,
    EvidenceOSRunnableAdapter,
    EvidenceOSUnavailableError,
    ToolException,
    deterministic_params_hash,
)


class _FakeResponse:
    def __init__(self, payload):
        self._payload = payload

    def read(self):
        return json.dumps(self._payload).encode("utf-8")

    def __enter__(self):
        return self

    def __exit__(self, *args):
        return False


class _SequenceHandler(BaseHTTPRequestHandler):
    responses = []
    requests = []

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("content-length", "0"))).decode("utf-8")
        self.requests.append(
            {
                "path": self.path,
                "headers": {k.lower(): v for k, v in self.headers.items()},
                "body": json.loads(body) if body else {},
            }
        )

        if self.path != "/v1/preflight_tool_call":
            self.send_response(404)
            self.end_headers()
            return
        payload = self.responses.pop(0)
        self.send_response(payload["status"])
        self.send_header("content-type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(payload["body"]).encode("utf-8"))

    def log_message(self, format, *args):
        return


@pytest.fixture
def preflight_server():
    _SequenceHandler.responses = []
    _SequenceHandler.requests = []
    server = HTTPServer(("127.0.0.1", 0), _SequenceHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield server
    finally:
        server.shutdown()
        thread.join(timeout=2)


def _handler(audit_events):
    return EvidenceOSGuardCallbackHandler(
        evidenceos_url="http://evidenceos.test",
        session_id="sess-1",
        agent_id="agent-1",
        audit_logger=audit_events.append,
    )


def test_allows_when_preflight_allows(monkeypatch):
    events = []
    monkeypatch.setattr(
        "langchain_evidenceos.guard.urllib_request.urlopen",
        lambda req, timeout: _FakeResponse({"decision": "ALLOW", "reasonCode": "PolicyAllow"}),
    )
    out = _handler(events).guard_tool_call(tool_name="read.docs", tool_input={"q": "x"})
    assert out == {"q": "x"}
    assert events[-1]["decision"] == "ALLOW"
    assert events[-1]["blocked"] is False


def test_blocks_when_preflight_denies(monkeypatch):
    events = []
    monkeypatch.setattr(
        "langchain_evidenceos.guard.urllib_request.urlopen",
        lambda req, timeout: _FakeResponse(
            {"decision": "DENY", "reasonCode": "PolicyDeny", "reasonDetail": "blocked"}
        ),
    )
    with pytest.raises(EvidenceOSDecisionError):
        _handler(events).guard_tool_call(tool_name="exec", tool_input={"cmd": "rm -rf /"})
    assert events[-1]["decision"] == "DENY"
    assert events[-1]["blocked"] is True


def test_blocks_when_preflight_requires_human(monkeypatch):
    events = []
    monkeypatch.setattr(
        "langchain_evidenceos.guard.urllib_request.urlopen",
        lambda req, timeout: _FakeResponse(
            {
                "decision": "REQUIRE_HUMAN",
                "reasonCode": "Escalate",
                "reasonDetail": "operator approval required",
            }
        ),
    )
    with pytest.raises(EvidenceOSDecisionError):
        _handler(events).guard_tool_call(tool_name="read.docs", tool_input={"q": "x"})
    assert events[-1]["decision"] == "REQUIRE_HUMAN"


def test_maps_defer_to_require_human(monkeypatch):
    events = []
    monkeypatch.setattr(
        "langchain_evidenceos.guard.urllib_request.urlopen",
        lambda req, timeout: _FakeResponse(
            {"decision": "DEFER", "reasonCode": "SJ_DEFER", "reasonDetail": "defer decision"}
        ),
    )
    with pytest.raises(EvidenceOSDecisionError):
        _handler(events).guard_tool_call(tool_name="read.docs", tool_input={"q": "x"})
    assert events[-1]["decision"] == "REQUIRE_HUMAN"
    assert events[-1]["reasonCode"] == "SJ_DEFER"


def test_rewrites_params_on_downgrade(monkeypatch):
    events = []
    monkeypatch.setattr(
        "langchain_evidenceos.guard.urllib_request.urlopen",
        lambda req, timeout: _FakeResponse(
            {
                "decision": "DOWNGRADE",
                "reasonCode": "Sanitized",
                "rewrittenParams": {"query": "public data"},
            }
        ),
    )
    out = _handler(events).guard_tool_call(
        tool_name="search.web", tool_input={"query": "internal secrets"}
    )
    assert out == {"query": "public data"}
    assert events[-1]["decision"] == "DOWNGRADE"


def test_fails_closed_on_network_error_for_all_tools_by_default(monkeypatch):
    events = []
    handler = _handler(events)

    def boom(req, timeout):
        raise urllib_error.URLError("timed out")

    monkeypatch.setattr("langchain_evidenceos.guard.urllib_request.urlopen", boom)

    with pytest.raises(EvidenceOSUnavailableError):
        handler.guard_tool_call(tool_name="read.docs", tool_input={"q": "whoami"})

    assert events[-1]["reasonCode"] == "EvidenceUnavailable"
    assert events[-1]["blocked"] is True


def test_can_be_configured_fail_open_for_low_risk(monkeypatch):
    events = []

    def boom(req, timeout):
        raise urllib_error.URLError("timed out")

    monkeypatch.setattr("langchain_evidenceos.guard.urllib_request.urlopen", boom)
    handler = EvidenceOSGuardCallbackHandler(
        evidenceos_url="http://evidenceos.test",
        fail_closed_risk="high-only",
        audit_logger=events.append,
    )

    out = handler.guard_tool_call(tool_name="read.docs", tool_input={"q": "x"})
    assert out == {"q": "x"}
    assert events[-1]["decision"] == "ALLOW"


def test_retries_then_succeeds_with_http_server(preflight_server):
    _SequenceHandler.responses = [
        {"status": 503, "body": {"error": "busy"}},
        {"status": 200, "body": {"decision": "ALLOW", "reasonCode": "PolicyAllow"}},
    ]
    events = []
    handler = EvidenceOSGuardCallbackHandler(
        evidenceos_url=f"http://127.0.0.1:{preflight_server.server_port}",
        max_retries=1,
        retry_backoff_ms=1,
        session_id="sess-1",
        agent_id="agent-1",
        audit_logger=events.append,
    )
    out = handler.guard_tool_call(tool_name="read.docs", tool_input={"q": "x"})
    assert out == {"q": "x"}
    assert events[-1]["decision"] == "ALLOW"
    assert len(_SequenceHandler.requests) == 2


def test_timeout_exhaustion_fails_closed(monkeypatch):
    events = []

    def timeout(*_args, **_kwargs):
        raise TimeoutError("socket timeout")

    monkeypatch.setattr("langchain_evidenceos.guard.urllib_request.urlopen", timeout)
    handler = EvidenceOSGuardCallbackHandler(
        evidenceos_url="http://evidenceos.test",
        max_retries=2,
        retry_backoff_ms=1,
        audit_logger=events.append,
    )

    with pytest.raises(EvidenceOSUnavailableError):
        handler.guard_tool_call(tool_name="exec", tool_input={"cmd": "id"})

    assert events[-1]["decision"] == "DENY"
    assert events[-1]["reasonCode"] == "EvidenceUnavailable"


def test_sends_bearer_token_and_payload(preflight_server):
    _SequenceHandler.responses = [
        {"status": 200, "body": {"decision": "ALLOW", "reasonCode": "PolicyAllow"}}
    ]
    handler = EvidenceOSGuardCallbackHandler(
        evidenceos_url=f"http://127.0.0.1:{preflight_server.server_port}",
        token="secret-token",
        session_id="sess-1",
        agent_id="agent-1",
    )

    out = handler.guard_tool_call(tool_name="search.web", tool_input={"query": "status"})
    assert out == {"query": "status"}

    request = _SequenceHandler.requests[-1]
    assert request["headers"]["authorization"] == "Bearer secret-token"
    assert request["body"] == {
        "toolName": "search.web",
        "params": {"query": "status"},
        "sessionId": "sess-1",
        "agentId": "agent-1",
    }


def test_runnable_adapter_attaches_policy_receipt(monkeypatch):
    monkeypatch.setattr(
        "langchain_evidenceos.guard.urllib_request.urlopen",
        lambda req, timeout: _FakeResponse(
            {
                "decision": "ALLOW",
                "reasonCode": "PolicyAllow",
                "budgetDelta": {"spent": 1, "remaining": 9},
            }
        ),
    )

    guard = EvidenceOSGuardCallbackHandler(
        evidenceos_url="http://evidenceos.test",
        session_id="sess",
        agent_id="agent",
    )
    adapter = EvidenceOSRunnableAdapter(
        tool_name="search.web",
        tool_func=lambda params: {"result": params["query"]},
        guard=guard,
    )

    result = adapter.invoke({"query": "status page"})
    assert result.output == {"result": "status page"}
    assert result.policy_receipt.reason_code == "PolicyAllow"
    assert result.policy_receipt.budget_delta == {"spent": 1, "remaining": 9}


def test_deterministic_params_hash():
    h1 = deterministic_params_hash({"b": 2, "a": [1, {"z": 3}]})
    h2 = deterministic_params_hash({"a": [1, {"z": 3}], "b": 2})
    assert h1 == h2
