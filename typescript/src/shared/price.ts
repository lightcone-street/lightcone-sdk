import Decimal from "decimal.js";

import { SdkError } from "../error";

/**
 * Parse a decimal string to a number.
 */
export function parseDecimal(value: string): number {
  const parsed = Number.parseFloat(value);
  if (Number.isNaN(parsed)) {
    throw SdkError.validation(`Invalid decimal string: ${value}`);
  }
  return parsed;
}

/**
 * Format a number to a fixed decimal string.
 */
export function formatDecimal(value: number, precision: number): string {
  return value.toFixed(precision);
}

/**
 * Whether a decimal string represents numeric zero.
 */
export function isZero(value: string): boolean {
  const parsed = Number.parseFloat(value);
  return !Number.isNaN(parsed) && parsed === 0;
}

/**
 * Division context for derived prices: 28 significant digits, the precision
 * of the Rust SDK's 96-bit decimal, rounding half to even.
 */
const PriceDecimal = Decimal.clone({ precision: 28, rounding: Decimal.ROUND_HALF_EVEN });

/**
 * Exact quote per base as normalized decimal text (no exponent, no trailing
 * zeros), or `undefined` for a zero base amount.
 */
export function quotePerBase(
  quote: Decimal.Value,
  base: Decimal.Value,
): string | undefined {
  const divisor = new PriceDecimal(base);
  if (divisor.isZero()) return undefined;
  return new PriceDecimal(quote).div(divisor).toFixed();
}
