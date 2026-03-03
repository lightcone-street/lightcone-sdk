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
  | { event_type: "snapshot"; payload: PriceHistorySnapshot }
  | { event_type: "update"; payload: PriceHistoryUpdate }
  | { event_type: "heartbeat"; payload: PriceHistoryHeartbeat };
