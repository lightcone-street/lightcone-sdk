import Decimal from "decimal.js";

import { SdkError } from "../error";
import type { OrderBookPair } from "../domain/orderbook";
import type { ConditionalToken } from "../domain/market/tokens";
import { OrderSide } from "../program/types";

export type Branded<T, Brand extends string> = T & { readonly __brand: Brand };

export type OrderBookId = Branded<string, "OrderBookId">;
export type PubkeyStr = Branded<string, "PubkeyStr">;

export function asOrderBookId(value: string): OrderBookId {
  return value as OrderBookId;
}

export function asPubkeyStr(value: string): PubkeyStr {
  return value as PubkeyStr;
}

export enum Side {
  Bid = "bid",
  Ask = "ask",
}

export function parseSide(value: string): Side {
  if (value === "bid" || value === "buy") return Side.Bid;
  if (value === "ask" || value === "sell") return Side.Ask;
  throw SdkError.validation(`Invalid side: ${value}`);
}

export function sideLabel(side: Side): "Buy" | "Sell" {
  return side === Side.Bid ? "Buy" : "Sell";
}

/**
 * The denomination of the asset this side spends (Bid spends quote, Ask spends
 * base). Also a trade form's default display denomination.
 */
export function spendDenominator(side: Side): Denominator {
  return side === Side.Bid ? Denominator.Quote : Denominator.Base;
}

/**
 * The denomination of the asset this side receives (Bid receives base, Ask
 * receives quote).
 */
export function receiveDenominator(side: Side): Denominator {
  return side === Side.Bid ? Denominator.Base : Denominator.Quote;
}

/**
 * The price to submit with a market (IOC) order: the worst book fill price
 * padded by the impact-protection percentage in the direction that lets the
 * order fill.
 *
 * Returns null unless both inputs are positive.
 */
export function applyImpactProtection(
  side: Side,
  worstFillPrice: Decimal,
  protectionPercent: Decimal
): Decimal | null {
  if (worstFillPrice.lte(0) || protectionPercent.lte(0)) {
    return null;
  }
  const factor = protectionPercent.div(100);
  return side === Side.Bid
    ? worstFillPrice.mul(factor.add(1)) // buying: willing to pay more
    : worstFillPrice.mul(new Decimal(1).sub(factor)); // selling: willing to receive less
}

/** Map a wire/domain `Side` onto the on-chain `OrderSide`. */
export function toOrderSide(side: Side): OrderSide {
  return side === Side.Bid ? OrderSide.BID : OrderSide.ASK;
}

export enum Denominator {
  Base = "Base",
  Quote = "Quote",
}

export function allDenominators(): Denominator[] {
  return [Denominator.Quote, Denominator.Base];
}

/** The conditional token this denomination refers to on `pair`. */
export function denominatorToken(denominator: Denominator, pair: OrderBookPair): ConditionalToken {
  return denominator === Denominator.Base ? pair.base : pair.quote;
}

export function denominatorSymbol(denominator: Denominator, pair: OrderBookPair): string {
  return denominatorToken(denominator, pair).symbol;
}

export function denominatorDepositSymbol(denominator: Denominator, pair: OrderBookPair): string {
  return denominatorToken(denominator, pair).depositSymbol;
}

/**
 * Convert `amount` from one denomination into another at the given price
 * (quote per one base).
 *
 * Same-denomination conversion is the identity and never needs a price;
 * crossing denominations requires a positive price — null otherwise.
 */
export function convertDenomination(
  from: Denominator,
  to: Denominator,
  amount: Decimal,
  basePriceInQuote: Decimal
): Decimal | null {
  if (from === to) {
    return amount;
  }
  if (basePriceInQuote.lte(0)) {
    return null;
  }
  return from === Denominator.Base
    ? amount.mul(basePriceInQuote)
    : amount.div(basePriceInQuote);
}

export enum TimeInForce {
  Gtc = "GTC",
  Ioc = "IOC",
  Fok = "FOK",
  Alo = "ALO",
}

export enum TriggerType {
  TakeProfit = "TP",
  StopLoss = "SL",
}

export enum TriggerStatus {
  Created = "created",
  Triggered = "triggered",
  Failed = "failed",
  Expired = "expired",
  Invalidated = "invalidated",
}

export enum OrderUpdateType {
  Placement = "PLACEMENT",
  Update = "UPDATE",
  Cancellation = "CANCELLATION",
}

export enum TriggerUpdateType {
  Created = "CREATED",
  Triggered = "TRIGGERED",
  Failed = "FAILED",
  Expired = "EXPIRED",
  Invalidated = "INVALIDATED",
}

export enum TriggerResultStatus {
  Filled = "filled",
  Accepted = "accepted",
  Rejected = "rejected",
}

export enum DepositSource {
  Global = "global",
  Market = "market",
}

export enum Resolution {
  Minute1 = "1m",
  Minute5 = "5m",
  Minute15 = "15m",
  Hour1 = "1h",
  Hour4 = "4h",
  Day1 = "1d",
}

export function parseResolution(value: string): Resolution {
  switch (value) {
    case "1m":
      return Resolution.Minute1;
    case "5m":
      return Resolution.Minute5;
    case "15m":
      return Resolution.Minute15;
    case "1h":
      return Resolution.Hour1;
    case "4h":
      return Resolution.Hour4;
    case "1d":
      return Resolution.Day1;
    default:
      throw SdkError.validation(`Invalid resolution: ${value}`);
  }
}

export function resolutionSeconds(resolution: Resolution): number {
  switch (resolution) {
    case Resolution.Minute1:
      return 60;
    case Resolution.Minute5:
      return 300;
    case Resolution.Minute15:
      return 900;
    case Resolution.Hour1:
      return 3600;
    case Resolution.Hour4:
      return 14400;
    case Resolution.Day1:
      return 86400;
  }
}

export function deriveOrderbookId(baseToken: string, quoteToken: string): OrderBookId {
  return `${baseToken.slice(0, 8)}_${quoteToken.slice(0, 8)}` as OrderBookId;
}

export interface SubmitOrderRequest {
  maker: string;
  nonce: number;
  salt: number;
  market_pubkey: string;
  base_token: string;
  quote_token: string;
  side: number;
  /** u64 amount — validated to fit in Number.MAX_SAFE_INTEGER at construction time */
  amount_in: number;
  /** u64 amount — validated to fit in Number.MAX_SAFE_INTEGER at construction time */
  amount_out: number;
  expiration: number;
  signature: string;
  orderbook_id: string;
  tif?: TimeInForce;
  trigger_price?: number;
  trigger_type?: TriggerType;
  deposit_source?: DepositSource;
}
