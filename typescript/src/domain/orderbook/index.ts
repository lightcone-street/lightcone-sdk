import Decimal from "decimal.js";
import { PublicKey } from "@solana/web3.js";
import type { OrderBookId, PubkeyStr } from "../../shared";
import type { OrderbookDecimals } from "../../shared/scaling";
import type { ConditionalToken } from "../market";

export * from "./aggregation";
export * from "./client";
export * from "./wire";
export * from "./state";
export * from "./ticker";
export { orderBookPairFromWire } from "./convert";

export interface OrderBookPair {
  id: number;
  marketPubkey: PubkeyStr;
  orderbookId: OrderBookId;
  base: ConditionalToken;
  quote: ConditionalToken;
  outcomeIndex: number;
  tickSize: number;
  totalBids: number;
  totalAsks: number;
  lastTradePrice?: string;
  lastTradeTime?: Date;
  active: boolean;
}

export enum ImpactDirection {
  Negative = "negative",
  Zero = "zero",
  Positive = "positive",
}

export interface OutcomeImpact {
  direction: ImpactDirection;
  pct: number;
  dollar: string;
}

export function impactSign(direction: ImpactDirection): string {
  switch (direction) {
    case ImpactDirection.Negative:
      return "-";
    case ImpactDirection.Zero:
      return "";
    case ImpactDirection.Positive:
      return "+";
  }
}

export function impactPct(depositPrice: Decimal, conditionalPrice: Decimal): [number, string] {
  if (depositPrice.isZero() || conditionalPrice.isZero()) {
    return [0, ""];
  }

  const value = conditionalPrice.minus(depositPrice).div(depositPrice).mul(100);
  return [value.toNumber(), value.greaterThan(0) ? "+" : ""];
}

export function impact(
  depositAssetPrice: Decimal,
  conditionalPrice: Decimal
): OutcomeImpact {
  if (depositAssetPrice.isZero()) {
    return { direction: ImpactDirection.Zero, pct: 0, dollar: "0" };
  }

  const dollarDelta = conditionalPrice.minus(depositAssetPrice);
  const pctDecimal = dollarDelta.div(depositAssetPrice).mul(100);
  const pct = pctDecimal.toNumber();
  const direction = dollarDelta.greaterThan(0)
    ? ImpactDirection.Positive
    : dollarDelta.lessThan(0)
      ? ImpactDirection.Negative
      : ImpactDirection.Zero;

  return {
    direction,
    pct: Math.abs(pct),
    dollar: dollarDelta.abs().toString(),
  };
}

/**
 * Derive scaling decimals from an orderbook pair's token metadata.
 *
 * No REST call needed — decimals are computed from the base/quote token objects.
 */
export function orderbookDecimals(pair: OrderBookPair): OrderbookDecimals {
  const baseDecimals = pair.base.decimals;
  const quoteDecimals = pair.quote.decimals;
  return {
    orderbookId: pair.orderbookId,
    baseDecimals,
    quoteDecimals,
    priceDecimals: Math.max(0, 6 + quoteDecimals - baseDecimals),
    tickSize: BigInt(Math.max(pair.tickSize, 0)),
  };
}

/** Return the market as a `PublicKey`. */
export function orderBookMarket(pair: OrderBookPair): PublicKey {
  return new PublicKey(pair.marketPubkey);
}

/** Return the base conditional-token mint as a `PublicKey`. */
export function orderBookBaseMint(pair: OrderBookPair): PublicKey {
  return new PublicKey(pair.base.pubkey);
}

/** Return the quote conditional-token mint as a `PublicKey`. */
export function orderBookQuoteMint(pair: OrderBookPair): PublicKey {
  return new PublicKey(pair.quote.pubkey);
}

export class OrderBookValidationError extends Error {
  readonly orderbookId: string;
  readonly details: string[];

  constructor(orderbookId: string, details: string[]) {
    super(`OrderBook validation errors (${orderbookId}): ${details.join("; ")}`);
    this.name = "OrderBookValidationError";
    this.orderbookId = orderbookId;
    this.details = details;
  }
}
