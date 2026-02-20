import json
from urllib import error as urllib_error

import pytest

from langchain_evidenceos import ToolException
from langchain_evidenceos.guard import (
    EvidenceOSGuardCallbackHandler,
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
    with pytest.raises(ToolException):
        _handler(events).guard_tool_call(tool_name="exec", tool_input={"cmd": "rm -rf /"})
    assert events[-1]["decision"] == "DENY"
    assert events[-1]["blocked"] is True


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


def test_fails_closed_on_network_error_for_high_risk_tools(monkeypatch):
    events = []
    handler = _handler(events)

    def boom(req, timeout):
        raise urllib_error.URLError("timed out")

    monkeypatch.setattr("langchain_evidenceos.guard.urllib_request.urlopen", boom)

    with pytest.raises(ToolException):
        handler.guard_tool_call(tool_name="exec", tool_input={"cmd": "whoami"})

    assert events[-1]["reasonCode"] == "EvidenceUnavailable"
    assert events[-1]["blocked"] is True


def test_deterministic_params_hash():
    h1 = deterministic_params_hash({"b": 2, "a": [1, {"z": 3}]})
    h2 = deterministic_params_hash({"a": [1, {"z": 3}], "b": 2})
    assert h1 == h2
