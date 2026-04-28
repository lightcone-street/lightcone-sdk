import type { MarketEvent } from "../domain/market";
import { normalizeUserUpdate, type AuthUpdate, type UserUpdate } from "../domain/order/wire";
import type { OrderBook, WsTickerData } from "../domain/orderbook";
import type {
  DepositAssetPriceEvent,
  DepositPrice,
  PriceHistory,
} from "../domain/price_history";
import type { WsTrade } from "../domain/trade";
import { WsError as WsErrorClass } from "../error";
import { LightconeEnv, wsUrl } from "../env";
import type { OrderBookId, PubkeyStr, Resolution } from "../shared";

export * from "./client.node";
export * from "./subscriptions";
export type { IWsClient } from "./types";

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
  | { type: "market"; version: number; data: MarketEvent }
  | { type: "deposit_price"; version: number; data: DepositPrice }
  | { type: "deposit_asset_price"; version: number; data: DepositAssetPriceEvent };

export type Kind = MessageIn;

export interface WsError {
  error: string;
  code?: string;
  orderbook_id?: string;
  wallet_address?: string;
  deposit_asset?: string;
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
  url: wsUrl(LightconeEnv.Prod),
  reconnect: true,
  maxReconnectAttempts: 10,
  baseReconnectDelayMs: 1_000,
  pingIntervalMs: 30_000,
  pongTimeoutMs: 10_000,
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

export function subscribeDepositPrice(depositAsset: string, resolution: Resolution): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "deposit_price",
      deposit_asset: depositAsset,
      resolution,
    },
  };
}

export function unsubscribeDepositPrice(depositAsset: string, resolution: Resolution): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "deposit_price",
      deposit_asset: depositAsset,
      resolution,
    },
  };
}

/**
 * Subscribe to the live spot price for one deposit asset (snapshot +
 * per-asset price ticks). Distinct from `subscribeDepositPrice`, which
 * carries OHLCV candles per resolution.
 */
export function subscribeDepositAssetPrice(depositAsset: string): MessageOut {
  return {
    method: "subscribe",
    params: { type: "deposit_asset_price", deposit_asset: depositAsset },
  };
}

export function unsubscribeDepositAssetPrice(depositAsset: string): MessageOut {
  return {
    method: "unsubscribe",
    params: { type: "deposit_asset_price", deposit_asset: depositAsset },
  };
}

const VALID_MESSAGE_TYPES = new Set([
  "book_update",
  "pong",
  "user",
  "error",
  "price_history",
  "trades",
  "auth",
  "ticker",
  "market",
  "deposit_price",
  "deposit_asset_price",
]);

export function parseMessageIn(input: string): MessageIn {
  const parsed: unknown = JSON.parse(input);
  if (typeof parsed !== "object" || parsed === null || !("type" in parsed)) {
    throw new WsErrorClass("ProtocolError", `Invalid WS message: missing "type" field`);
  }
  const obj = parsed as Record<string, unknown>;
  if (typeof obj.type !== "string" || !VALID_MESSAGE_TYPES.has(obj.type)) {
    throw new WsErrorClass("ProtocolError", `Invalid WS message type: "${String(obj.type)}"`);
  }
  if (!("version" in obj) || typeof obj.version !== "number") {
    throw new WsErrorClass("ProtocolError", `Invalid WS message: missing or invalid "version" field`);
  }
  if (!("data" in obj) || typeof obj.data !== "object" || obj.data === null) {
    throw new WsErrorClass("ProtocolError", `Invalid WS message: missing or invalid "data" field`);
  }
  const message = parsed as MessageIn;
  if (message.type === "user") {
    return {
      ...message,
      data: normalizeUserUpdate(message.data as Parameters<typeof normalizeUserUpdate>[0]),
    };
  }
  return message;
}
