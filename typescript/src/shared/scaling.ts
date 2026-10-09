import Decimal from "decimal.js";
import { OrderSide } from "../program/types";

export const PRICE_SCALE = 1_000_000n;
export const I64_MAX = (1n << 63n) - 1n;
/** Largest raw atom amount; the program and engine accept the full u64 range. */
export const U64_MAX = (1n << 64n) - 1n;

/** Immutable engine admission rules. Quantum display strings are never arithmetic inputs. */
export interface TradingRules {
  baseSizeDecimals: number;
  maxPriceDecimals: number;
  maxPriceSignificantFigures: number;
  integerPricesAlwaysAllowed: boolean;
  priceQuantum: string;
  priceQuantumRaw: bigint;
  baseSizeQuantum: string;
  baseSizeQuantumRaw: bigint;
}

/** Complete rules required to build or preflight an order. */
export interface OrderbookRules {
  orderbookId: string;
  baseDecimals: number;
  quoteDecimals: number;
  priceDecimals: number;
  tradingRules: TradingRules;
}

export interface ScaledAmounts {
  amountIn: bigint;
  amountOut: bigint;
  priceRaw: bigint;
  baseAtoms: bigint;
  quoteAtoms: bigint;
}

export class ScalingError extends Error {
  readonly code?: string;

  constructor(message: string, code?: string) {
    super(message);
    this.name = "ScalingError";
    this.code = code;
  }

  static code(code: string, detail?: string): ScalingError {
    return new ScalingError(detail ? `${code}: ${detail}` : code, code);
  }
}

/** Parse a decimal value at an exact scale without rounding or truncation. */
export function exactScaledInteger(
  value: string | Decimal,
  decimals: number
): bigint {
  if (!Number.isSafeInteger(decimals) || decimals < 0) {
    throw new ScalingError(`Invalid decimal scale: ${decimals}`);
  }
  const source = value instanceof Decimal ? value.toString() : value;
  const input = source.trim();
  if (input.length === 0) throw invalidDecimal(source, "empty input");
  if (input.startsWith("-")) throw invalidDecimal(source, "negative values are not supported");

  const match = input.match(/^\+?(?:(\d+)(?:\.(\d*))?|\.(\d+))(?:[eE]([+-]?\d+))?$/);
  if (!match) throw invalidDecimal(source, "expected base-10 decimal syntax");
  const whole = match[1] ?? "";
  const fraction = match[2] ?? match[3] ?? "";
  const exponent = match[4] === undefined ? 0 : Number(match[4]);
  if (!Number.isSafeInteger(exponent) || Math.abs(exponent) > 10_000) {
    throw invalidDecimal(source, "invalid or unsupported exponent");
  }

  const coefficient = `${whole}${fraction}`.replace(/^0+/, "");
  if (coefficient.length === 0) return 0n;
  const shift = decimals + exponent - fraction.length;
  let digits: string;
  if (shift >= 0) {
    if (coefficient.length + shift > 10_000) {
      throw invalidDecimal(source, "scaled value is too large");
    }
    digits = coefficient + "0".repeat(shift);
  } else {
    const remove = -shift;
    if (remove > coefficient.length) {
      throw invalidDecimal(source, "cannot be represented exactly at this scale");
    }
    const removed = coefficient.slice(coefficient.length - remove);
    if (!/^0*$/.test(removed)) {
      throw invalidDecimal(source, "cannot be represented exactly at this scale");
    }
    digits = coefficient.slice(0, coefficient.length - remove) || "0";
  }
  return BigInt(digits);
}

function invalidDecimal(input: string, reason: string): ScalingError {
  return new ScalingError(`Invalid decimal '${input}': ${reason}`);
}

function significantDigits(value: bigint): number {
  let reduced = value;
  while (reduced > 0n && reduced % 10n === 0n) reduced /= 10n;
  return reduced.toString().length;
}

function validatePriceRaw(priceRaw: bigint, rules: OrderbookRules): bigint {
  if (priceRaw <= 0n || priceRaw > U64_MAX) {
    throw ScalingError.code("PRICE_OUT_OF_RANGE");
  }
  const humanScale = 10n ** BigInt(rules.priceDecimals);
  const integerPrice = priceRaw % humanScale === 0n;
  if (!integerPrice) {
    if (
      rules.tradingRules.priceQuantumRaw <= 0n ||
      priceRaw % rules.tradingRules.priceQuantumRaw !== 0n
    ) {
      throw ScalingError.code("INVALID_PRICE_DECIMALS");
    }
    if (significantDigits(priceRaw) > rules.tradingRules.maxPriceSignificantFigures) {
      throw ScalingError.code("INVALID_PRICE_SIGNIFICANT_FIGURES");
    }
  }
  return priceRaw;
}

function validateBaseAtoms(baseAtoms: bigint, rules: OrderbookRules): bigint {
  if (baseAtoms <= 0n) throw new ScalingError(`Size must be positive, got ${baseAtoms}`);
  if (
    rules.tradingRules.baseSizeQuantumRaw <= 0n ||
    baseAtoms % rules.tradingRules.baseSizeQuantumRaw !== 0n
  ) {
    throw ScalingError.code("INVALID_SIZE_DECIMALS");
  }
  assertAtomRange(baseAtoms, "base amount");
  return baseAtoms;
}

export function scalePriceSize(
  price: string | Decimal,
  size: string | Decimal,
  side: OrderSide,
  rules: OrderbookRules
): ScaledAmounts {
  const priceSource = price instanceof Decimal ? price.toString() : price;
  const sizeSource = size instanceof Decimal ? size.toString() : size;
  const priceRaw = exactScaledInteger(price, rules.priceDecimals);
  if (priceRaw <= 0n) throw new ScalingError(`Price must be positive, got ${priceSource}`);
  const baseAtoms = exactScaledInteger(size, rules.baseDecimals);
  if (baseAtoms <= 0n) throw new ScalingError(`Size must be positive, got ${sizeSource}`);
  validatePriceRaw(priceRaw, rules);
  validateBaseAtoms(baseAtoms, rules);

  const quoteNumerator = priceRaw * baseAtoms;
  if (quoteNumerator % PRICE_SCALE !== 0n) {
    throw ScalingError.code("PRICE_NOT_EXACTLY_REPRESENTABLE");
  }
  const quoteAtoms = quoteNumerator / PRICE_SCALE;
  assertAtomRange(quoteAtoms, "quote amount");
  const [amountIn, amountOut] =
    side === OrderSide.BID
      ? [quoteAtoms, baseAtoms]
      : side === OrderSide.ASK
        ? [baseAtoms, quoteAtoms]
        : (() => { throw new ScalingError("side must be BID or ASK"); })();
  return { amountIn, amountOut, priceRaw, baseAtoms, quoteAtoms };
}

/**
 * Preflight caller-supplied signed amounts against the same engine rules.
 *
 * `amountIn` and `amountOut` are raw atoms of the given and received mints.
 * Any nonzero u64 is in range.
 */
export function validateRawAmounts(
  amountIn: bigint,
  amountOut: bigint,
  side: OrderSide,
  rules: OrderbookRules
): ScaledAmounts {
  validateSignedFields(amountIn, amountOut);
  const [baseAtoms, quoteAtoms] =
    side === OrderSide.BID
      ? [amountOut, amountIn]
      : side === OrderSide.ASK
        ? [amountIn, amountOut]
        : (() => { throw new ScalingError("side must be BID or ASK"); })();
  validateBaseAtoms(baseAtoms, rules);
  const numerator = quoteAtoms * PRICE_SCALE;
  if (numerator % baseAtoms !== 0n) {
    throw ScalingError.code("PRICE_NOT_EXACTLY_REPRESENTABLE");
  }
  const priceRaw = numerator / baseAtoms;
  validatePriceRaw(priceRaw, rules);
  return { amountIn, amountOut, priceRaw, baseAtoms, quoteAtoms };
}

/** A nonzero raw atom amount within the u64 range. */
function assertAtomRange(value: bigint, field: string): void {
  if (value <= 0n || value > U64_MAX) {
    throw ScalingError.code("ORDER_FIELD_OUT_OF_RANGE", field);
  }
}

/**
 * Check the signed amount fields that the program and engine reject
 * outright. Amounts are raw token atoms and must be nonzero; the full u64
 * range is accepted.
 */
export function validateSignedFields(amountIn: bigint, amountOut: bigint): void {
  assertAtomRange(amountIn, "amount_in");
  assertAtomRange(amountOut, "amount_out");
}
