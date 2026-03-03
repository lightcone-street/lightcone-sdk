import type { MarketEvent } from "../domain/market";
import type { AuthUpdate, UserUpdate } from "../domain/order";
import type { OrderBook, WsTickerData } from "../domain/orderbook";
import type { PriceHistory } from "../domain/price_history";
import type { WsTrade } from "../domain/trade";
import { DEFAULT_WS_URL } from "../network";
import type { OrderBookId, PubkeyStr, Resolution } from "../shared";

export * from "./client";
export * from "./subscriptions";

export type MessageOut =
  | { method: "subscribe"; params: import("./subscriptions").SubscribeParams }
  | { method: "unsubscribe"; params: import("./subscriptions").UnsubscribeParams }
  | { method: "ping" };

export type MessageIn =
  | { type: "book_update"; version: number; data: OrderBook }
  | { type: "pong"; version: number; data: Record<string, never> }
  | { type: "user"; version: number; data: UserUpdate }
  | { type: "error"; version: number; data: WsError }
  | { type: "price_history"; version: number; data: PriceHistory }
  | { type: "trades"; version: number; data: WsTrade }
  | { type: "auth"; version: number; data: AuthUpdate }
  | { type: "ticker"; version: number; data: WsTickerData }
  | { type: "market"; version: number; data: MarketEvent };

export type Kind = MessageIn;

export interface WsError {
  error: string;
  code?: string;
  orderbook_id?: string;
  wallet_address?: string;
  hint?: string;
  details?: string;
}

export type WsEvent =
  | { type: "Message"; message: MessageIn }
  | { type: "Connected" }
  | { type: "Disconnected"; code?: number; reason: string }
  | { type: "Error"; error: string }
  | { type: "MaxReconnectReached" };

export interface WsConfig {
  url: string;
  reconnect: boolean;
  maxReconnectAttempts: number;
  baseReconnectDelayMs: number;
  pingIntervalMs: number;
  pongTimeoutMs: number;
}

export const WS_DEFAULT_CONFIG: WsConfig = {
  url: DEFAULT_WS_URL,
  reconnect: true,
  maxReconnectAttempts: 10,
  baseReconnectDelayMs: 1_000,
  pingIntervalMs: 30_000,
  pongTimeoutMs: 1_000,
};

export enum ReadyState {
  Connecting = 0,
  Open = 1,
  Closing = 2,
  Closed = 3,
}

export function readyStateFrom(value: number): ReadyState {
  switch (value) {
    case 0:
      return ReadyState.Connecting;
    case 1:
      return ReadyState.Open;
    case 2:
      return ReadyState.Closing;
    case 3:
    default:
      return ReadyState.Closed;
  }
}

export function ping(): MessageOut {
  return { method: "ping" };
}

export function subscribeBooks(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "book_update",
      orderbook_ids: orderbookIds,
    },
  };
}

export function unsubscribeBooks(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "book_update",
      orderbook_ids: orderbookIds,
    },
  };
}

export function subscribeTrades(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "trades",
      orderbook_ids: orderbookIds,
    },
  };
}

export function unsubscribeTrades(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "trades",
      orderbook_ids: orderbookIds,
    },
  };
}

export function subscribeUser(walletAddress: PubkeyStr): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "user",
      wallet_address: walletAddress,
    },
  };
}

export function unsubscribeUser(walletAddress: PubkeyStr): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "user",
      wallet_address: walletAddress,
    },
  };
}

export function subscribePriceHistory(orderbookId: OrderBookId, resolution: Resolution): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "price_history",
      orderbook_id: orderbookId,
      resolution,
      include_ohlcv: false,
    },
  };
}

export function unsubscribePriceHistory(orderbookId: OrderBookId, resolution: Resolution): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "price_history",
      orderbook_id: orderbookId,
      resolution,
    },
  };
}

export function subscribeTicker(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "ticker",
      orderbook_ids: orderbookIds,
    },
  };
}

export function unsubscribeTicker(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "ticker",
      orderbook_ids: orderbookIds,
    },
  };
}

export function subscribeMarket(marketPubkey: PubkeyStr): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "market",
      market_pubkey: marketPubkey,
    },
  };
}

export function unsubscribeMarket(marketPubkey: PubkeyStr): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "market",
      market_pubkey: marketPubkey,
    },
  };
}

export function parseMessageIn(input: string): MessageIn {
  return JSON.parse(input) as MessageIn;
}
