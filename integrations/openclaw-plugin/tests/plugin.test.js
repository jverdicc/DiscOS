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
  assert.equal(config.postflightUrl, "http://127.0.0.1:8787/v1/postflight_tool_call");
  assert.equal(config.timeoutMs, 250);
  assert.equal(config.circuitBreakerThreshold, 3);
  assert.equal(config.circuitBreakerResetMs, 5000);
  assert.equal(config.failClosedRisk, "all");
  assert.deepEqual(config.highRiskTools, ["custom.tool"]);
  assert.equal(config.maxOutputBytes, 4096);
  assert.equal(config.snapParamsMaxString, 2048);
  assert.equal(config.snapParamsMaxArray, 64);
  assert.equal(config.injectReceiptToAgent, "on_block");
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
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("before_tool_call sends snapped params and full paramsHash", async () => {
  const originalFetch = globalThis.fetch;
  const seen = [];
  globalThis.fetch = async (_input, init) => {
    seen.push(JSON.parse(String(init?.body)));
    return new Response(JSON.stringify({ decision: "ALLOW", reasonCode: "PolicyAllow", receiptHash: "pre-1" }), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };

  try {
    const plugin = createEvidenceGuardPlugin({
      evidenceUrl: "http://127.0.0.1:8787",
      snapParamsMaxString: 10,
      snapParamsMaxArray: 2,
      requireAspecForToolWrites: false,
    });

    await plugin.hooks.before_tool_call({
      toolName: "safe.tool",
      params: {
        x: "abcdefghijklmnopqrstuvwxyz",
        values: [1, 2, 3, 4],
      },
    });

    assert.equal(seen.length, 1);
    assert.equal(typeof seen[0].paramsHash, "string");
    assert.equal(seen[0].paramsHash.length, 64);
    assert.equal(seen[0].params.x.truncated, true);
    assert.equal(seen[0].params.values.truncated, true);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("after_tool_call enforces REDACT rewrite", async () => {
  const originalFetch = globalThis.fetch;
  const calls = [];
  globalThis.fetch = async (_input, init) => {
    const body = JSON.parse(String(init?.body));
    calls.push(body);
    if (String(_input).includes("preflight")) {
      return new Response(JSON.stringify({ decision: "ALLOW", reasonCode: "PolicyAllow", receiptHash: "pre-1" }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    }

    return new Response(JSON.stringify({
      decision: "REDACT",
      reason: "large output",
      outputRewrite: { truncated: true, preview: "abc" },
      receiptHash: "post-1",
    }), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };

  try {
    const plugin = createEvidenceGuardPlugin({ evidenceUrl: "http://127.0.0.1:8787", requireAspecForToolWrites: false });
    const ctx = { toolName: "safe.tool", params: { x: 1 }, sessionId: "s1" };
    await plugin.hooks.before_tool_call(ctx);
    const out = await plugin.hooks.after_tool_call(ctx, { hello: "world" });

    assert.deepEqual(out, { truncated: true, preview: "abc" });
    assert.equal(calls.length, 2);
    assert.equal(calls[1].preflightReceiptHash, "pre-1");
    assert.equal(calls[1].outputHash.length, 64);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("before_tool_call blocks tool writes without wasm+content when ASPEC gate enabled", async () => {
  const plugin = createEvidenceGuardPlugin({
    evidenceUrl: "http://127.0.0.1:8787",
    requireAspecForToolWrites: true,
  });

  const response = await plugin.hooks.before_tool_call({
    toolName: "fs.write",
    params: { path: "tools/new-tool.js", content: "console.log(1);" },
  });

  assert.equal(response.block, true);
  assert.match(response.blockReason ?? "", /AspecRequired/);
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
      stage: "preflight",
      toolName: "safe.tool",
      paramsHash: "a".repeat(64),
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
  assert.equal(parsed.paramsHash, "a".repeat(64));
});
