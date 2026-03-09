import Decimal from "decimal.js";
import type { OrderBookId, PubkeyStr } from "../../shared";
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
