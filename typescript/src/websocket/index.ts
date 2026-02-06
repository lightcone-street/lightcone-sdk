/**
 * WebSocket client module for Lightcone.
 *
 * This module provides real-time data streaming functionality for
 * live orderbook updates, trade notifications, and market events.
 *
 * @example
 * ```typescript
 * import { websocket } from "@lightcone/sdk";
 *
 * const client = await websocket.LightconeWebSocketClient.connectDefault();
 *
 * client.on((event) => {
 *   if (event.type === "BookUpdate") {
 *     const book = client.getOrderbook(event.orderbookId);
 *     console.log("Best bid:", book?.bestBid());
 *   }
 * });
 *
 * await client.subscribeBookUpdates(["market1:ob1"]);
 * ```
 *
 * @module websocket
 */

// Client
export {
  LightconeWebSocketClient,
  DEFAULT_WS_URL,
} from "./client";
export type {
  WebSocketConfig,
  ConnectionState,
  EventCallback,
} from "./client";

// Error types
export { WebSocketError } from "./error";
export type { WebSocketErrorVariant, WsResult } from "./error";

// Types
export type {
  WsRequest,
  SubscribeParams,
  BookUpdateParams,
  TradesParams,
  UserParams,
  PriceHistoryParams,
  MarketParams,
  WsServerMessage,
  AuthData,
  WsMessage,
  BookUpdateData,
  PriceLevel,
  TradeData,
  UserEventData,
  Order,
  OrderUpdate,
  Balance,
  OutcomeBalance,
  BalanceEntry,
  PriceHistoryData,
  Candle,
  MarketEventData,
  MarketEventType,
  ErrorData,
  ErrorCode,
  PongData,
  WsEvent,
  MessageType,
  Side,
  PriceLevelSide,
} from "./types";

export {
  createSubscribeRequest,
  createUnsubscribeRequest,
  createPingRequest,
  bookUpdateParams,
  tradesParams,
  userParams,
  priceHistoryParams,
  marketParams,
  toCandle,
  parseMarketEventType,
  parseErrorCode,
  parseMessageType,
  parseWsMessage,
  parseSide,
  sideToNumber,
  parsePriceLevelSide,
} from "./types";

// State management
export { LocalOrderbook, UserState, PriceHistory, PriceHistoryKey } from "./state";

// Subscription management
export { SubscriptionManager, subscriptionToParams, subscriptionType } from "./subscriptions";
export type { Subscription } from "./subscriptions";

// Message handlers
export { MessageHandler } from "./handlers";

// Authentication
export {
  AUTH_API_URL,
  authenticate,
  generateSigninMessage,
  generateSigninMessageWithTimestamp,
  signMessage,
} from "../auth";
export type { AuthCredentials } from "../auth";
