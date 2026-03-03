import type { OrderBookId, Side } from "../../shared";

export interface TradeResponse {
  id: number;
  orderbook_id: OrderBookId;
  taker_pubkey: string;
  maker_pubkey: string;
  side: Side;
  size: string;
  price: string;
  taker_fee?: string;
  maker_fee?: string;
  executed_at: number;
}

export interface TradesDecimals {
  price?: number;
  size?: number;
  fee?: number;
}

export interface TradesResponse {
  orderbook_id: OrderBookId;
  trades: TradeResponse[];
  next_cursor?: number;
  has_more?: boolean;
  decimals?: TradesDecimals;
}

export interface WsTrade {
  orderbook_id: OrderBookId;
  trade_id: string;
  timestamp: string;
  price: string;
  size: string;
  side: Side;
}
