import test from "node:test";
import assert from "node:assert/strict";

import {
  createEvidenceGuardPlugin,
  parseEvidenceGuardPluginConfig,
} from "../dist/index.js";

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
  assert.equal(config.maxParamBytes, 8192);
  assert.equal(config.redactLargeStringsOver, 2048);
  assert.equal(typeof config.auditLogger, "function");
});

test("parseEvidenceGuardPluginConfig uses EVIDENCEOS_TOKEN by default and allows override", () => {
  const originalToken = process.env.EVIDENCEOS_TOKEN;
  process.env.EVIDENCEOS_TOKEN = "env-token";

  try {
    const fromEnv = parseEvidenceGuardPluginConfig({
      evidenceUrl: "http://127.0.0.1:8787",
    });
    assert.equal(fromEnv.bearerToken, "env-token");

    const overridden = parseEvidenceGuardPluginConfig({
      evidenceUrl: "http://127.0.0.1:8787",
      bearerToken: "explicit-token",
    });
    assert.equal(overridden.bearerToken, "explicit-token");
  } finally {
    if (originalToken === undefined) {
      delete process.env.EVIDENCEOS_TOKEN;
    } else {
      process.env.EVIDENCEOS_TOKEN = originalToken;
    }
  }
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
    assert.equal(response.receipt?.decision, "DENY");
    assert.ok(!("block" in (response.params ?? {})));
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("circuit breaker opens after threshold and then resets", async () => {
  const calls = [];
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
    assert.equal(first.receipt?.decision, "ALLOW");

    const second = await plugin.hooks.before_tool_call({ toolName: "safe.tool", params: { x: 1 } });
    assert.ok(!("block" in second));
    assert.deepEqual(second.params, { safe: true });
    assert.equal(second.receipt?.decision, "ALLOW");
  } finally {
    globalThis.fetch = originalFetch;
  }
});



test("before_tool_call fails closed when preflight response omits decision", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () =>
    new Response(JSON.stringify({ reasonCode: "PolicyAllow" }), {
      status: 200,
      headers: { "content-type": "application/json" },
    });

  try {
    const plugin = createEvidenceGuardPlugin({
      evidenceUrl: "http://127.0.0.1:8787",
      failClosedRisk: "high-only",
      highRiskTools: ["exec"],
    });

    const response = await plugin.hooks.before_tool_call({
      toolName: "exec",
      params: { cmd: "whoami" },
    });

    assert.equal(response.block, true);
    assert.match(response.blockReason ?? "", /missing decision/);
    assert.equal(response.receipt?.decision, "DENY");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("default audit logger emits deterministic one-line JSON", () => {
  const config = parseEvidenceGuardPluginConfig({
    evidenceUrl: "http://127.0.0.1:8787",
  });

  const originalLog = console.log;
  const lines = [];
  console.log = (line) => {
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


test("tool mutation gate denies non-wasm writes in tool directories", async () => {
  const originalFetch = globalThis.fetch;
  let called = false;
  globalThis.fetch = async () => {
    called = true;
    return new Response();
  };

  try {
    const plugin = createEvidenceGuardPlugin({
      evidenceUrl: "http://127.0.0.1:8787",
    });

    const response = await plugin.hooks.before_tool_call({
      toolName: "fs.write",
      params: { path: "/tools/not-allowed.txt", content: "abc" },
    });

    assert.equal(called, false);
    assert.equal(response.block, true);
    assert.equal(response.receipt?.reasonCode, "TOOL_ADMISSION_DENIED");
  } finally {
    globalThis.fetch = originalFetch;
  }
});
