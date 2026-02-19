import test from "node:test";
import assert from "node:assert/strict";
import http from "node:http";

import { createEvidenceGuardPlugin } from "../src/index.ts";

test("integration: plugin talks to stubbed EvidenceOS policy endpoint", async () => {
  const server = http.createServer((req, res) => {
    if (req.method !== "POST" || req.url !== "/v1/preflight_tool_call") {
      res.statusCode = 404;
      res.end();
      return;
    }

    const chunks: Buffer[] = [];
    req.on("data", (chunk) => chunks.push(chunk));
    req.on("end", () => {
      const body = JSON.parse(Buffer.concat(chunks).toString("utf8"));
      const response = body.toolName === "safe.tool"
        ? { decision: "ALLOW", reasonCode: "PolicyAllow", rewrittenParams: { mode: "safe" } }
        : { decision: "DENY", reasonCode: "PolicyDeny", reasonDetail: "forbidden tool" };
      res.setHeader("content-type", "application/json");
      res.end(JSON.stringify(response));
    });
  });

  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", () => resolve()));
  const address = server.address();
  if (!address || typeof address === "string") {
    server.close();
    throw new Error("failed to bind test server");
  }

  const plugin = createEvidenceGuardPlugin({
    evidenceUrl: `http://127.0.0.1:${address.port}`,
    failClosedRisk: "all",
  });

  try {
    const allowed = await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: { a: 1 } });
    assert.deepEqual(allowed.params, { mode: "safe" });
    assert.ok(!("block" in allowed));

    const denied = await plugin.hooks.before_tool_call({ toolName: "danger.tool", params: { a: 1 } });
    assert.equal(denied.block, true);
    assert.match(denied.blockReason ?? "", /PolicyDeny/);
  } finally {
    await new Promise<void>((resolve, reject) => {
      server.close((err) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }
});
