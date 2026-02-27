import test from "node:test";
import assert from "node:assert/strict";
import http from "node:http";

import { createEvidenceGuardPlugin } from "../dist/index.js";

const EVIDENCEOS_URL = process.env.EVIDENCEOS_PREFLIGHT_URL;

async function startProxy(targetBaseUrl) {
  const seenRequestIds = [];
  const seenPaths = [];
  const server = http.createServer((req, res) => {
    const requestId = req.headers["x-request-id"];
    if (typeof requestId === "string" && requestId.length > 0) {
      seenRequestIds.push(requestId);
    }
    seenPaths.push(req.url ?? "");

    const chunks = [];
    req.on("data", (chunk) => chunks.push(chunk));
    req.on("end", async () => {
      try {
        const upstream = await fetch(`${targetBaseUrl}${req.url ?? ""}`, {
          method: req.method,
          headers: req.headers,
          body: chunks.length > 0 ? Buffer.concat(chunks) : undefined,
        });

        res.statusCode = upstream.status;
        upstream.headers.forEach((value, key) => res.setHeader(key, value));
        const body = Buffer.from(await upstream.arrayBuffer());
        res.end(body);
      } catch (error) {
        res.statusCode = 502;
        res.end(JSON.stringify({ error: String(error) }));
      }
    });
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", () => resolve()));
  const address = server.address();
  if (!address || typeof address === "string") {
    throw new Error("proxy bind failed");
  }

  return {
    baseUrl: `http://127.0.0.1:${address.port}`,
    seenRequestIds,
    seenPaths,
    close: () =>
      new Promise((resolve, reject) => {
        server.close((err) => (err ? reject(err) : resolve()));
      }),
  };
}

test("e2e preflight+postflight contract against daemon via plugin path", { skip: !EVIDENCEOS_URL }, async () => {
  const proxy = await startProxy(EVIDENCEOS_URL);
  const plugin = createEvidenceGuardPlugin({
    evidenceUrl: proxy.baseUrl,
    failClosedRisk: "all",
    timeoutMs: 5_000,
    maxOutputBytes: 16,
    requireAspecForToolWrites: false,
  });

  const candidate = { toolName: "search.web", params: { query: "internal secrets" }, sessionId: "e2e-session" };

  try {
    await plugin.hooks.after_tool_call(candidate, { text: "small output" });
  const candidates = [
    { toolName: "search.web", params: { query: "internal secrets" } },
    { toolName: "fs.delete_tree", params: { path: "/tmp/demo" } },
    { toolName: "exec", params: { cmd: "cat /etc/shadow" } },
  ];

  let observedRewrite = false;
  let requestsMade = 0;

  try {
    for (const candidate of candidates.slice(0, 2)) {
      await plugin.hooks.before_tool_call(candidate);
      requestsMade += 1;
    }

    const execCandidate = candidates[2];
    for (let i = 0; i < 64 && !observedRewrite; i += 1) {
      const out = await plugin.hooks.before_tool_call(execCandidate);
      requestsMade += 1;
      if (out.params && JSON.stringify(out.params) !== JSON.stringify(execCandidate.params)) {
        observedRewrite = true;
      }
    }
  } finally {
    await proxy.close();
  }

  assert.ok(proxy.seenRequestIds.length >= 2, "expected X-Request-Id on each request");
  assert.ok(proxy.seenPaths.some((path) => path.includes("/v1/postflight_tool_call")));
  assert.ok(proxy.seenRequestIds.length >= requestsMade, "expected X-Request-Id on each request");
  for (const requestId of proxy.seenRequestIds) {
    assert.ok(requestId.length > 0, "X-Request-Id must be non-empty");
  }
  assert.ok(observedRewrite, "expected at least one DOWNGRADE rewrite from daemon preflight");
});
