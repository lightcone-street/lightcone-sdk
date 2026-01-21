/**
 * Message handlers for WebSocket events.
 *
 * Routes incoming messages to appropriate handlers and emits events.
 */

import { WebSocketError } from "./error";
import { LocalOrderbook, UserState, PriceHistory, PriceHistoryKey } from "./state";
import type {
  RawWsMessage,
  WsEvent,
  BookUpdateData,
  TradeData,
  UserEventData,
  PriceHistoryData,
  MarketEventData,
  ErrorData,
} from "./types";
import { parseMessageType } from "./types";

/**
 * Handles incoming WebSocket messages.
 */
export class MessageHandler {
  /** Local orderbook state */
  private orderbooks: Map<string, LocalOrderbook> = new Map();
  /** Local user state */
  private userStates: Map<string, UserState> = new Map();
  /** Price history state */
  private priceHistories: Map<string, PriceHistory> = new Map();

  /**
   * Handle an incoming message and return events.
   */
  handleMessage(text: string): WsEvent[] {
    // Parse the raw message first
    let rawMsg: RawWsMessage;
    try {
      rawMsg = JSON.parse(text);
    } catch (e) {
      console.warn("Failed to parse WebSocket message:", e);
      return [
        {
          type: "Error",
          error: WebSocketError.messageParseError(String(e)),
        },
      ];
    }

    // Route by message type
    const msgType = parseMessageType(rawMsg.type);
    switch (msgType) {
      case "BookUpdate":
        return this.handleBookUpdate(rawMsg);
      case "Trades":
        return this.handleTrade(rawMsg);
      case "User":
        return this.handleUserEvent(rawMsg);
      case "PriceHistory":
        return this.handlePriceHistory(rawMsg);
      case "Market":
        return this.handleMarketEvent(rawMsg);
      case "Error":
        return this.handleError(rawMsg);
      case "Pong":
        return [{ type: "Pong" }];
      case "Unknown":
        console.warn("Unknown message type:", rawMsg.type);
        return [];
    }
  }

  /**
   * Handle book update message.
   */
  private handleBookUpdate(rawMsg: RawWsMessage): WsEvent[] {
    const data = rawMsg.data as BookUpdateData;

    // Check for resync signal
    if (data.resync) {
      console.info("Resync required for orderbook:", data.orderbook_id);
      return [{ type: "ResyncRequired", orderbookId: data.orderbook_id }];
    }

    const orderbookId = data.orderbook_id;
    const isSnapshot = data.is_snapshot;

    // Update local state
    let book = this.orderbooks.get(orderbookId);
    if (!book) {
      book = new LocalOrderbook(orderbookId);
      this.orderbooks.set(orderbookId, book);
    }

    try {
      book.applyUpdate(data);
      return [{ type: "BookUpdate", orderbookId, isSnapshot }];
    } catch (e) {
      if (e instanceof WebSocketError && e.variant === "SequenceGap") {
        console.warn(
          `Sequence gap in orderbook ${orderbookId}:`,
          e.message
        );
        // Clear the orderbook state on sequence gap
        book.clear();
        return [{ type: "ResyncRequired", orderbookId }];
      }
      return [{ type: "Error", error: e as WebSocketError }];
    }
  }

  /**
   * Handle trade message.
   */
  private handleTrade(rawMsg: RawWsMessage): WsEvent[] {
    const data = rawMsg.data as TradeData;
    return [{ type: "Trade", orderbookId: data.orderbook_id, trade: data }];
  }

  /**
   * Handle user event message.
   */
  private handleUserEvent(rawMsg: RawWsMessage): WsEvent[] {
    const data = rawMsg.data as UserEventData;
    const eventType = data.event_type;

    // We need to determine the user key
    const user =
      data.orders[0]?.market_pubkey ||
      data.market_pubkey ||
      "unknown";

    // Update local state for all known user states
    for (const state of this.userStates.values()) {
      state.applyEvent(data);
    }

    return [{ type: "UserUpdate", eventType, user }];
  }

  /**
   * Handle price history message.
   */
  private handlePriceHistory(rawMsg: RawWsMessage): WsEvent[] {
    const data = rawMsg.data as PriceHistoryData;

    // Heartbeats don't have orderbook_id
    if (data.event_type === "heartbeat") {
      // Update all price histories with heartbeat
      for (const history of this.priceHistories.values()) {
        history.applyHeartbeat(data);
      }
      return [];
    }

    const orderbookId = data.orderbook_id;
    if (!orderbookId) {
      console.warn("Price history message missing orderbook_id");
      return [];
    }

    const resolution = data.resolution || "1m";
    const key = `${orderbookId}:${resolution}`;

    let history = this.priceHistories.get(key);
    if (history) {
      history.applyEvent(data);
    } else if (data.event_type === "snapshot") {
      // Create new history if this is a snapshot
      history = new PriceHistory(
        orderbookId,
        resolution,
        data.include_ohlcv || false
      );
      history.applyEvent(data);
      this.priceHistories.set(key, history);
    }

    return [{ type: "PriceUpdate", orderbookId, resolution }];
  }

  /**
   * Handle market event message.
   */
  private handleMarketEvent(rawMsg: RawWsMessage): WsEvent[] {
    const data = rawMsg.data as MarketEventData;
    return [
      {
        type: "MarketEvent",
        eventType: data.event_type,
        marketPubkey: data.market_pubkey,
      },
    ];
  }

  /**
   * Handle error message from server.
   */
  private handleError(rawMsg: RawWsMessage): WsEvent[] {
    const data = rawMsg.data as ErrorData;
    console.error(`Server error: ${data.error} (code: ${data.code})`);
    return [
      {
        type: "Error",
        error: WebSocketError.serverError(data.code, data.error),
      },
    ];
  }

  /**
   * Initialize orderbook state for a subscription.
   */
  initOrderbook(orderbookId: string): void {
    if (!this.orderbooks.has(orderbookId)) {
      this.orderbooks.set(orderbookId, new LocalOrderbook(orderbookId));
    }
  }

  /**
   * Initialize user state for a subscription.
   */
  initUserState(user: string): void {
    if (!this.userStates.has(user)) {
      this.userStates.set(user, new UserState(user));
    }
  }

  /**
   * Initialize price history state for a subscription.
   */
  initPriceHistory(
    orderbookId: string,
    resolution: string,
    includeOhlcv: boolean
  ): void {
    const key = `${orderbookId}:${resolution}`;
    if (!this.priceHistories.has(key)) {
      this.priceHistories.set(
        key,
        new PriceHistory(orderbookId, resolution, includeOhlcv)
      );
    }
  }

  /**
   * Get orderbook state.
   */
  getOrderbook(orderbookId: string): LocalOrderbook | undefined {
    return this.orderbooks.get(orderbookId);
  }

  /**
   * Get user state.
   */
  getUserState(user: string): UserState | undefined {
    return this.userStates.get(user);
  }

  /**
   * Get price history state.
   */
  getPriceHistory(orderbookId: string, resolution: string): PriceHistory | undefined {
    const key = `${orderbookId}:${resolution}`;
    return this.priceHistories.get(key);
  }

  /**
   * Clear orderbook state.
   */
  clearOrderbook(orderbookId: string): void {
    const book = this.orderbooks.get(orderbookId);
    if (book) {
      book.clear();
    }
  }

  /**
   * Clear user state.
   */
  clearUserState(user: string): void {
    const state = this.userStates.get(user);
    if (state) {
      state.clear();
    }
  }

  /**
   * Clear price history state.
   */
  clearPriceHistory(orderbookId: string, resolution: string): void {
    const key = `${orderbookId}:${resolution}`;
    const history = this.priceHistories.get(key);
    if (history) {
      history.clear();
    }
  }

  /**
   * Clear all state.
   */
  clearAll(): void {
    this.orderbooks.clear();
    this.userStates.clear();
    this.priceHistories.clear();
  }
}
