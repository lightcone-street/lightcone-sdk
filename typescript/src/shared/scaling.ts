import Decimal from "decimal.js";
import { OrderSide } from "../program/types";

/**
 * Orderbook decimal configuration
 */
export interface OrderbookDecimals {
  orderbookId: string;
  baseDecimals: number;
  quoteDecimals: number;
  priceDecimals: number;
}

/**
 * Scaled amounts result
 */
export interface ScaledAmounts {
  makerAmount: bigint;
  takerAmount: bigint;
}

/**
 * Error thrown during price/size scaling
 */
export class ScalingError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ScalingError";
  }

  static zeroPriceOrSize(): ScalingError {
    return new ScalingError("Price and size must be non-zero");
  }

  static negativeValue(field: string): ScalingError {
    return new ScalingError(`${field} must be positive`);
  }

  static overflow(field: string): ScalingError {
    return new ScalingError(`${field} exceeds u64 range`);
  }
}

const U64_MAX = (1n << 64n) - 1n;

/**
 * Scale price and size to maker_amount and taker_amount using exact decimal arithmetic.
 *
 * For BID orders (buying base with quote):
 *   - maker_amount = price * size * 10^quoteDecimals (what maker gives in quote tokens)
 *   - taker_amount = size * 10^baseDecimals (what maker receives in base tokens)
 *
 * For ASK orders (selling base for quote):
 *   - maker_amount = size * 10^baseDecimals (what maker gives in base tokens)
 *   - taker_amount = price * size * 10^quoteDecimals (what maker receives in quote tokens)
 *
 * @param priceStr - Price as a decimal string (e.g., "0.75")
 * @param sizeStr - Size as a decimal string (e.g., "100")
 * @param side - Order side (BID or ASK)
 * @param decimals - Orderbook decimal configuration
 * @returns Scaled maker_amount and taker_amount as bigints
 */
export function scalePriceSize(
  priceStr: string,
  sizeStr: string,
  side: OrderSide,
  decimals: OrderbookDecimals
): ScaledAmounts {
  const price = new Decimal(priceStr);
  const size = new Decimal(sizeStr);

  if (price.isZero() || size.isZero()) {
    throw ScalingError.zeroPriceOrSize();
  }
  if (price.isNegative()) {
    throw ScalingError.negativeValue("price");
  }
  if (size.isNegative()) {
    throw ScalingError.negativeValue("size");
  }

  const baseScale = new Decimal(10).pow(decimals.baseDecimals);
  const quoteScale = new Decimal(10).pow(decimals.quoteDecimals);

  const baseAmount = size.mul(baseScale).floor();
  const quoteAmount = price.mul(size).mul(quoteScale).floor();

  const baseAmountBigInt = BigInt(baseAmount.toFixed(0));
  const quoteAmountBigInt = BigInt(quoteAmount.toFixed(0));

  if (baseAmountBigInt > U64_MAX) {
    throw ScalingError.overflow("base amount");
  }
  if (quoteAmountBigInt > U64_MAX) {
    throw ScalingError.overflow("quote amount");
  }

  if (side === OrderSide.BID) {
    return {
      makerAmount: quoteAmountBigInt,
      takerAmount: baseAmountBigInt,
    };
  } else {
    return {
      makerAmount: baseAmountBigInt,
      takerAmount: quoteAmountBigInt,
    };
  }
}
