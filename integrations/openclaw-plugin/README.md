<!-- Copyright (c) 2026 Joseph Verdicchio and DiscOS Contributors -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# @evidenceos/openclaw-guard

OpenClaw plugin that hard-gates tool execution through EvidenceOS preflight policy checks.

## Why

This plugin is designed to be a drop-in safety middleware for OpenClaw:

- Enforces policy in `before_tool_call` (execution path hard gate).
- Blocks with machine-readable reasons (`{ block: true, blockReason }`).
- Applies parameter rewrites when EvidenceOS returns downgrade/sanitization.
- Emits deterministic one-line JSON audit events for every decision.

## Security defaults

- **Fail-closed** for high-risk tools (`exec`, `fs.write`, `email.send`, etc.).
- **Strict timeout** (default 120ms) for policy RPC.
- **Circuit breaker** to avoid hanging OpenClaw loops when EvidenceOS is unstable.
- **High priority hook** (`priority: 1000`) to reduce ordering conflicts.
- **No `block: false` emission** to avoid accidental merge semantics override.

## Developer requirements

- Node.js **>= 22.6.0** is required to run tests because the test workflow relies on Node's TypeScript type-stripping support (`--experimental-strip-types`).

## Install

```bash
openclaw plugins install @evidenceos/openclaw-guard
```

## Configure

```ts
import { createEvidenceGuardPlugin } from "@evidenceos/openclaw-guard";

export default createEvidenceGuardPlugin({
  evidenceUrl: "http://127.0.0.1:8787",
  bearerToken: process.env.EVIDENCEOS_TOKEN, // optional
  timeoutMs: 120,
  failClosedRisk: "high-only",
  // Optional stable identifiers (recommended for long-running budgets):
  sessionId: process.env.EVIDENCEOS_SESSION_ID,
  agentId: process.env.EVIDENCEOS_AGENT_ID ?? "openclaw",
});
```

Minimal working config:

```ts
createEvidenceGuardPlugin({
  evidenceUrl: "http://127.0.0.1:8787",
});
```


Session/agent identity defaults:

- `sessionId` is always sent; if OpenClaw does not provide one, the plugin uses `sessionId` config, then `EVIDENCEOS_SESSION_ID`, then an auto-generated `openclaw-<uuid>`.
- `agentId` is always sent; defaults to `agentId` config, then `EVIDENCEOS_AGENT_ID`, then `"openclaw"`.
- Pass a stable `sessionId` when you want EvidenceOS operation budgets to span process restarts.
- EvidenceOS may require `sessionId` for high-risk tools and deny requests when it is missing.


## Wire contract (`POST /v1/preflight_tool_call`)

This plugin sends:

- Header `Content-Type: application/json`
- Header `X-Request-Id: <uuid-v4>` (**required by EvidenceOS preflight policy**)
- Header `Authorization: Bearer <token>` (optional; from `bearerToken` or `EVIDENCEOS_TOKEN`)

Payload:

```json
{
  "toolName": "fs.delete_tree",
  "params": { "path": "/tmp/demo" },
  "sessionId": "session-123",
  "agentId": "agent-abc"
}
```

Response fields accepted (camelCase and snake_case aliases):

- `decision`
- `reasonCode` / `reason_code`
- `reasonDetail` / `reason_detail`
- `rewrittenParams` / `rewritten_params`
- `budgetDelta` / `budget_delta`

Example response:

```json
{
  "decision": "DOWNGRADE",
  "reasonCode": "PolicyDowngrade",
  "reasonDetail": "Delete path rewritten to a safe sandbox",
  "rewrittenParams": { "path": "/tmp/sandbox/demo" },
  "budgetDelta": { "spent": 1, "remaining": 99 }
}
```

## Demo sequence

1. Start EvidenceOS (`evidenceosd start` or docker compose).
2. Start OpenClaw gateway.
3. Trigger a high-risk tool in a sandbox (for example `fs.delete_tree` against a temp directory).
4. Verify OpenClaw receives a clean refusal with a machine-readable reason.
5. Verify audit logs include `toolName`, `paramsHash`, `decision`, `reasonCode`, and budget state.
