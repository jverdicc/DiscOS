# langchain-evidenceos

`langchain-evidenceos` is an installable Python package that enforces EvidenceOS preflight policy before a LangChain tool call executes.

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

## Usage

```python
from langchain_evidenceos import EvidenceOSGuardCallbackHandler

guard = EvidenceOSGuardCallbackHandler(
    evidenceos_url="http://127.0.0.1:50051",
    session_id="session-123",
    agent_id="agent-abc",
)

# Framework calls on_tool_start internally, or call directly:
safe_params = guard.guard_tool_call(tool_name="search.web", tool_input={"query": "status page"})
```

Behavior:

- `DENY` and `REQUIRE_HUMAN` raise `ToolException` and block execution.
- `DOWNGRADE` with `rewrittenParams` rewrites tool input.
- Network/preflight errors fail closed for high-risk tools (`exec`, `shell.exec`, `fs.write`, `fs.delete_tree`, `email.send`, `payment.charge`) to match OpenClaw behavior.

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
