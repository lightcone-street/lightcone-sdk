/**
 * Shared utilities used across API and WebSocket modules.
 */

export { Resolution } from "./types";
export { parseDecimal, formatDecimal, isZero } from "./price";
export {
  scalePriceSize,
  ScalingError,
} from "./scaling";
export type { OrderbookDecimals, ScaledAmounts } from "./scaling";

// deriveOrderbookId is exported from the program module (orders.ts)
// Re-exported here for convenience via the shared module path
