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
}

export interface EvidenceGuardPluginConfig {
  evidenceUrl: string;
  bearerToken?: string;
  timeoutMs?: number;
  circuitBreakerThreshold?: number;
  circuitBreakerResetMs?: number;
  failClosedRisk?: "high-only" | "all";
  highRiskTools?: string[];
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

function parsePreflightResponse(payload: PreflightResponseWire): PreflightResponse {
  return {
    decision: payload.decision ?? "ALLOW",
    reasonCode: payload.reasonCode ?? payload.reason_code ?? "Unknown",
    reasonDetail: payload.reasonDetail ?? payload.reason_detail,
    rewrittenParams: payload.rewrittenParams ?? payload.rewritten_params,
    budgetDelta: payload.budgetDelta ?? payload.budget_delta,
  };
}

export function createEvidenceGuardPlugin(rawConfig: EvidenceGuardPluginConfig) {
  const config = parseEvidenceGuardPluginConfig(rawConfig);

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
      };
    }

    // Important: never set block=false to avoid merge override collisions.
    return {};
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
        body: JSON.stringify(ctx),
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
            };
          }

          if (decision.rewrittenParams) {
            return {
              params: decision.rewrittenParams,
            };
          }

          // Allow-path intentionally returns empty object; never set block=false.
          return {};
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
    auditLogger: (event: AuditEvent) => {
      // Deterministic one-line JSON for machine ingestion.
      console.log(JSON.stringify({ type: "evidenceos.audit", ...event }));
    },
    ...rawConfig,
  };
}
