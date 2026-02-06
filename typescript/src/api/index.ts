/**
 * REST API client module for Lightcone.
 *
 * This module provides HTTP client functionality for interacting with
 * the Lightcone REST API for orderbook data, market info, and more.
 *
 * @example
 * ```typescript
 * import { api } from "@lightcone/sdk";
 *
 * const client = new api.LightconeApiClient();
 * const markets = await client.getMarkets();
 * ```
 *
 * @module api
 */

// Client
export {
  LightconeApiClient,
  DEFAULT_API_URL,
  DEFAULT_RETRY_CONFIG,
} from "./client";
export type { LightconeApiClientConfig, RetryConfig } from "./client";

// Error types
export { ApiError } from "./error";
export type { ApiErrorVariant, ErrorResponse } from "./error";

// Validation utilities
export {
  validatePubkey,
  validateSignature,
  validateLimit,
  MAX_PAGINATION_LIMIT,
  DEFAULT_TIMEOUT_MS,
} from "./validation";

// All types
export * from "./types";
