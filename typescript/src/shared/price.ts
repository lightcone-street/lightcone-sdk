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

/** Largest 96-bit mantissa, the Rust SDK decimal's integer limit. */
const MAX_MANTISSA = (1n << 96n) - 1n;
/** Largest fractional scale of the Rust SDK decimal. */
const MAX_SCALE = 28;

const PLAIN_DECIMAL = /^-?[0-9]+(\.[0-9]+)?$/;

/**
 * Split a decimal into a signed integer mantissa and a scale. Plain decimal
 * text keeps its written scale, as the Rust SDK decimal does.
 */
function toScaled(value: Decimal.Value): { mantissa: bigint; scale: number } {
  const text = typeof value === "string" && PLAIN_DECIMAL.test(value)
    ? value
    : new Decimal(value).toFixed();
  const negative = text.startsWith("-");
  const unsigned = negative ? text.slice(1) : text;
  const [whole, fraction = ""] = unsigned.split(".");
  const mantissa = BigInt(whole + fraction);
  return { mantissa: negative ? -mantissa : mantissa, scale: fraction.length };
}

/** Round `numerator / denominator` (both non-negative) half to even. */
function divRoundHalfEven(numerator: bigint, denominator: bigint): bigint {
  const quotient = numerator / denominator;
  const twice = (numerator - quotient * denominator) * 2n;
  if (twice > denominator || (twice === denominator && quotient % 2n === 1n)) {
    return quotient + 1n;
  }
  return quotient;
}

/**
 * Exact quote per base as normalized decimal text (no exponent, no trailing
 * zeros), or `undefined` for a zero base amount or a quotient beyond the
 * 96-bit range.
 *
 * Matches the Rust SDK's `quote_per_base`: the quotient keeps the largest
 * scale, at most 28, whose mantissa fits in 96 bits, and rounds half to even.
 *
 * @internal
 */
export function quotePerBase(
  quote: Decimal.Value,
  base: Decimal.Value,
): string | undefined {
  const dividend = toScaled(quote);
  const divisor = toScaled(base);
  if (divisor.mantissa === 0n) return undefined;

  const negative = (dividend.mantissa < 0n) !== (divisor.mantissa < 0n);
  const numerator = (dividend.mantissa < 0n ? -dividend.mantissa : dividend.mantissa)
    * 10n ** BigInt(divisor.scale);
  const denominator = (divisor.mantissa < 0n ? -divisor.mantissa : divisor.mantissa)
    * 10n ** BigInt(dividend.scale);

  for (let scale = MAX_SCALE; scale >= 0; scale -= 1) {
    const scaled = numerator * 10n ** BigInt(scale);
    if (scaled / denominator > MAX_MANTISSA) continue;
    const mantissa = divRoundHalfEven(scaled, denominator);
    if (mantissa > MAX_MANTISSA) continue;
    if (mantissa === 0n) return "0";
    return new Decimal(`${negative ? "-" : ""}${mantissa}e-${scale}`).toFixed();
  }
  return undefined;
}

/**
 * Exact sum of two decimals at the larger of their scales, as the Rust SDK
 * decimal adds. Independent of any global decimal.js precision.
 *
 * @internal
 */
export function addDecimals(left: Decimal.Value, right: Decimal.Value): string {
  const a = toScaled(left);
  const b = toScaled(right);
  const scale = Math.max(a.scale, b.scale);
  const sum = a.mantissa * 10n ** BigInt(scale - a.scale)
    + b.mantissa * 10n ** BigInt(scale - b.scale);
  const digits = (sum < 0n ? -sum : sum).toString().padStart(scale + 1, "0");
  const whole = digits.slice(0, digits.length - scale);
  const fraction = scale > 0 ? `.${digits.slice(digits.length - scale)}` : "";
  return `${sum < 0n ? "-" : ""}${whole}${fraction}`;
}
