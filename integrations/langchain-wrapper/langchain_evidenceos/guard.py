from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Callable, Mapping, MutableMapping
from urllib import error as urllib_error
from urllib import request as urllib_request

if importlib.util.find_spec("langchain_core"):
    from langchain_core.callbacks import BaseCallbackHandler
    from langchain_core.tools import ToolException
else:
    class BaseCallbackHandler:  # type: ignore[no-redef]
        pass

    class ToolException(Exception):  # type: ignore[no-redef]
        pass

Decision = str

DEFAULT_TIMEOUT_MS = 120
DEFAULT_HIGH_RISK_TOOLS = {
    "exec",
    "shell.exec",
    "fs.write",
    "fs.delete_tree",
    "email.send",
    "payment.charge",
}


@dataclass(frozen=True)
class EvidenceOSConfig:
    evidenceos_url: str
    token: str | None = None
    timeout_ms: int = DEFAULT_TIMEOUT_MS

    @classmethod
    def from_env(cls) -> "EvidenceOSConfig":
        url = os.getenv("EVIDENCEOS_URL")
        if not url:
            raise ValueError("EVIDENCEOS_URL is required")
        token = os.getenv("EVIDENCEOS_TOKEN") or None
        timeout_ms = int(os.getenv("EVIDENCEOS_TIMEOUT_MS", str(DEFAULT_TIMEOUT_MS)))
        return cls(evidenceos_url=url.rstrip("/"), token=token, timeout_ms=timeout_ms)


def _stable_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def deterministic_params_hash(params: Mapping[str, Any]) -> str:
    digest = hashlib.sha256(_stable_json(params).encode("utf-8")).hexdigest()
    return f"sha256:{digest}"


class EvidenceOSGuardCallbackHandler(BaseCallbackHandler):
    """LangChain callback that gates tool calls via EvidenceOS CODEX-E7 preflight."""

    def __init__(
        self,
        *,
        evidenceos_url: str | None = None,
        token: str | None = None,
        timeout_ms: int = DEFAULT_TIMEOUT_MS,
        session_id: str | None = None,
        agent_id: str | None = None,
        high_risk_tools: set[str] | None = None,
        fail_closed_risk: str = "high-only",
        audit_logger: Callable[[dict[str, Any]], None] | None = None,
    ) -> None:
        cfg = EvidenceOSConfig.from_env() if evidenceos_url is None else EvidenceOSConfig(
            evidenceos_url=evidenceos_url.rstrip("/"),
            token=token,
            timeout_ms=timeout_ms,
        )
        self.evidenceos_url = cfg.evidenceos_url
        self.token = cfg.token
        self.timeout_ms = cfg.timeout_ms
        self.session_id = session_id
        self.agent_id = agent_id
        self.high_risk_tools = high_risk_tools or set(DEFAULT_HIGH_RISK_TOOLS)
        self.fail_closed_risk = fail_closed_risk
        self.audit_logger = audit_logger or self._default_audit_logger
        self.last_rewritten_params: dict[str, Any] | None = None

    @property
    def always_verbose(self) -> bool:
        return True

    def on_tool_start(
        self,
        serialized: dict[str, Any],
        input_str: str,
        *,
        run_id: Any,
        parent_run_id: Any | None = None,
        tags: list[str] | None = None,
        metadata: dict[str, Any] | None = None,
        inputs: dict[str, Any] | None = None,
        **kwargs: Any,
    ) -> Any:
        tool_name = serialized.get("name", "unknown_tool")
        if inputs is not None:
            tool_input: dict[str, Any] | str = inputs
        else:
            try:
                tool_input = json.loads(input_str)
            except json.JSONDecodeError:
                tool_input = {"input": input_str}

        rewritten = self.guard_tool_call(tool_name=tool_name, tool_input=tool_input)
        self.last_rewritten_params = rewritten

        if inputs is not None and isinstance(inputs, MutableMapping):
            inputs.clear()
            inputs.update(rewritten)
        return rewritten

    def guard_tool_call(self, *, tool_name: str, tool_input: Mapping[str, Any] | str) -> dict[str, Any]:
        params = self._normalize_params(tool_input)
        payload = {
            "toolName": tool_name,
            "params": params,
            "sessionId": self.session_id,
            "agentId": self.agent_id,
        }

        try:
            response = self._preflight(payload)
        except (TimeoutError, ConnectionError, urllib_error.HTTPError, urllib_error.URLError, ValueError) as exc:
            if self._should_fail_closed(tool_name):
                self._emit_audit(tool_name=tool_name, params=params, decision="DENY", reason_code="EvidenceUnavailable", reason_detail=str(exc), blocked=True)
                raise ToolException(f"EvidenceOS preflight unavailable: {exc}") from exc

            self._emit_audit(tool_name=tool_name, params=params, decision="ALLOW", reason_code="EvidenceUnavailable", reason_detail=str(exc), blocked=False)
            return dict(params)

        decision = str(response.get("decision", "DENY"))
        reason_code = str(response.get("reasonCode", "UnknownDecision"))
        reason_detail = response.get("reasonDetail")
        rewritten = response.get("rewrittenParams")
        blocked = decision in {"DENY", "REQUIRE_HUMAN"}

        next_params = dict(params)
        if decision == "DOWNGRADE" and isinstance(rewritten, dict):
            next_params = rewritten

        self._emit_audit(
            tool_name=tool_name,
            params=params,
            decision=decision,
            reason_code=reason_code,
            reason_detail=reason_detail,
            blocked=blocked,
            budget_delta=response.get("budgetDelta"),
        )

        if blocked:
            raise ToolException(f"{reason_code}:{reason_detail or 'n/a'}")
        return next_params

    def _preflight(self, payload: Mapping[str, Any]) -> dict[str, Any]:
        headers = {"content-type": "application/json"}
        if self.token:
            headers["authorization"] = f"Bearer {self.token}"

        req = urllib_request.Request(
            url=f"{self.evidenceos_url}/v1/preflight_tool_call",
            data=json.dumps(payload).encode("utf-8"),
            headers=headers,
            method="POST",
        )
        with urllib_request.urlopen(req, timeout=self.timeout_ms / 1000) as resp:
            body = json.loads(resp.read().decode("utf-8"))
        if not isinstance(body, dict):
            raise ValueError("Invalid JSON body from EvidenceOS")
        return body

    def _should_fail_closed(self, tool_name: str) -> bool:
        return self.fail_closed_risk == "all" or tool_name in self.high_risk_tools

    def _emit_audit(self, *, tool_name: str, params: Mapping[str, Any], decision: Decision, reason_code: str, reason_detail: str | None, blocked: bool, budget_delta: dict[str, Any] | None = None) -> None:
        event: dict[str, Any] = {
            "type": "evidenceos.audit",
            "ts": datetime.now(tz=timezone.utc).isoformat(),
            "toolName": tool_name,
            "paramsHash": deterministic_params_hash(params),
            "decision": decision,
            "reasonCode": reason_code,
            "blocked": blocked,
        }
        if reason_detail is not None:
            event["reasonDetail"] = reason_detail
        if budget_delta is not None:
            event["budgetDelta"] = budget_delta
        self.audit_logger(event)

    @staticmethod
    def _normalize_params(tool_input: Mapping[str, Any] | str) -> dict[str, Any]:
        if isinstance(tool_input, Mapping):
            return dict(tool_input)
        return {"input": tool_input}

    @staticmethod
    def _default_audit_logger(event: dict[str, Any]) -> None:
        print(json.dumps(event, separators=(",", ":"), sort_keys=True))
