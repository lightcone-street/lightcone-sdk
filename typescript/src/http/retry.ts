export interface RetryConfig {
  maxRetries: number;
  initialDelayMs: number;
  maxDelayMs: number;
  backoffFactor: number;
  jitter: boolean;
  retryableStatuses: readonly number[];
}

export type RetryPolicy =
  | { kind: "none" }
  | { kind: "idempotent" }
  | { kind: "custom"; config: RetryConfig };

export const RetryPolicy = {
  None: { kind: "none" } as const,
  Idempotent: { kind: "idempotent" } as const,
  custom(config: RetryConfig): RetryPolicy {
    return { kind: "custom", config };
  },
};

export const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  initialDelayMs: 200,
  maxDelayMs: 10_000,
  backoffFactor: 2,
  jitter: true,
  retryableStatuses: [502, 503, 504],
};

export function idempotentRetryConfig(): RetryConfig {
  return {
    ...DEFAULT_RETRY_CONFIG,
    retryableStatuses: [429, 502, 503, 504],
  };
}

export function retryConfigForPolicy(policy: RetryPolicy): RetryConfig | null {
  switch (policy.kind) {
    case "none":
      return null;
    case "idempotent":
      return idempotentRetryConfig();
    case "custom":
      return policy.config;
  }
}

export function delayForAttempt(config: RetryConfig, attempt: number): number {
  const base = config.initialDelayMs * config.backoffFactor ** attempt;
  const capped = Math.min(base, config.maxDelayMs);
  if (!config.jitter) {
    return Math.max(0, Math.floor(capped));
  }

  const jitterRange = capped * 0.25;
  const jitter = (Math.random() - 0.5) * 2 * jitterRange;
  return Math.max(0, Math.floor(capped + jitter));
}
