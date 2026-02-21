# langchain-evidenceos

`langchain-evidenceos` is a supported Python integration for enforcing EvidenceOS preflight policy around LangChain-style tool execution.

## What this adapter provides

- `EvidenceOSGuardCallbackHandler` for callback-based preflight gating.
- `EvidenceOSRunnableAdapter` for Runnable-style tool wrapping.
- Policy receipts attached to every tool execution result.
- Configurable timeout + retries for preflight calls.

## EvidenceOS compatibility

This package targets the EvidenceOS **CODEX-E7** preflight endpoint:

- **Method + path**: `POST /v1/preflight_tool_call`
- **Request JSON**: `{ "toolName", "params", "sessionId", "agentId" }`
- **Response fields used**:
  - `decision`: `ALLOW | DENY | REQUIRE_HUMAN | DOWNGRADE`
  - `reasonCode`
  - optional `reasonDetail`
  - optional `rewrittenParams`
  - optional `budgetDelta`

## Install

```bash
pip install -e integrations/langchain-wrapper
```

## Configuration

Environment variables:

- `EVIDENCEOS_URL` (required unless passed directly)
- `EVIDENCEOS_TOKEN` (optional Bearer token)
- `EVIDENCEOS_TIMEOUT_MS` (optional, default `120`)
- `EVIDENCEOS_MAX_RETRIES` (optional, default `2`)
- `EVIDENCEOS_RETRY_BACKOFF_MS` (optional, default `25`)

## Usage (Runnable adapter)

```python
from langchain_evidenceos import EvidenceOSGuardCallbackHandler, EvidenceOSRunnableAdapter

guard = EvidenceOSGuardCallbackHandler(
    evidenceos_url="http://127.0.0.1:50051",
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

Behavior:

- `DENY` and `REQUIRE_HUMAN` raise `ToolException` and block execution.
- `DOWNGRADE` with `rewrittenParams` rewrites tool input before tool execution.
- Network/preflight errors fail closed for high-risk tools (`exec`, `shell.exec`, `fs.write`, `fs.delete_tree`, `email.send`, `payment.charge`) to match OpenClaw behavior.

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
