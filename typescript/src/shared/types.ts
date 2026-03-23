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
  throw new Error(`Invalid side: ${value}`);
}

export function sideLabel(side: Side): "Buy" | "Sell" {
  return side === Side.Bid ? "Buy" : "Sell";
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
