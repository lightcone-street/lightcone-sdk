import type { OrderBookId, Resolution } from "../../shared";

export interface PriceCandle {
  t: number;
  m?: string;
  o?: string;
  h?: string;
  l?: string;
  c?: string;
  v?: string;
  bb?: string;
  ba?: string;
}

export interface PriceHistorySnapshot {
  orderbook_id: OrderBookId;
  resolution: Resolution;
  prices: PriceCandle[];
  last_timestamp?: number;
  server_time?: number;
  include_ohlcv?: boolean;
}

export interface PriceHistoryUpdate {
  orderbook_id: OrderBookId;
  resolution: Resolution;
  t: number;
  m?: string;
  o?: string;
  h?: string;
  l?: string;
  c?: string;
  v?: string;
  bb?: string;
  ba?: string;
}

export interface PriceHistoryHeartbeat {
  server_time: number;
  last_processed?: number;
}

export type PriceHistory =
  | ({ event_type: "snapshot" } & PriceHistorySnapshot)
  | ({ event_type: "update" } & PriceHistoryUpdate)
  | ({ event_type: "heartbeat" } & PriceHistoryHeartbeat);

export interface PriceHistoryRestResponse {
  orderbook_id: string;
  resolution: string;
  include_ohlcv: boolean;
  prices: PriceCandle[];
  next_cursor: number | null;
  has_more: boolean;
  decimals: { price: number; volume: number };
}

export interface DepositPriceCandle {
  t: number;
  tc?: number;
  c: string;
}

export interface DepositPriceRestResponse {
  deposit_asset: string;
  binance_symbol: string;
  resolution: string;
  prices: DepositPriceCandle[];
  next_cursor: number | null;
  has_more: boolean;
}
