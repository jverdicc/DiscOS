# langchain-evidenceos

`langchain-evidenceos` is a supported Python integration for enforcing EvidenceOS preflight policy around LangChain-style tool execution.

## What this adapter provides

- `EvidenceOSGuardCallbackHandler` for callback-based preflight gating.
- `EvidenceOSRunnableAdapter` for Runnable-style tool wrapping.
- Structured return types (`PolicyReceipt`, `PreflightResult`, `ToolExecutionResult`).
- Typed exceptions (`EvidenceOSUnavailableError`, `EvidenceOSDecisionError`).
- Configurable timeout + retries for preflight calls.

## EvidenceOS compatibility

This package targets the EvidenceOS **CODEX-E7** HTTP preflight endpoint:

- **Compatibility tested against EvidenceOS CODEX-E7+**: `91029aa`

- **Method + path**: `POST /v1/preflight_tool_call`
- **Transport**: HTTP/JSON (not gRPC)
- **Typical local URL**: `http://127.0.0.1:8787`
- **Request JSON**: `{ "toolName", "params", "sessionId", "agentId" }`
- **Required request header**: `X-Request-Id: <uuid-v4>` (generated for every preflight call)
- **Response fields used**:
  - `decision`: `ALLOW | DENY | REQUIRE_HUMAN | DOWNGRADE`
  - `reasonCode` (legacy fallback: `reason_code`)
  - optional `reasonDetail` (legacy fallback: `reason_detail`)
  - optional `rewrittenParams` (legacy fallback: `rewritten_params`)
  - optional `budgetDelta` (legacy fallback: `budget_delta`)

## Compatibility matrix

- **LangChain support:** `langchain-core>=0.2.0` (validated against the current wrapper test suite).
- **Execution modes:** synchronous `invoke`/callback flow is supported; async (`ainvoke`/async callbacks) is not implemented yet.
- **Auth options:** Bearer token (`Authorization: Bearer <token>`) is supported; HMAC signing is not supported in this wrapper.

## Install

```bash
pip install -e integrations/langchain-wrapper
```

## Configuration

Environment variables:

- `EVIDENCEOS_URL` (required unless passed directly)
- `EVIDENCEOS_TOKEN` (optional Bearer token for preflight auth)
- `EVIDENCEOS_TIMEOUT_MS` (optional, default `120`)
- `EVIDENCEOS_MAX_RETRIES` (optional, default `2`)
- `EVIDENCEOS_RETRY_BACKOFF_MS` (optional, default `25`)

Authentication behavior:

- When `EVIDENCEOS_TOKEN` or `token=...` is set, the client sends `Authorization: Bearer <token>`.
- `401`/`403` responses are treated as non-retryable auth failures.
- In the default safe profile (`fail_closed_risk="all"`), auth failures deny tool execution.
- If `fail_closed_risk="high-only"`, non-high-risk tools can fail open when preflight is unavailable.

Request-id behavior:

- Every preflight request includes `X-Request-Id` and uses a newly generated UUIDv4.
- If the daemon rejects a request (for example, missing/invalid request id), the wrapper surfaces an `EvidenceOSDecisionError` with daemon-provided `reasonCode` / `reasonDetail`.

## Usage (Runnable adapter)

```python
from langchain_evidenceos import EvidenceOSGuardCallbackHandler, EvidenceOSRunnableAdapter

guard = EvidenceOSGuardCallbackHandler(
    evidenceos_url="http://127.0.0.1:8787",
    session_id="session-123",
    agent_id="agent-abc",
)

adapter = EvidenceOSRunnableAdapter(
    tool_name="search.web",
    tool_func=lambda params: {"answer": f"result for {params['query']}"},
    guard=guard,
)

result = adapter.invoke({"query": "status page"})
print(result.output)
print(result.policy_receipt)
```

Decision handling:

- `ALLOW`: tool executes with original params.
- `DOWNGRADE`: tool executes with `rewrittenParams`.
- `DENY` and `REQUIRE_HUMAN`: raises `EvidenceOSDecisionError`.
- `DEFER` responses are normalized to `REQUIRE_HUMAN`.

Failure handling:

- Timeout/network/5xx failures are retried with exponential backoff.
- 4xx failures are parsed as policy/contract decisions (when daemon returns structured decision JSON).
- Default is fail-closed for all tools (`fail_closed_risk="all"`).

## End-to-end example

```bash
python integrations/langchain-wrapper/examples/e2e_preflight_adapter.py
```

## Audit format

Every decision emits one JSON line compatible with OpenClaw audit events:

```json
{
  "type": "evidenceos.audit",
  "ts": "2026-01-01T00:00:00.000000+00:00",
  "toolName": "exec",
  "paramsHash": "sha256:...",
  "decision": "DENY",
  "reasonCode": "PolicyDeny",
  "reasonDetail": "...",
  "blocked": true,
  "budgetDelta": {"spent": 1, "remaining": 9}
}
```

## Tests

```bash
pip install -e integrations/langchain-wrapper[test]
pytest integrations/langchain-wrapper/tests -q
```
