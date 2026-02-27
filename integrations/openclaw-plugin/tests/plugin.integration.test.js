import test from "node:test";
import assert from "node:assert/strict";
import http from "node:http";

import { createEvidenceGuardPlugin } from "../dist/index.js";

test("integration: plugin sends required headers and applies policy response contract", async () => {
  let observedRequestId;
  let observedAuth;
  let observedBody;

  const server = http.createServer((req, res) => {
    if (req.method !== "POST" || req.url !== "/v1/preflight_tool_call") {
      res.statusCode = 404;
      res.end();
      return;
    }

    observedRequestId = req.headers["x-request-id"];
    observedAuth = req.headers.authorization;

    const chunks = [];
    req.on("data", (chunk) => chunks.push(chunk));
    req.on("end", () => {
      const body = JSON.parse(Buffer.concat(chunks).toString("utf8"));
      observedBody = body;
      const response = body.toolName === "safe.tool"
        ? {
            decision: "DOWNGRADE",
            reason_code: "PolicyDowngrade",
            reason_detail: "tool args sanitized",
            rewritten_params: { mode: "safe", scrubbed: true },
            budget_delta: { spent: 2, remaining: 98 },
          }
        : { decision: "DENY", reasonCode: "PolicyDeny", reasonDetail: "forbidden tool" };
      res.setHeader("content-type", "application/json");
      res.end(JSON.stringify(response));
    });
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", () => resolve()));
  const address = server.address();
  if (!address || typeof address === "string") {
    server.close();
    throw new Error("failed to bind test server");
  }

  const plugin = createEvidenceGuardPlugin({
    evidenceUrl: `http://127.0.0.1:${address.port}`,
    bearerToken: "test-token",
    failClosedRisk: "all",
  });

  try {
    const downgraded = await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: { a: 1 } });
    assert.deepEqual(downgraded.params, { mode: "safe", scrubbed: true });
    assert.ok(!("block" in downgraded));

    assert.ok(observedRequestId, "expected X-Request-Id header to be set");
    assert.match(observedRequestId ?? "", /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    assert.equal(observedAuth, "Bearer test-token");
    assert.equal(typeof observedBody.sessionId, "string");
    assert.ok(observedBody.sessionId.length > 0);
    assert.equal(observedBody.agentId, "openclaw");

    const denied = await plugin.hooks.before_tool_call({ toolName: "danger.tool", params: { a: 1 } });
    assert.equal(denied.block, true);
    assert.match(denied.blockReason ?? "", /PolicyDeny/);
  } finally {
    await new Promise((resolve, reject) => {
      server.close((err) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }
});
