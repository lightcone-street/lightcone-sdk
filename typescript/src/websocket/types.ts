/**
 * Message types for the Lightcone WebSocket protocol.
 *
 * This module contains all request and response types for the WebSocket API.
 */

import type { WebSocketError } from "./error";

// ============================================================================
// REQUEST TYPES (Client → Server)
// ============================================================================

/**
 * Subscribe/Unsubscribe request wrapper.
 */
export interface WsRequest {
  method: string;
  params?: SubscribeParams;
}

/**
 * Subscription parameters (polymorphic).
 */
export type SubscribeParams =
  | BookUpdateParams
  | TradesParams
  | UserParams
  | PriceHistoryParams
  | MarketParams;

/**
 * Book update subscription params.
 */
export interface BookUpdateParams {
  type: "book_update";
  orderbook_ids: string[];
}

/**
 * Trades subscription params.
 */
export interface TradesParams {
  type: "trades";
  orderbook_ids: string[];
}

/**
 * User subscription params.
 */
export interface UserParams {
  type: "user";
  user: string;
}

/**
 * Price history subscription params.
 */
export interface PriceHistoryParams {
  type: "price_history";
  orderbook_id: string;
  resolution: string;
  include_ohlcv: boolean;
}

/**
 * Market subscription params.
 */
export interface MarketParams {
  type: "market";
  market_pubkey: string;
}

/**
 * Create a subscribe request.
 */
export function createSubscribeRequest(params: SubscribeParams): WsRequest {
  return { method: "subscribe", params };
}

/**
 * Create an unsubscribe request.
 */
export function createUnsubscribeRequest(params: SubscribeParams): WsRequest {
  return { method: "unsubscribe", params };
}

/**
 * Create a ping request.
 */
export function createPingRequest(): WsRequest {
  return { method: "ping" };
}

/**
 * Create book update subscription params.
 */
export function bookUpdateParams(orderbookIds: string[]): BookUpdateParams {
  return { type: "book_update", orderbook_ids: orderbookIds };
}

/**
 * Create trades subscription params.
 */
export function tradesParams(orderbookIds: string[]): TradesParams {
  return { type: "trades", orderbook_ids: orderbookIds };
}

/**
 * Create user subscription params.
 */
export function userParams(user: string): UserParams {
  return { type: "user", user };
}

/**
 * Create price history subscription params.
 */
export function priceHistoryParams(
  orderbookId: string,
  resolution: string,
  includeOhlcv: boolean
): PriceHistoryParams {
  return {
    type: "price_history",
    orderbook_id: orderbookId,
    resolution,
    include_ohlcv: includeOhlcv,
  };
}

/**
 * Create market subscription params.
 */
export function marketParams(marketPubkey: string): MarketParams {
  return { type: "market", market_pubkey: marketPubkey };
}

// ============================================================================
// RESPONSE TYPES (Server → Client)
// ============================================================================

/**
 * Raw message wrapper for initial parsing.
 */
export interface RawWsMessage {
  type: string;
  version: number;
  data: unknown;
}

/**
 * Generic WebSocket message wrapper.
 */
export interface WsMessage<T> {
  type: string;
  version: number;
  data: T;
}

// ============================================================================
// BOOK UPDATE TYPES
// ============================================================================

/**
 * Orderbook snapshot/delta data.
 */
export interface BookUpdateData {
  orderbook_id: string;
  timestamp: string;
  seq: number;
  bids: PriceLevel[];
  asks: PriceLevel[];
  is_snapshot: boolean;
  resync: boolean;
  message?: string;
}

/**
 * Price level in the orderbook.
 */
export interface PriceLevel {
  side: string;
  /** Price as decimal string (e.g., "0.500000") */
  price: string;
  /** Size as decimal string */
  size: string;
}

// ============================================================================
// TRADE TYPES
// ============================================================================

/**
 * Trade execution data.
 */
export interface TradeData {
  orderbook_id: string;
  /** Price as decimal string */
  price: string;
  /** Size as decimal string */
  size: string;
  side: string;
  timestamp: string;
  trade_id: string;
}

// ============================================================================
// USER EVENT TYPES
// ============================================================================

/**
 * User event data (snapshot, order_update, balance_update).
 */
export interface UserEventData {
  event_type: string;
  orders: Order[];
  balances: Record<string, BalanceEntry>;
  order?: OrderUpdate;
  balance?: Balance;
  market_pubkey?: string;
  orderbook_id?: string;
  deposit_mint?: string;
  timestamp?: string;
}

/**
 * User order from snapshot.
 */
export interface Order {
  order_hash: string;
  market_pubkey: string;
  orderbook_id: string;
  /** 0 = BUY, 1 = SELL */
  side: number;
  /** Maker amount as decimal string */
  maker_amount: string;
  /** Taker amount as decimal string */
  taker_amount: string;
  /** Remaining amount as decimal string */
  remaining: string;
  /** Filled amount as decimal string */
  filled: string;
  /** Price as decimal string */
  price: string;
  created_at: number;
  expiration: number;
}

/**
 * Order update from real-time event.
 */
export interface OrderUpdate {
  order_hash: string;
  /** Price as decimal string */
  price: string;
  /** Fill amount as decimal string */
  fill_amount: string;
  /** Remaining amount as decimal string */
  remaining: string;
  /** Filled amount as decimal string */
  filled: string;
  /** 0 = BUY, 1 = SELL */
  side: number;
  is_maker: boolean;
  created_at: number;
  balance?: Balance;
}

/**
 * Balance containing outcome balances.
 */
export interface Balance {
  outcomes: OutcomeBalance[];
}

/**
 * Individual outcome balance.
 */
export interface OutcomeBalance {
  outcome_index: number;
  mint: string;
  /** Idle balance as decimal string */
  idle: string;
  /** On-book balance as decimal string */
  on_book: string;
}

/**
 * Balance entry from user snapshot.
 */
export interface BalanceEntry {
  market_pubkey: string;
  deposit_mint: string;
  outcomes: OutcomeBalance[];
}

// ============================================================================
// PRICE HISTORY TYPES
// ============================================================================

/**
 * Price history data (snapshot, update, heartbeat).
 */
export interface PriceHistoryData {
  event_type: string;
  orderbook_id?: string;
  resolution?: string;
  include_ohlcv?: boolean;
  prices: Candle[];
  last_timestamp?: number;
  server_time?: number;
  last_processed?: number;
  // For updates (inline candle data)
  t?: number;
  o?: string;
  h?: string;
  l?: string;
  c?: string;
  v?: string;
  m?: string;
  bb?: string;
  ba?: string;
}

/**
 * Convert inline candle data to a Candle struct (for update events).
 */
export function toCandle(data: PriceHistoryData): Candle | undefined {
  if (data.t === undefined) return undefined;
  return {
    t: data.t,
    o: data.o,
    h: data.h,
    l: data.l,
    c: data.c,
    v: data.v,
    m: data.m,
    bb: data.bb,
    ba: data.ba,
  };
}

/**
 * OHLCV candle data.
 */
export interface Candle {
  /** Timestamp (Unix ms) */
  t: number;
  /** Open price as decimal string (null if no trades) */
  o?: string;
  /** High price as decimal string (null if no trades) */
  h?: string;
  /** Low price as decimal string (null if no trades) */
  l?: string;
  /** Close price as decimal string (null if no trades) */
  c?: string;
  /** Volume as decimal string (null if no trades) */
  v?: string;
  /** Midpoint: (best_bid + best_ask) / 2 as decimal string */
  m?: string;
  /** Best bid price as decimal string */
  bb?: string;
  /** Best ask price as decimal string */
  ba?: string;
}

// ============================================================================
// MARKET EVENT TYPES
// ============================================================================

/**
 * Market event data.
 */
export interface MarketEventData {
  /** Event type: "orderbook_created", "settled", "opened", "paused" */
  event_type: string;
  market_pubkey: string;
  orderbook_id?: string;
  timestamp: string;
}

/**
 * Market event types.
 */
export type MarketEventType =
  | "OrderbookCreated"
  | "Settled"
  | "Opened"
  | "Paused"
  | "Unknown";

/**
 * Parse market event type from string.
 */
export function parseMarketEventType(s: string): MarketEventType {
  switch (s) {
    case "orderbook_created":
      return "OrderbookCreated";
    case "settled":
      return "Settled";
    case "opened":
      return "Opened";
    case "paused":
      return "Paused";
    default:
      return "Unknown";
  }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/**
 * Error response from server.
 */
export interface ErrorData {
  error: string;
  code: string;
  orderbook_id?: string;
}

/**
 * Server error codes.
 */
export type ErrorCode =
  | "EngineUnavailable"
  | "InvalidJson"
  | "InvalidMethod"
  | "RateLimited"
  | "Unknown";

/**
 * Parse error code from string.
 */
export function parseErrorCode(s: string): ErrorCode {
  switch (s) {
    case "ENGINE_UNAVAILABLE":
      return "EngineUnavailable";
    case "INVALID_JSON":
      return "InvalidJson";
    case "INVALID_METHOD":
      return "InvalidMethod";
    case "RATE_LIMITED":
      return "RateLimited";
    default:
      return "Unknown";
  }
}

// ============================================================================
// PONG TYPE
// ============================================================================

/**
 * Pong response data (empty).
 */
export interface PongData {}

// ============================================================================
// CLIENT EVENTS
// ============================================================================

/**
 * Events emitted by the WebSocket client.
 */
export type WsEvent =
  | { type: "Connected" }
  | { type: "Disconnected"; reason: string }
  | { type: "BookUpdate"; orderbookId: string; isSnapshot: boolean }
  | { type: "Trade"; orderbookId: string; trade: TradeData }
  | { type: "UserUpdate"; eventType: string; user: string }
  | { type: "PriceUpdate"; orderbookId: string; resolution: string }
  | { type: "MarketEvent"; eventType: string; marketPubkey: string }
  | { type: "Error"; error: WebSocketError }
  | { type: "ResyncRequired"; orderbookId: string }
  | { type: "Pong" }
  | { type: "Reconnecting"; attempt: number };

// ============================================================================
// MESSAGE TYPE ENUM
// ============================================================================

/**
 * Enum for all possible server message types.
 */
export type MessageType =
  | "BookUpdate"
  | "Trades"
  | "User"
  | "PriceHistory"
  | "Market"
  | "Error"
  | "Pong"
  | "Unknown";

/**
 * Parse message type from string.
 */
export function parseMessageType(s: string): MessageType {
  switch (s) {
    case "book_update":
      return "BookUpdate";
    case "trades":
      return "Trades";
    case "user":
      return "User";
    case "price_history":
      return "PriceHistory";
    case "market":
      return "Market";
    case "error":
      return "Error";
    case "pong":
      return "Pong";
    default:
      return "Unknown";
  }
}

// ============================================================================
// SIDE HELPERS
// ============================================================================

/**
 * Order side enum for user events.
 */
export type Side = "Buy" | "Sell";

/**
 * Parse side from number.
 */
export function parseSide(value: number): Side {
  return value === 0 ? "Buy" : "Sell";
}

/**
 * Convert side to number.
 */
export function sideToNumber(side: Side): number {
  return side === "Buy" ? 0 : 1;
}

/**
 * Price level side (from orderbook updates).
 */
export type PriceLevelSide = "Bid" | "Ask";

/**
 * Parse price level side from string.
 */
export function parsePriceLevelSide(s: string): PriceLevelSide {
  return s === "bid" ? "Bid" : "Ask";
}
