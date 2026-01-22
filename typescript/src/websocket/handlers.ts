/**
 * Message handlers for WebSocket events.
 *
 * Routes incoming messages to appropriate handlers and emits events.
 */

import { WebSocketError } from "./error";
import { LocalOrderbook, UserState, PriceHistory } from "./state";
import type {
  WsEvent,
  BookUpdateData,
  TradeData,
  UserEventData,
  PriceHistoryData,
  MarketEventData,
  ErrorData,
  AuthData,
} from "./types";
import { parseWsMessage } from "./types";

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
  /** Currently subscribed user (single user per connection) */
  private subscribedUser: string | null = null;

  /**
   * Handle an incoming message and return events.
   */
  handleMessage(text: string): WsEvent[] {
    const msg = parseWsMessage(text);
    if (!msg) {
      return [
        {
          type: "Error",
          error: WebSocketError.messageParseError("Invalid message"),
        },
      ];
    }

    // TypeScript narrows msg.data type automatically based on msg.type
    switch (msg.type) {
      case "book_update":
        return this.handleBookUpdate(msg.data);
      case "trades":
        return this.handleTrade(msg.data);
      case "user":
        return this.handleUserEvent(msg.data);
      case "price_history":
        return this.handlePriceHistory(msg.data);
      case "market":
        return this.handleMarketEvent(msg.data);
      case "error":
        return this.handleError(msg.data);
      case "pong":
        return [{ type: "Pong" }];
      case "auth":
        return this.handleAuth(msg.data);
      default:
        console.warn("Unknown message type:", (msg as { type: string }).type);
        return [];
    }
  }

  /**
   * Handle book update message.
   */
  private handleBookUpdate(data: BookUpdateData): WsEvent[] {
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
  private handleTrade(data: TradeData): WsEvent[] {
    return [{ type: "Trade", orderbookId: data.orderbook_id, trade: data }];
  }

  /**
   * Handle user event message.
   */
  private handleUserEvent(data: UserEventData): WsEvent[] {
    const eventType = data.event_type;

    // Use the tracked subscribed user (single user per connection)
    const user = this.subscribedUser;
    if (!user) {
      console.warn(
        `Received user event '${eventType}' but no user subscription exists. ` +
          `Call subscribeUser() before receiving events to avoid data loss.`
      );
      return [{ type: "UserUpdate", eventType, user: "unknown" }];
    }

    // Update local state for the subscribed user
    const state = this.userStates.get(user);
    if (state) {
      state.applyEvent(data);
    }

    return [{ type: "UserUpdate", eventType, user }];
  }

  /**
   * Handle price history message.
   */
  private handlePriceHistory(data: PriceHistoryData): WsEvent[] {
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
  private handleMarketEvent(data: MarketEventData): WsEvent[] {
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
  private handleError(data: ErrorData): WsEvent[] {
    console.error(`Server error: ${data.error} (code: ${data.code})`);
    return [
      {
        type: "Error",
        error: WebSocketError.serverError(data.code, data.error),
      },
    ];
  }

  /**
   * Handle auth message.
   */
  private handleAuth(data: AuthData): WsEvent[] {
    if (data.status === "error") {
      return [
        {
          type: "Error",
          error: WebSocketError.authenticationFailed(data.message || "Authentication failed"),
        },
      ];
    }
    // For authenticated/anonymous, just log and continue
    return [];
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
    this.subscribedUser = user;
    if (!this.userStates.has(user)) {
      this.userStates.set(user, new UserState(user));
    }
  }

  /**
   * Clear the subscribed user.
   */
  clearSubscribedUser(user: string): void {
    if (this.subscribedUser === user) {
      this.subscribedUser = null;
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
