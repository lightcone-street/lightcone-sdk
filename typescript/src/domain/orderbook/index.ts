import Decimal from "decimal.js";
import { PublicKey } from "@solana/web3.js";
import type { OrderBookId, PubkeyStr } from "../../shared";
import type { OrderbookDecimals } from "../../shared/scaling";
import type { ConditionalToken } from "../market";

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

export interface OutcomeImpact {
  sign: string;
  pct: number;
  dollar: string;
  isPositive: boolean;
}

export function impactPct(pair: OrderBookPair, depositPrice: Decimal): [number, string] {
  if (depositPrice.isZero() || !pair.lastTradePrice) {
    return [0, ""];
  }

  const conditional = new Decimal(pair.lastTradePrice);
  const value = conditional.minus(depositPrice).div(depositPrice).mul(100);
  return [value.toNumber(), value.greaterThan(0) ? "+" : ""];
}

export function impact(
  depositAssetPrice: Decimal,
  conditionalPrice: Decimal
): OutcomeImpact {
  if (depositAssetPrice.isZero()) {
    return { sign: "", pct: 0, dollar: "0", isPositive: false };
  }

  const pctDecimal = conditionalPrice.minus(depositAssetPrice).div(depositAssetPrice).mul(100);
  const pct = pctDecimal.toNumber();
  const dollar = conditionalPrice.minus(depositAssetPrice).abs().toString();

  return {
    sign: pct > 0 ? "+" : "-",
    pct: Math.abs(pct),
    dollar,
    isPositive: pct > 0,
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
