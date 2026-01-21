/**
 * Price utilities for the Lightcone SDK.
 *
 * This module provides helper functions for working with decimal string prices.
 * The SDK uses String types for price/size/balance fields to preserve
 * the exact decimal representation from the server, as different tokens
 * have different decimal places (USDC=6, SOL=9, BTC=8, etc.).
 */

/**
 * Parse a decimal string to number for calculations.
 *
 * @example
 * ```typescript
 * parseDecimal("0.500000"); // returns 0.5
 * parseDecimal("1.000000"); // returns 1.0
 * ```
 */
export function parseDecimal(s: string): number {
  const result = parseFloat(s);
  if (isNaN(result)) {
    throw new Error(`Invalid decimal string: ${s}`);
  }
  return result;
}

/**
 * Format a number as a decimal string with specified precision.
 *
 * @example
 * ```typescript
 * formatDecimal(0.5, 6);  // returns "0.500000"
 * formatDecimal(1.0, 6);  // returns "1.000000"
 * ```
 */
export function formatDecimal(value: number, precision: number): string {
  return value.toFixed(precision);
}

/**
 * Check if a size string represents zero.
 * Used for determining whether a price level should be removed.
 */
export function isZeroSize(size: string): boolean {
  return (
    size === "0" ||
    size === "0.0" ||
    size === "0.000000" ||
    parseFloat(size) === 0
  );
}
