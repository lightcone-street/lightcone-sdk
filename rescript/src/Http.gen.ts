/* TypeScript file generated from Http.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type RetryConfig_t = {
  readonly maxRetries: number;
  readonly initialDelayMs: number;
  readonly maxDelayMs: number;
  readonly backoffFactor: number;
  readonly jitter: boolean;
  readonly retryableStatuses: number[]
};

export type RetryPolicy_t =
    "NoRetry"
  | "Idempotent"
  | { TAG: "Custom"; _0: RetryConfig_t };

export abstract class t { protected opaque!: any }; /* simulate opaque types */
