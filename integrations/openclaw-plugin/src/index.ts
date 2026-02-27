export type Decision = "ALLOW" | "DENY" | "REQUIRE_HUMAN" | "DOWNGRADE";

export interface PreflightResponse {
  decision: Decision;
  reasonCode: string;
  reasonDetail?: string;
  rewrittenParams?: Record<string, unknown>;
  budgetDelta?: {
    spent: number;
    remaining: number;
  };
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
  maxParamBytes?: number;
  redactLargeStringsOver?: number;
  redactFields?: Record<string, string[]>;
  toolMutationDirs?: string[];
  maxToolArtifactBytes?: number;
  auditLogger?: (event: AuditEvent) => void;
}

export interface ResolvedEvidenceGuardPluginConfig {
  evidenceUrl: string;
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
  maxParamBytes: number;
  redactLargeStringsOver: number;
  redactFields: Record<string, string[]>;
  toolMutationDirs: string[];
  maxToolArtifactBytes: number;
  auditLogger: (event: AuditEvent) => void;
}

export interface AuditEvent {
  ts: string;
  toolName: string;
  paramsHash: string;
  decision: Decision;
  reasonCode: string;
  reasonDetail?: string;
  blocked: boolean;
  budgetDelta?: {
    spent: number;
    remaining: number;
  };
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
}

const DEFAULT_TIMEOUT_MS = 120;
const DEFAULT_CIRCUIT_BREAKER_THRESHOLD = 3;
const DEFAULT_CIRCUIT_BREAKER_RESET_MS = 5000;
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

function hashParams(params: Record<string, unknown>): string {
  const data = stableStringify(params);
  let h = 2166136261;
  for (let i = 0; i < data.length; i += 1) {
    h ^= data.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return `fnv1a32:${(h >>> 0).toString(16).padStart(8, "0")}`;
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
  };
}

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

  let failures = 0;
  let circuitOpenedAt: number | null = null;

  function isHighRisk(toolName: string): boolean {
    return config.highRiskTools.includes(toolName);
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
      toolName: ctx.toolName,
      paramsHash: hashParams(ctx.params),
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

  async function preflight(ctx: ToolCallContext): Promise<PreflightResponse> {
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
      const response = await fetch(`${config.evidenceUrl}/v1/preflight_tool_call`, {
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

      if (!response.ok) {
        throw new Error(`EvidenceOS returned ${response.status}`);
      }

      const payload = (await response.json()) as PreflightResponseWire;
      return parsePreflightResponse(payload);
    } finally {
      clearTimeout(timeout);
    }
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
          const decision = await preflight(ctx);
          failures = 0;

          const blocked =
            decision.decision === "DENY" || decision.decision === "REQUIRE_HUMAN";

          config.auditLogger({
            ts: new Date().toISOString(),
            toolName: ctx.toolName,
            paramsHash: hashParams(ctx.params),
            decision: decision.decision,
            reasonCode: decision.reasonCode,
            reasonDetail: decision.reasonDetail,
            blocked,
            budgetDelta: decision.budgetDelta,
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
    },
  };
}

export function parseEvidenceGuardPluginConfig(
  rawConfig: EvidenceGuardPluginConfig,
): ResolvedEvidenceGuardPluginConfig {
  const envToken = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env?.EVIDENCEOS_TOKEN;

  return {
    bearerToken: envToken,
    timeoutMs: DEFAULT_TIMEOUT_MS,
    circuitBreakerThreshold: DEFAULT_CIRCUIT_BREAKER_THRESHOLD,
    circuitBreakerResetMs: DEFAULT_CIRCUIT_BREAKER_RESET_MS,
    failClosedRisk: "high-only" as const,
    highRiskTools: [...DEFAULT_HIGH_RISK_TOOLS],
    autoSessionId: true,
    autoAgentId: true,
    maxParamBytes: DEFAULT_MAX_PARAM_BYTES,
    redactLargeStringsOver: DEFAULT_REDACT_LARGE_STRINGS_OVER,
    redactFields: {},
    toolMutationDirs: [...DEFAULT_TOOL_MUTATION_DIRS],
    maxToolArtifactBytes: DEFAULT_MAX_TOOL_ARTIFACT_BYTES,
    auditLogger: (event: AuditEvent) => {
      // Deterministic one-line JSON for machine ingestion.
      console.log(JSON.stringify({ type: "evidenceos.audit", ...event }));
    },
    ...rawConfig,
  };
}
