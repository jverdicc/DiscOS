from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Callable, Literal, Mapping, MutableMapping
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

Decision = Literal["ALLOW", "DENY", "REQUIRE_HUMAN", "DOWNGRADE"]

DEFAULT_TIMEOUT_MS = 120
DEFAULT_MAX_RETRIES = 2
DEFAULT_RETRY_BACKOFF_MS = 25
DEFAULT_HIGH_RISK_TOOLS = {
    "exec",
    "shell.exec",
    "fs.write",
    "fs.delete_tree",
    "email.send",
    "payment.charge",
}


@dataclass(frozen=True)
class PolicyReceipt:
    decision: Decision
    reason_code: str
    reason_detail: str | None
    budget_delta: dict[str, Any] | None = None


@dataclass(frozen=True)
class PreflightResult:
    params: dict[str, Any]
    blocked: bool
    receipt: PolicyReceipt


@dataclass(frozen=True)
class EvidenceOSConfig:
    evidenceos_url: str
    token: str | None = None
    timeout_ms: int = DEFAULT_TIMEOUT_MS
    max_retries: int = DEFAULT_MAX_RETRIES
    retry_backoff_ms: int = DEFAULT_RETRY_BACKOFF_MS

    @classmethod
    def from_env(cls) -> "EvidenceOSConfig":
        url = os.getenv("EVIDENCEOS_URL")
        if not url:
            raise ValueError("EVIDENCEOS_URL is required")
        token = os.getenv("EVIDENCEOS_TOKEN") or None
        timeout_ms = int(os.getenv("EVIDENCEOS_TIMEOUT_MS", str(DEFAULT_TIMEOUT_MS)))
        max_retries = int(os.getenv("EVIDENCEOS_MAX_RETRIES", str(DEFAULT_MAX_RETRIES)))
        retry_backoff_ms = int(
            os.getenv("EVIDENCEOS_RETRY_BACKOFF_MS", str(DEFAULT_RETRY_BACKOFF_MS))
        )
        return cls(
            evidenceos_url=url.rstrip("/"),
            token=token,
            timeout_ms=timeout_ms,
            max_retries=max_retries,
            retry_backoff_ms=retry_backoff_ms,
        )


@dataclass(frozen=True)
class EvidenceOSDecision:
    decision: Decision
    reason_code: str
    reason_detail: str | None
    rewritten_params: dict[str, Any] | None
    budget_delta: dict[str, Any] | None


class EvidenceOSToolException(ToolException):
    """Base exception for typed policy failures."""


class EvidenceOSUnavailableError(EvidenceOSToolException):
    """EvidenceOS preflight could not be reached or produced invalid output."""


class EvidenceOSDecisionError(EvidenceOSToolException):
    """EvidenceOS returned a blocking policy decision."""

    def __init__(self, message: str, *, receipt: PolicyReceipt) -> None:
        super().__init__(message)
        self.receipt = receipt


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
        max_retries: int = DEFAULT_MAX_RETRIES,
        retry_backoff_ms: int = DEFAULT_RETRY_BACKOFF_MS,
        session_id: str | None = None,
        agent_id: str | None = None,
        high_risk_tools: set[str] | None = None,
        fail_closed_risk: str = "all",
        audit_logger: Callable[[dict[str, Any]], None] | None = None,
        sleeper: Callable[[float], None] | None = None,
    ) -> None:
        cfg = (
            EvidenceOSConfig.from_env()
            if evidenceos_url is None
            else EvidenceOSConfig(
                evidenceos_url=evidenceos_url.rstrip("/"),
                token=token,
                timeout_ms=timeout_ms,
                max_retries=max_retries,
                retry_backoff_ms=retry_backoff_ms,
            )
        )
        self.evidenceos_url = cfg.evidenceos_url
        self.token = cfg.token
        self.timeout_ms = cfg.timeout_ms
        self.max_retries = cfg.max_retries
        self.retry_backoff_ms = cfg.retry_backoff_ms
        self.session_id = session_id
        self.agent_id = agent_id
        self.high_risk_tools = high_risk_tools or set(DEFAULT_HIGH_RISK_TOOLS)
        self.fail_closed_risk = fail_closed_risk
        self.audit_logger = audit_logger or self._default_audit_logger
        self.last_rewritten_params: dict[str, Any] | None = None
        self.last_policy_receipt: PolicyReceipt | None = None
        self._sleep = sleeper or time.sleep

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

        preflight_result = self.preflight_tool_call(tool_name=tool_name, tool_input=tool_input)
        rewritten = preflight_result.params
        self.last_rewritten_params = rewritten
        self.last_policy_receipt = preflight_result.receipt

        if inputs is not None and isinstance(inputs, MutableMapping):
            inputs.clear()
            inputs.update(rewritten)
        return rewritten

    def preflight_tool_call(
        self, *, tool_name: str, tool_input: Mapping[str, Any] | str
    ) -> PreflightResult:
        params = self._normalize_params(tool_input)
        payload = {
            "toolName": tool_name,
            "params": params,
            "sessionId": self.session_id,
            "agentId": self.agent_id,
        }

        try:
            response = self._preflight(payload)
        except (
            TimeoutError,
            ConnectionError,
            urllib_error.HTTPError,
            urllib_error.URLError,
            ValueError,
        ) as exc:
            receipt = PolicyReceipt(
                decision="DENY" if self._should_fail_closed(tool_name) else "ALLOW",
                reason_code="EvidenceUnavailable",
                reason_detail=str(exc),
            )
            self._emit_audit(
                tool_name=tool_name,
                params=params,
                decision=receipt.decision,
                reason_code=receipt.reason_code,
                reason_detail=receipt.reason_detail,
                blocked=receipt.decision == "DENY",
            )
            if receipt.decision == "DENY":
                raise EvidenceOSUnavailableError(f"EvidenceOS preflight unavailable: {exc}") from exc
            return PreflightResult(params=dict(params), blocked=False, receipt=receipt)

        parsed = self._parse_decision(response)
        blocked = parsed.decision in {"DENY", "REQUIRE_HUMAN"}

        next_params = dict(params)
        if parsed.decision == "DOWNGRADE" and isinstance(parsed.rewritten_params, dict):
            next_params = parsed.rewritten_params

        receipt = PolicyReceipt(
            decision=parsed.decision,
            reason_code=parsed.reason_code,
            reason_detail=parsed.reason_detail,
            budget_delta=parsed.budget_delta,
        )
        self._emit_audit(
            tool_name=tool_name,
            params=params,
            decision=parsed.decision,
            reason_code=parsed.reason_code,
            reason_detail=parsed.reason_detail,
            blocked=blocked,
            budget_delta=parsed.budget_delta,
        )

        if blocked:
            raise EvidenceOSDecisionError(
                f"{parsed.reason_code}:{parsed.reason_detail or 'n/a'}",
                receipt=receipt,
            )
        return PreflightResult(params=next_params, blocked=False, receipt=receipt)

    def guard_tool_call(
        self, *, tool_name: str, tool_input: Mapping[str, Any] | str
    ) -> dict[str, Any]:
        result = self.preflight_tool_call(tool_name=tool_name, tool_input=tool_input)
        self.last_policy_receipt = result.receipt
        return result.params

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

        attempts = self.max_retries + 1
        last_exc: Exception | None = None
        for attempt in range(attempts):
            try:
                with urllib_request.urlopen(req, timeout=self.timeout_ms / 1000) as resp:
                    body = json.loads(resp.read().decode("utf-8"))
                if not isinstance(body, dict):
                    raise ValueError("Invalid JSON body from EvidenceOS")
                return body
            except (urllib_error.HTTPError, urllib_error.URLError, TimeoutError, ValueError) as exc:
                retryable_http = isinstance(exc, urllib_error.HTTPError) and exc.code >= 500
                retryable = retryable_http or isinstance(exc, (urllib_error.URLError, TimeoutError))
                if attempt >= self.max_retries or not retryable:
                    last_exc = exc
                    break
                backoff_s = ((2**attempt) * self.retry_backoff_ms) / 1000
                self._sleep(backoff_s)

        if last_exc is not None:
            raise last_exc
        raise ValueError("EvidenceOS preflight failed without a captured exception")

    def _should_fail_closed(self, tool_name: str) -> bool:
        return self.fail_closed_risk == "all" or tool_name in self.high_risk_tools

    @staticmethod
    def _parse_decision(response: Mapping[str, Any]) -> EvidenceOSDecision:
        if not isinstance(response, Mapping):
            raise ValueError("Invalid JSON body from EvidenceOS")

        raw_decision = str(response.get("decision", "DENY")).upper()
        decision: Decision
        if raw_decision == "DEFER":
            decision = "REQUIRE_HUMAN"
        elif raw_decision in {"ALLOW", "DENY", "REQUIRE_HUMAN", "DOWNGRADE"}:
            decision = raw_decision
        else:
            decision = "DENY"

        reason_detail = response.get("reasonDetail")
        return EvidenceOSDecision(
            decision=decision,
            reason_code=str(response.get("reasonCode", "UnknownDecision")),
            reason_detail=str(reason_detail) if reason_detail is not None else None,
            rewritten_params=response.get("rewrittenParams")
            if isinstance(response.get("rewrittenParams"), dict)
            else None,
            budget_delta=response.get("budgetDelta")
            if isinstance(response.get("budgetDelta"), dict)
            else None,
        )

    def _emit_audit(
        self,
        *,
        tool_name: str,
        params: Mapping[str, Any],
        decision: Decision,
        reason_code: str,
        reason_detail: str | None,
        blocked: bool,
        budget_delta: dict[str, Any] | None = None,
    ) -> None:
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
