import Decimal from "decimal.js";
import { OrderSide } from "../program/types";

export interface OrderbookDecimals {
  orderbookId: string;
  baseDecimals: number;
  quoteDecimals: number;
  priceDecimals: number;
  tickSize?: bigint;
}

export interface ScaledAmounts {
  amountIn: bigint;
  amountOut: bigint;
  // Backward-compatible aliases used by v1 builder code.
  makerAmount: bigint;
  takerAmount: bigint;
}

const U64_MAX = (1n << 64n) - 1n;

export class ScalingError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ScalingError";
  }

  static nonPositivePrice(value: string): ScalingError {
    return new ScalingError(`Price must be positive, got ${value}`);
  }

  static nonPositiveSize(value: string): ScalingError {
    return new ScalingError(`Size must be positive, got ${value}`);
  }

  static overflow(context: string): ScalingError {
    return new ScalingError(`Overflow: ${context}`);
  }

  static zeroAmount(): ScalingError {
    return new ScalingError("Computed amount is zero");
  }

  static fractionalAmount(value: string): ScalingError {
    return new ScalingError(`Fractional lamports not allowed: ${value}`);
  }

  static invalidDecimal(input: string, reason: string): ScalingError {
    return new ScalingError(`Invalid decimal '${input}': ${reason}`);
  }
}

export function alignPriceToTick(price: Decimal, decimals: OrderbookDecimals): Decimal {
  const tickSize = decimals.tickSize ?? 0n;
  if (tickSize <= 1n) {
    return price;
  }

  const quoteMultiplier = new Decimal(10).pow(decimals.quoteDecimals);
  const tick = new Decimal(tickSize.toString());
  const lamports = price.mul(quoteMultiplier).trunc();
  const aligned = lamports.div(tick).trunc().mul(tick);
  return aligned.div(quoteMultiplier);
}

export function scalePriceSize(
  priceInput: string | Decimal,
  sizeInput: string | Decimal,
  side: OrderSide,
  decimals: OrderbookDecimals
): ScaledAmounts {
  const price = normalizeDecimal(priceInput, "price");
  const size = normalizeDecimal(sizeInput, "size");

  if (price.lte(0)) {
    throw ScalingError.nonPositivePrice(price.toString());
  }
  if (size.lte(0)) {
    throw ScalingError.nonPositiveSize(size.toString());
  }

  const baseMultiplier = new Decimal(10).pow(decimals.baseDecimals);
  const quoteMultiplier = new Decimal(10).pow(decimals.quoteDecimals);

  const baseLamports = size.mul(baseMultiplier);
  const quoteLamports = price.mul(size).mul(quoteMultiplier);

  assertWhole(baseLamports, "base_lamports");
  assertWhole(quoteLamports, "quote_lamports");

  const base = toU64(baseLamports, "base_lamports");
  const quote = toU64(quoteLamports, "quote_lamports");

  if (base === 0n || quote === 0n) {
    throw ScalingError.zeroAmount();
  }

  if (side === OrderSide.BID) {
    return {
      amountIn: quote,
      amountOut: base,
      makerAmount: quote,
      takerAmount: base,
    };
  }

  return {
    amountIn: base,
    amountOut: quote,
    makerAmount: base,
    takerAmount: quote,
  };
}

function normalizeDecimal(value: string | Decimal, field: string): Decimal {
  if (value instanceof Decimal) {
    return value;
  }

  try {
    return new Decimal(value);
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    throw ScalingError.invalidDecimal(value, reason || field);
  }
}

function assertWhole(value: Decimal, label: string): void {
  if (!value.isInteger()) {
    throw ScalingError.fractionalAmount(`${label} = ${value.toString()}`);
  }
}

function toU64(value: Decimal, label: string): bigint {
  const bigint = BigInt(value.toFixed(0));
  if (bigint < 0n || bigint > U64_MAX) {
    throw ScalingError.overflow(`${label} ${value.toString()} does not fit in u64`);
  }
  return bigint;
}

// Legacy aliases kept for existing integrations.
export interface LegacyScaledAmounts {
  makerAmount: bigint;
  takerAmount: bigint;
}

export function scalePriceSizeLegacy(
  price: string,
  size: string,
  side: OrderSide,
  decimals: OrderbookDecimals
): LegacyScaledAmounts {
  const scaled = scalePriceSize(price, size, side, decimals);
  return {
    makerAmount: scaled.amountIn,
    takerAmount: scaled.amountOut,
  };
}
