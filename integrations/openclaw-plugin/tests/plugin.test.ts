import test from "node:test";
import assert from "node:assert/strict";

import {
  createEvidenceGuardPlugin,
  parseEvidenceGuardPluginConfig,
} from "../src/index.ts";

test("parseEvidenceGuardPluginConfig applies defaults and overrides", () => {
  const config = parseEvidenceGuardPluginConfig({
    evidenceUrl: "http://127.0.0.1:8787",
    timeoutMs: 250,
    failClosedRisk: "all",
    highRiskTools: ["custom.tool"],
  });

  assert.equal(config.evidenceUrl, "http://127.0.0.1:8787");
  assert.equal(config.timeoutMs, 250);
  assert.equal(config.circuitBreakerThreshold, 3);
  assert.equal(config.circuitBreakerResetMs, 5000);
  assert.equal(config.failClosedRisk, "all");
  assert.deepEqual(config.highRiskTools, ["custom.tool"]);
  assert.equal(typeof config.auditLogger, "function");
});

test("before_tool_call emits deterministic paramsHash for fixed policy input", async () => {
  const auditEvents: Array<{ paramsHash: string }> = [];

  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () =>
    new Response(
      JSON.stringify({
        decision: "ALLOW",
        reasonCode: "PolicyAllow",
      }),
      {
        status: 200,
        headers: { "content-type": "application/json" },
      },
    );

  try {
    const plugin = createEvidenceGuardPlugin({
      evidenceUrl: "http://127.0.0.1:8787",
      auditLogger: (event) => {
        auditEvents.push({ paramsHash: event.paramsHash });
      },
    });

    const result = await plugin.hooks.before_tool_call({
      toolName: "safe.tool",
      params: {
        alpha: 1,
        nested: { x: true, y: ["a", "b"] },
      },
    });

    assert.deepEqual(result, {});
    assert.equal(auditEvents.length, 1);
    assert.equal(auditEvents[0].paramsHash, "fnv1a32:42135e97");
  } finally {
    globalThis.fetch = originalFetch;
  }
});
