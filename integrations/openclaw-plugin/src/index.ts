export type Decision = "ALLOW" | "DENY" | "REQUIRE_HUMAN" | "DOWNGRADE";
export type PostflightDecision = "ALLOW" | "BLOCK" | "REQUIRE_HUMAN" | "REDACT";
export type InjectReceiptMode = "never" | "on_block" | "always";

export interface PreflightResponse {
  decision: Decision;
  reasonCode: string;
  reasonDetail?: string;
  rewrittenParams?: Record<string, unknown>;
  budgetDelta?: {
    spent: number;
    remaining: number;
  };
  receiptHash?: string;
  lane?: string;
}

export interface PostflightResponse {
  decision: PostflightDecision;
  reason?: string;
  outputRewrite?: unknown;
  lane?: string;
  budgetDelta?: number;
  budgetRemainingBits?: number;
  receiptHash: string;
}

export interface ToolCallContext {
  toolName: string;
  params: Record<string, unknown>;
  sessionId?: string;
  agentId?: string;
}

export interface HookResponse {
  block?: boolean;
  blockReason?: string;
  params?: Record<string, unknown>;
  receipt?: PreflightResponse;
}

export interface EvidenceGuardPluginConfig {
  evidenceUrl: string;
  postflightUrl?: string;
  aspecAdmitUrl?: string;
  bearerToken?: string;
  timeoutMs?: number;
  circuitBreakerThreshold?: number;
  circuitBreakerResetMs?: number;
  failClosedRisk?: "high-only" | "all";
  highRiskTools?: string[];
  sessionId?: string;
  agentId?: string;
  autoSessionId?: boolean;
  autoAgentId?: boolean;
  enablePostflight?: boolean;
  maxOutputBytes?: number;
  snapParamsMaxString?: number;
  snapParamsMaxArray?: number;
  injectReceiptToAgent?: InjectReceiptMode;
  toolWritePaths?: string[];
  requireAspecForToolWrites?: boolean;
  maxParamBytes?: number;
  redactLargeStringsOver?: number;
  redactFields?: Record<string, string[]>;
  toolMutationDirs?: string[];
  maxToolArtifactBytes?: number;
  auditLogger?: (event: AuditEvent) => void;
}

export interface ResolvedEvidenceGuardPluginConfig {
  evidenceUrl: string;
  postflightUrl: string;
  aspecAdmitUrl: string;
  bearerToken?: string;
  timeoutMs: number;
  circuitBreakerThreshold: number;
  circuitBreakerResetMs: number;
  failClosedRisk: "high-only" | "all";
  highRiskTools: string[];
  sessionId?: string;
  agentId?: string;
  autoSessionId: boolean;
  autoAgentId: boolean;
  enablePostflight: boolean;
  maxOutputBytes: number;
  snapParamsMaxString: number;
  snapParamsMaxArray: number;
  injectReceiptToAgent: InjectReceiptMode;
  toolWritePaths: string[];
  requireAspecForToolWrites: boolean;
  maxParamBytes: number;
  redactLargeStringsOver: number;
  redactFields: Record<string, string[]>;
  toolMutationDirs: string[];
  maxToolArtifactBytes: number;
  auditLogger: (event: AuditEvent) => void;
}

export interface AuditEvent {
  ts: string;
  stage: "preflight" | "postflight" | "aspec";
  toolName: string;
  paramsHash: string;
  outputHash?: string;
  decision: Decision | PostflightDecision;
  reasonCode: string;
  reasonDetail?: string;
  blocked: boolean;
  receiptHash?: string;
  lane?: string;
  budgetDelta?: {
    spent: number;
    remaining: number;
  };
  budgetRemainingBits?: number;
}

interface PreflightResponseWire {
  decision?: Decision;
  reasonCode?: string;
  reason_code?: string;
  reasonDetail?: string;
  reason_detail?: string;
  rewrittenParams?: Record<string, unknown>;
  rewritten_params?: Record<string, unknown>;
  budgetDelta?: {
    spent: number;
    remaining: number;
  };
  budget_delta?: {
    spent: number;
    remaining: number;
  };
  receiptHash?: string;
  receipt_hash?: string;
  lane?: string;
}

interface PostflightResponseWire {
  decision?: PostflightDecision;
  reason?: string;
  outputRewrite?: unknown;
  output_rewrite?: unknown;
  lane?: string;
  budgetDelta?: number;
  budget_delta?: number;
  budgetRemainingBits?: number;
  budget_remaining_bits?: number;
  receiptHash?: string;
  receipt_hash?: string;
}

const DEFAULT_TIMEOUT_MS = 120;
const DEFAULT_CIRCUIT_BREAKER_THRESHOLD = 3;
const DEFAULT_CIRCUIT_BREAKER_RESET_MS = 5000;
const DEFAULT_MAX_OUTPUT_BYTES = 4096;
const DEFAULT_SNAP_PARAMS_MAX_STRING = 2048;
const DEFAULT_SNAP_PARAMS_MAX_ARRAY = 64;
const DEFAULT_MAX_PARAM_BYTES = 8192;
const DEFAULT_REDACT_LARGE_STRINGS_OVER = 2048;
const DEFAULT_TOOL_MUTATION_DIRS = ["/tools/", "/plugins/", "/.openclaw/tools/"];
const DEFAULT_MAX_TOOL_ARTIFACT_BYTES = 1024 * 1024;
const NON_FALSIFIABLE_FIELDS = ["prompt", "systemPrompt", "chainOfThought", "cot", "reasoning", "messages"];
const DEFAULT_HIGH_RISK_TOOLS = [
  "exec",
  "shell.exec",
  "fs.write",
  "fs.delete_tree",
  "email.send",
  "payment.charge",
];
const DEFAULT_TOOL_WRITE_PATHS = [".openclaw/tools", "tools/", "workspace/tools"];

interface PendingRecord {
  startedAtMs: number;
  preflightReceiptHash?: string;
  lane?: string;
  decision: Decision;
}

function stableStringify(value: unknown): string {
  if (value === null || typeof value !== "object") {
    return JSON.stringify(value);
  }

  if (Array.isArray(value)) {
    return `[${value.map((item) => stableStringify(item)).join(",")}]`;
  }

  const entries = Object.entries(value as Record<string, unknown>).sort(([a], [b]) =>
    a.localeCompare(b),
  );
  return `{${entries
    .map(([k, v]) => `${JSON.stringify(k)}:${stableStringify(v)}`)
    .join(",")}}`;
}

function rightRotate(value: number, amount: number): number {
  return (value >>> amount) | (value << (32 - amount));
}

function sha256Hex(ascii: string): string {
  const mathPow = Math.pow;
  const maxWord = mathPow(2, 32);
  const words: number[] = [];
  const asciiBitLength = ascii.length * 8;
  const hash: number[] = [];
  const k: number[] = [];
  let primeCounter = 0;
  const isComposite: Record<number, boolean> = {};

  for (let candidate = 2; primeCounter < 64; candidate += 1) {
    if (!isComposite[candidate]) {
      for (let i = 0; i < 313; i += candidate) {
        isComposite[i] = true;
      }
      hash[primeCounter] = (mathPow(candidate, 0.5) * maxWord) | 0;
      k[primeCounter] = (mathPow(candidate, 1 / 3) * maxWord) | 0;
      primeCounter += 1;
    }
  }

  ascii += "\x80";
  while ((ascii.length % 64) - 56) ascii += "\x00";
  for (let i = 0; i < ascii.length; i += 1) {
    const j = ascii.charCodeAt(i);
    words[i >> 2] |= j << (((3 - i) % 4) * 8);
  }
  words[words.length] = ((asciiBitLength / maxWord) | 0);
  words[words.length] = (asciiBitLength);

  for (let j = 0; j < words.length;) {
    const w = words.slice(j, j += 16);
    const oldHash = hash.slice(0);

    for (let i = 0; i < 64; i += 1) {
      const i2 = i + j;
      let w15 = w[i - 15];
      let w2 = w[i - 2];

      const a = hash[0];
      const e = hash[4];
      const temp1 = hash[7]
        + (rightRotate(e, 6) ^ rightRotate(e, 11) ^ rightRotate(e, 25))
        + ((e & hash[5]) ^ ((~e) & hash[6]))
        + k[i]
        + (w[i] = (i < 16)
          ? w[i]
          : (w[i - 16]
            + (rightRotate(w15, 7) ^ rightRotate(w15, 18) ^ (w15 >>> 3))
            + w[i - 7]
            + (rightRotate(w2, 17) ^ rightRotate(w2, 19) ^ (w2 >>> 10))) | 0);
      const temp2 = (rightRotate(a, 2) ^ rightRotate(a, 13) ^ rightRotate(a, 22))
        + ((a & hash[1]) ^ (a & hash[2]) ^ (hash[1] & hash[2]));

      hash.unshift((temp1 + temp2) | 0);
      hash[4] = (hash[4] + temp1) | 0;
      hash.pop();
    }

    for (let i = 0; i < 8; i += 1) {
      hash[i] = (hash[i] + oldHash[i]) | 0;
    }
  }

  let result = "";
  for (let i = 0; i < 8; i += 1) {
    for (let j = 3; j + 1; j -= 1) {
      const b = (hash[i] >> (j * 8)) & 255;
      result += ((b < 16) ? 0 : "") + b.toString(16);
    }
  }
  return result;
}

function hashUnknown(value: unknown): string {
  return sha256Hex(stableStringify(value));
}

function truncateString(value: string, maxLen: number): Record<string, unknown> {
  if (value.length <= maxLen) {
    return { value };
  }
  return {
    truncated: true,
    len: value.length,
    sha256: sha256Hex(value),
    preview: value.slice(0, maxLen),
  };
}

function snapParams(
  value: unknown,
  maxString: number,
  maxArray: number,
  depth = 0,
): unknown {
  if (typeof value === "string") {
    return truncateString(value, maxString);
  }
  if (value === null || typeof value !== "object") {
    return value;
  }
  if (Array.isArray(value)) {
    const items = value.slice(0, maxArray).map((v) => snapParams(v, maxString, maxArray, depth + 1));
    if (value.length <= maxArray) {
      return items;
    }
    return {
      truncated: true,
      len: value.length,
      firstItems: items,
      remaining: value.length - maxArray,
    };
  }

  const out: Record<string, unknown> = {};
  const entries = Object.entries(value as Record<string, unknown>);
  const limit = depth > 0 ? maxArray : entries.length;
  const kept = entries.slice(0, limit);
  for (const [key, child] of kept) {
    const alwaysKeep = ["path", "url", "method", "model", "tool", "name", "cmd"].includes(key);
    if (alwaysKeep && typeof child === "string") {
      out[key] = truncateString(child, maxString * 2);
      continue;
    }
    out[key] = snapParams(child, maxString, maxArray, depth + 1);
  }
  if (entries.length > limit) {
    out.__truncatedKeys = entries.length - limit;
  }
  return out;
}

function byteLen(value: string): number {
  return new TextEncoder().encode(value).length;
}

function sha256Hex(value: string): string {
  // Fallback deterministic fingerprint for environments without Node crypto typings.
  return hashParams({ value }).replace("fnv1a32:", "");
}

function parsePreflightResponse(payload: PreflightResponseWire): PreflightResponse {
  if (!payload.decision) {
    throw new Error("missing decision");
  }

  return {
    decision: payload.decision,
    reasonCode: payload.reasonCode ?? payload.reason_code ?? "Unknown",
    reasonDetail: payload.reasonDetail ?? payload.reason_detail,
    rewrittenParams: payload.rewrittenParams ?? payload.rewritten_params,
    budgetDelta: payload.budgetDelta ?? payload.budget_delta,
    receiptHash: payload.receiptHash ?? payload.receipt_hash,
    lane: payload.lane,
  };
}

function parsePostflightResponse(payload: PostflightResponseWire): PostflightResponse {
  if (!payload.decision) {
    throw new Error("missing postflight decision");
  }
  const receiptHash = payload.receiptHash ?? payload.receipt_hash;
  if (!receiptHash) {
    throw new Error("missing receiptHash");
  }

  return {
    decision: payload.decision,
    reason: payload.reason,
    outputRewrite: payload.outputRewrite ?? payload.output_rewrite,
    lane: payload.lane,
    budgetDelta: payload.budgetDelta ?? payload.budget_delta,
    budgetRemainingBits: payload.budgetRemainingBits ?? payload.budget_remaining_bits,
    receiptHash,
  };
}

function withReceipt(result: unknown, receiptHash: string): unknown {
  if (result !== null && typeof result === "object") {
    return { ...(result as Record<string, unknown>), evidenceReceiptHash: receiptHash };
  }
  return { result, evidenceReceiptHash: receiptHash };
}

function normalizePath(raw: string): string {
  return raw.replaceAll("\\", "/").replace(/^\.\//, "").toLowerCase();
}

function matchesToolWritePath(rawPath: unknown, toolWritePaths: string[]): boolean {
  if (typeof rawPath !== "string") {
    return false;
  }
  const normalized = normalizePath(rawPath);
  return toolWritePaths.some((prefix) => normalized.startsWith(normalizePath(prefix)));
function redactValue(value: unknown, config: ResolvedEvidenceGuardPluginConfig): unknown {
  if (typeof value === "string" && value.length > config.redactLargeStringsOver) {
    return { __redacted: true, sha256: sha256Hex(value), len: value.length };
  }

  if (Array.isArray(value)) {
    return value.map((item) => redactValue(item, config));
  }

  if (value && typeof value === "object") {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
      out[k] = redactValue(v, config);
    }
    return out;
  }

  return value;
}

function snapshotParams(
  toolName: string,
  params: Record<string, unknown>,
  config: ResolvedEvidenceGuardPluginConfig,
): Record<string, unknown> {
  const dropFields = new Set([...(config.redactFields[toolName] ?? []), ...NON_FALSIFIABLE_FIELDS]);
  const reduced = Object.fromEntries(
    Object.entries(params)
      .filter(([key]) => !dropFields.has(key))
      .map(([key, value]) => [key, redactValue(value, config)]),
  );

  const json = stableStringify(reduced);
  if (byteLen(json) <= config.maxParamBytes) {
    return reduced;
  }

  const truncated = JSON.stringify(reduced).slice(0, config.maxParamBytes);
  return {
    __truncated: true,
    sha256: sha256Hex(json),
    len: byteLen(json),
    preview: truncated,
  };
}

function createSyntheticReceipt(decision: Decision, reasonCode: string, reasonDetail: string): PreflightResponse {
  return {
    decision,
    reasonCode,
    reasonDetail,
  };
}

function gateToolMutation(
  ctx: ToolCallContext,
  config: ResolvedEvidenceGuardPluginConfig,
): PreflightResponse | null {
  if (ctx.toolName !== "fs.write") {
    return null;
  }

  const path = typeof ctx.params.path === "string" ? ctx.params.path : "";
  const content = typeof ctx.params.content === "string" ? ctx.params.content : "";
  const inToolDir = config.toolMutationDirs.some((prefix) => path.startsWith(prefix));
  if (!inToolDir) {
    return null;
  }

  if (!path.endsWith(".wasm")) {
    return createSyntheticReceipt("DENY", "TOOL_ADMISSION_DENIED", "tool mutation only permits .wasm artifacts");
  }

  if (byteLen(content) > config.maxToolArtifactBytes) {
    return createSyntheticReceipt("DENY", "TOOL_ADMISSION_DENIED", "tool artifact exceeds maxToolArtifactBytes");
  }

  return null;
}

export function createEvidenceGuardPlugin(rawConfig: EvidenceGuardPluginConfig) {
  const config = parseEvidenceGuardPluginConfig(rawConfig);
  const env = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env;
  const defaultSessionId =
    config.sessionId
    ?? env?.EVIDENCEOS_SESSION_ID
    ?? `openclaw-${crypto.randomUUID()}`;
  const defaultAgentId =
    config.agentId
    ?? env?.EVIDENCEOS_AGENT_ID
    ?? "openclaw";

  const pending = new Map<string, PendingRecord[]>();

  let failures = 0;
  let circuitOpenedAt: number | null = null;

  function isHighRisk(toolName: string): boolean {
    return config.highRiskTools.includes(toolName);
  }

  function getIds(ctx: ToolCallContext): { sessionId?: string; agentId?: string } {
    return {
      sessionId: config.autoSessionId ? (ctx.sessionId ?? defaultSessionId) : ctx.sessionId,
      agentId: config.autoAgentId ? (ctx.agentId ?? defaultAgentId) : ctx.agentId,
    };
  }

  function keyFor(toolName: string, sessionId: string | undefined, paramsHash: string): string {
    return `${sessionId ?? ""}|${toolName}|${paramsHash}`;
  }

  function pushPending(toolName: string, sessionId: string | undefined, paramsHash: string, value: PendingRecord): void {
    const key = keyFor(toolName, sessionId, paramsHash);
    const existing = pending.get(key) ?? [];
    existing.push(value);
    pending.set(key, existing);
  }

  function popPending(toolName: string, sessionId: string | undefined, paramsHash: string): PendingRecord | undefined {
    const key = keyFor(toolName, sessionId, paramsHash);
    const existing = pending.get(key);
    if (!existing || existing.length === 0) {
      return undefined;
    }
    const found = existing.shift();
    if (existing.length === 0) {
      pending.delete(key);
    }
    return found;
  }

  function circuitOpen(now: number): boolean {
    if (circuitOpenedAt === null) {
      return false;
    }

    if (now - circuitOpenedAt >= config.circuitBreakerResetMs) {
      failures = 0;
      circuitOpenedAt = null;
      return false;
    }

    return true;
  }

  function maybeFailClosed(ctx: ToolCallContext, reasonCode: string, reasonDetail: string): HookResponse {
    const blocked = config.failClosedRisk === "all" || isHighRisk(ctx.toolName);
    const decision: Decision = blocked ? "DENY" : "ALLOW";
    const receipt = createSyntheticReceipt(decision, reasonCode, reasonDetail);

    config.auditLogger({
      ts: new Date().toISOString(),
      stage: "preflight",
      toolName: ctx.toolName,
      paramsHash: hashUnknown(ctx.params),
      decision,
      reasonCode,
      reasonDetail,
      blocked,
    });

    if (blocked) {
      return {
        block: true,
        blockReason: `${reasonCode}:${reasonDetail}`,
        receipt,
      };
    }

    return { receipt };
  }

  async function callEvidence(path: string, body: unknown): Promise<Response> {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), config.timeoutMs);
    const requestId = crypto.randomUUID();
    const headers: Record<string, string> = {
      "content-type": "application/json",
      "X-Request-Id": requestId,
    };
    if (config.bearerToken) {
      headers.Authorization = `Bearer ${config.bearerToken}`;
    }

    try {
      return await fetch(path, {
        method: "POST",
        headers,
        body: JSON.stringify({
          toolName: ctx.toolName,
          params: snapshotParams(ctx.toolName, ctx.params, config),
          sessionId: config.autoSessionId ? (ctx.sessionId ?? defaultSessionId) : ctx.sessionId,
          agentId: config.autoAgentId ? (ctx.agentId ?? defaultAgentId) : ctx.agentId,
        }),
        signal: controller.signal,
      });
    } finally {
      clearTimeout(timeout);
    }
  }

  async function preflight(ctx: ToolCallContext, snappedParams: unknown, paramsHash: string): Promise<PreflightResponse> {
    const ids = getIds(ctx);
    const response = await callEvidence(`${config.evidenceUrl}/v1/preflight_tool_call`, {
      toolName: ctx.toolName,
      params: snappedParams,
      paramsHash,
      sessionId: ids.sessionId,
      agentId: ids.agentId,
    });

    if (!response.ok) {
      throw new Error(`EvidenceOS returned ${response.status}`);
    }

    const payload = (await response.json()) as PreflightResponseWire;
    return parsePreflightResponse(payload);
  }

  async function aspecAdmit(moduleBase64: string): Promise<{ admissible: boolean; reason?: string; receiptHash?: string }> {
    const response = await callEvidence(config.aspecAdmitUrl, {
      moduleBase64,
      declaredExports: [],
      fuelLimit: 1_000_000,
    });
    if (!response.ok) {
      throw new Error(`EvidenceOS returned ${response.status}`);
    }
    const payload = (await response.json()) as { admissible: boolean; reason?: string; receiptHash?: string };
    return payload;
  }

  async function postflight(ctx: ToolCallContext, result: unknown, errorMessage?: string): Promise<PostflightResponse> {
    const ids = getIds(ctx);
    const paramsHash = hashUnknown(ctx.params);
    const pendingRecord = popPending(ctx.toolName, ids.sessionId, paramsHash);
    const startedAtMs = pendingRecord?.startedAtMs;

    const outputText = stableStringify(result);
    const outputBytes = new TextEncoder().encode(outputText).length;
    const outputHash = sha256Hex(outputText);
    const sendRaw = outputBytes <= config.maxOutputBytes;

    const response = await callEvidence(config.postflightUrl, {
      toolName: ctx.toolName,
      sessionId: ids.sessionId,
      agentId: ids.agentId,
      paramsHash,
      preflightReceiptHash: pendingRecord?.preflightReceiptHash,
      status: errorMessage ? "error" : "ok",
      output: sendRaw ? result : undefined,
      outputBytes,
      outputHash,
      errorMessage,
      startedAtMs,
      endedAtMs: Date.now(),
    });

    if (!response.ok) {
      throw new Error(`EvidenceOS returned ${response.status}`);
    }

    const payload = (await response.json()) as PostflightResponseWire;
    return parsePostflightResponse(payload);
  }

  async function maybeGateToolWrite(ctx: ToolCallContext): Promise<HookResponse | null> {
    if (!config.requireAspecForToolWrites) {
      return null;
    }
    if (!["fs.write", "file.write", "write_file"].includes(ctx.toolName)) {
      return null;
    }

    const path = (ctx.params.path ?? ctx.params.filePath) as unknown;
    if (!matchesToolWritePath(path, config.toolWritePaths)) {
      return null;
    }

    const content = typeof ctx.params.content === "string" ? ctx.params.content : undefined;
    if (typeof path !== "string" || !path.endsWith(".wasm") || !content) {
      return { block: true, blockReason: "AspecRequired:tool writes must be .wasm with inline content" };
    }

    const admitted = await aspecAdmit(content);
    const blocked = !admitted.admissible;
    config.auditLogger({
      ts: new Date().toISOString(),
      stage: "aspec",
      toolName: ctx.toolName,
      paramsHash: hashUnknown(ctx.params),
      decision: blocked ? "REQUIRE_HUMAN" : "ALLOW",
      reasonCode: blocked ? "AspecDenied" : "AspecAllowed",
      reasonDetail: admitted.reason,
      blocked,
      receiptHash: admitted.receiptHash,
    });
    if (blocked) {
      return { block: true, blockReason: `AspecDenied:${admitted.reason ?? "inadmissible"}` };
    }

    return null;
  }

  async function enforcePostflight(ctx: ToolCallContext, result: unknown, errorMessage?: string): Promise<unknown> {
    if (!config.enablePostflight) {
      return result;
    }

    const outcome = await postflight(ctx, result, errorMessage);
    const blocked = outcome.decision === "BLOCK" || outcome.decision === "REQUIRE_HUMAN";
    const outputHash = hashUnknown(result);

    config.auditLogger({
      ts: new Date().toISOString(),
      stage: "postflight",
      toolName: ctx.toolName,
      paramsHash: hashUnknown(ctx.params),
      outputHash,
      decision: outcome.decision,
      reasonCode: outcome.reason ?? "postflight",
      reasonDetail: outcome.reason,
      blocked,
      receiptHash: outcome.receiptHash,
      lane: outcome.lane,
      budgetRemainingBits: outcome.budgetRemainingBits,
    });

    if (outcome.decision === "REDACT") {
      return outcome.outputRewrite;
    }
    if (blocked) {
      throw new Error(`Postflight${outcome.decision}:${outcome.reason ?? "blocked"}`);
    }

    const shouldInject =
      config.injectReceiptToAgent === "always"
      || (config.injectReceiptToAgent === "on_block" && blocked);
    if (shouldInject) {
      return withReceipt(result, outcome.receiptHash);
    }

    return result;
  }

  return {
    name: "@evidenceos/openclaw-guard",
    priority: 1000,
    hooks: {
      before_tool_call: async (ctx: ToolCallContext): Promise<HookResponse> => {
        const mutationGate = gateToolMutation(ctx, config);
        if (mutationGate) {
          return {
            block: true,
            blockReason: `${mutationGate.reasonCode}:${mutationGate.reasonDetail ?? "n/a"}`,
            receipt: mutationGate,
          };
        }

        const now = Date.now();
        if (circuitOpen(now)) {
          return maybeFailClosed(ctx, "EvidenceUnavailable", "Circuit breaker is open");
        }

        try {
          const aspecDecision = await maybeGateToolWrite(ctx);
          if (aspecDecision) {
            return aspecDecision;
          }

          const paramsHash = hashUnknown(ctx.params);
          const snappedParams = snapParams(ctx.params, config.snapParamsMaxString, config.snapParamsMaxArray);
          const decision = await preflight(ctx, snappedParams, paramsHash);
          const ids = getIds(ctx);
          pushPending(ctx.toolName, ids.sessionId, paramsHash, {
            startedAtMs: now,
            preflightReceiptHash: decision.receiptHash,
            lane: decision.lane,
            decision: decision.decision,
          });
          failures = 0;

          const blocked =
            decision.decision === "DENY" || decision.decision === "REQUIRE_HUMAN";

          config.auditLogger({
            ts: new Date().toISOString(),
            stage: "preflight",
            toolName: ctx.toolName,
            paramsHash,
            decision: decision.decision,
            reasonCode: decision.reasonCode,
            reasonDetail: decision.reasonDetail,
            blocked,
            budgetDelta: decision.budgetDelta,
            receiptHash: decision.receiptHash,
            lane: decision.lane,
          });

          if (blocked) {
            return {
              block: true,
              blockReason: `${decision.reasonCode}:${decision.reasonDetail ?? "n/a"}`,
              receipt: decision,
            };
          }

          if (decision.rewrittenParams) {
            return {
              params: decision.rewrittenParams,
              receipt: decision,
            };
          }

          return { receipt: decision };
        } catch (error) {
          failures += 1;
          if (failures >= config.circuitBreakerThreshold) {
            circuitOpenedAt = Date.now();
          }

          const reason = error instanceof Error ? error.message : "Unknown failure";
          return maybeFailClosed(ctx, "EvidenceUnavailable", reason);
        }
      },
      after_tool_call: async (ctx: ToolCallContext, result: unknown): Promise<unknown> => enforcePostflight(ctx, result),
      tool_result_persist: async (ctx: ToolCallContext, result: unknown): Promise<unknown> => enforcePostflight(ctx, result),
    },
  };
}

export function parseEvidenceGuardPluginConfig(
  rawConfig: EvidenceGuardPluginConfig,
): ResolvedEvidenceGuardPluginConfig {
  const envToken = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env?.EVIDENCEOS_TOKEN;

  const baseEvidenceUrl = rawConfig.evidenceUrl.replace(/\/$/, "");
  const postflightUrl = rawConfig.postflightUrl ?? `${baseEvidenceUrl}/v1/postflight_tool_call`;

  return {
    bearerToken: envToken,
    timeoutMs: DEFAULT_TIMEOUT_MS,
    circuitBreakerThreshold: DEFAULT_CIRCUIT_BREAKER_THRESHOLD,
    circuitBreakerResetMs: DEFAULT_CIRCUIT_BREAKER_RESET_MS,
    failClosedRisk: "high-only" as const,
    highRiskTools: [...DEFAULT_HIGH_RISK_TOOLS],
    autoSessionId: true,
    autoAgentId: true,
    enablePostflight: true,
    maxOutputBytes: DEFAULT_MAX_OUTPUT_BYTES,
    snapParamsMaxString: DEFAULT_SNAP_PARAMS_MAX_STRING,
    snapParamsMaxArray: DEFAULT_SNAP_PARAMS_MAX_ARRAY,
    injectReceiptToAgent: "on_block",
    toolWritePaths: [...DEFAULT_TOOL_WRITE_PATHS],
    requireAspecForToolWrites: true,
    maxParamBytes: DEFAULT_MAX_PARAM_BYTES,
    redactLargeStringsOver: DEFAULT_REDACT_LARGE_STRINGS_OVER,
    redactFields: {},
    toolMutationDirs: [...DEFAULT_TOOL_MUTATION_DIRS],
    maxToolArtifactBytes: DEFAULT_MAX_TOOL_ARTIFACT_BYTES,
    auditLogger: (event: AuditEvent) => {
      console.log(JSON.stringify({ type: "evidenceos.audit", ...event }));
    },
    postflightUrl,
    aspecAdmitUrl: `${baseEvidenceUrl}/v1/aspec_admit`,
    ...rawConfig,
  };
}
