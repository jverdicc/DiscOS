import test from "node:test";
import assert from "node:assert/strict";
import http from "node:http";

import { createEvidenceGuardPlugin } from "../dist/index.js";

const EVIDENCEOS_URL = process.env.EVIDENCEOS_PREFLIGHT_URL;

async function startProxy(targetBaseUrl) {
  const seenRequestIds = [];
  const server = http.createServer((req, res) => {
    const requestId = req.headers["x-request-id"];
    if (typeof requestId === "string" && requestId.length > 0) {
      seenRequestIds.push(requestId);
    }

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
    close: () =>
      new Promise((resolve, reject) => {
        server.close((err) => (err ? reject(err) : resolve()));
      }),
  };
}

test("e2e preflight contract against daemon via plugin path", { skip: !EVIDENCEOS_URL }, async () => {
  const proxy = await startProxy(EVIDENCEOS_URL);
  const plugin = createEvidenceGuardPlugin({
    evidenceUrl: proxy.baseUrl,
    failClosedRisk: "all",
    timeoutMs: 5_000,
  });

  const candidates = [
    { toolName: "search.web", params: { query: "internal secrets" } },
    { toolName: "fs.delete_tree", params: { path: "/tmp/demo" } },
    { toolName: "exec", params: { cmd: "cat /etc/shadow" } },
  ];

  let observedRewrite = false;

  try {
    for (const candidate of candidates) {
      const out = await plugin.hooks.before_tool_call(candidate);
      if (out.params && JSON.stringify(out.params) !== JSON.stringify(candidate.params)) {
        observedRewrite = true;
      }
    }
  } finally {
    await proxy.close();
  }

  assert.ok(proxy.seenRequestIds.length >= candidates.length, "expected X-Request-Id on each request");
  for (const requestId of proxy.seenRequestIds) {
    assert.ok(requestId.length > 0, "X-Request-Id must be non-empty");
  }
  assert.ok(observedRewrite, "expected at least one DOWNGRADE rewrite from daemon preflight");
});
