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

test("before_tool_call fails closed on timeout for high-risk tools", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (_input, init) => {
    await new Promise((resolve, reject) => {
      init?.signal?.addEventListener("abort", () => reject(new Error("aborted")));
    });
    return new Response();
  };

  try {
    const plugin = createEvidenceGuardPlugin({
      evidenceUrl: "http://127.0.0.1:8787",
      timeoutMs: 10,
      failClosedRisk: "high-only",
      highRiskTools: ["exec"],
    });

    const response = await plugin.hooks.before_tool_call({
      toolName: "exec",
      params: { cmd: "whoami" },
    });

    assert.equal(response.block, true);
    assert.match(response.blockReason ?? "", /EvidenceUnavailable/);
    assert.ok(!("block" in (response.params ?? {})));
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("circuit breaker opens after threshold and then resets", async () => {
  const calls: string[] = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () => {
    calls.push("called");
    throw new Error("upstream down");
  };

  try {
    const plugin = createEvidenceGuardPlugin({
      evidenceUrl: "http://127.0.0.1:8787",
      circuitBreakerThreshold: 2,
      circuitBreakerResetMs: 20,
      failClosedRisk: "all",
    });

    await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: {} });
    await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: {} });
    await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: {} });
    assert.equal(calls.length, 2, "third call should be blocked by open circuit");

    await new Promise((resolve) => setTimeout(resolve, 25));
    await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: {} });
    assert.equal(calls.length, 3, "fetch should resume after reset window");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("before_tool_call never emits block:false and only rewrites params when returned", async () => {
  const originalFetch = globalThis.fetch;
  const queue = [
    { decision: "ALLOW", reasonCode: "PolicyAllow" },
    { decision: "ALLOW", reasonCode: "PolicyAllow", rewrittenParams: { safe: true } },
  ];

  globalThis.fetch = async () =>
    new Response(JSON.stringify(queue.shift()), {
      status: 200,
      headers: { "content-type": "application/json" },
    });

  try {
    const plugin = createEvidenceGuardPlugin({
      evidenceUrl: "http://127.0.0.1:8787",
    });

    const first = await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: { x: 1 } });
    assert.ok(!("block" in first));
    assert.ok(!("params" in first));

    const second = await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: { x: 1 } });
    assert.ok(!("block" in second));
    assert.deepEqual(second.params, { safe: true });
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("default audit logger emits deterministic one-line JSON", () => {
  const config = parseEvidenceGuardPluginConfig({
    evidenceUrl: "http://127.0.0.1:8787",
  });

  const originalLog = console.log;
  const lines: string[] = [];
  console.log = (line: string) => {
    lines.push(line);
  };

  try {
    config.auditLogger({
      ts: "2026-01-01T00:00:00.000Z",
      toolName: "safe.tool",
      paramsHash: "fnv1a32:42135e97",
      decision: "ALLOW",
      reasonCode: "PolicyAllow",
      blocked: false,
    });
  } finally {
    console.log = originalLog;
  }

  assert.equal(lines.length, 1);
  assert.ok(!lines[0].includes("\n"));
  const parsed = JSON.parse(lines[0]);
  assert.equal(parsed.type, "evidenceos.audit");
  assert.equal(parsed.toolName, "safe.tool");
  assert.equal(parsed.paramsHash, "fnv1a32:42135e97");
});
