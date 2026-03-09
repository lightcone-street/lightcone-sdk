import type { OrderBookId, Side } from "../../shared";

export interface OrderbookResponse {
  id: number;
  market_pubkey: string;
  orderbook_id: string;
  base_token: string;
  quote_token: string;
  outcome_index?: number;
  tick_size: number;
  total_bids: number;
  total_asks: number;
  last_trade_price?: string;
  last_trade_time?: string;
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface OrderbooksResponse {
  orderbooks: OrderbookResponse[];
  total: number;
}

export interface RestBookLevel {
  price: string;
  size: string;
  orders?: number;
}

export interface OrderbookDepthResponse {
  orderbook_id: OrderBookId;
  market_pubkey?: string;
  best_bid?: string;
  best_ask?: string;
  spread?: string;
  tick_size?: string;
  bids: RestBookLevel[];
  asks: RestBookLevel[];
}

export interface DecimalsResponse {
  orderbook_id: string;
  base_decimals: number;
  quote_decimals: number;
  price_decimals: number;
}

export interface WsBookLevel {
  side: Side;
  price: string;
  size: string;
}

export interface OrderBook {
  orderbook_id: OrderBookId;
  is_snapshot?: boolean;
  seq?: number;
  timestamp?: string;
  bids: WsBookLevel[];
  asks: WsBookLevel[];
}

export interface WsTickerData {
  orderbook_id: OrderBookId;
  best_bid?: string;
  best_ask?: string;
  mid?: string;
  timestamp?: string;
}
