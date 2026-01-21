/**
 * Main WebSocket client implementation.
 *
 * Provides a WebSocket client for real-time data streaming.
 */

import WebSocket from "ws";
import { Keypair } from "@solana/web3.js";
import { WebSocketError } from "./error";
import { MessageHandler } from "./handlers";
import { SubscriptionManager, subscriptionToParams } from "./subscriptions";
import type { LocalOrderbook, UserState, PriceHistory } from "./state";
import type { WsRequest, WsEvent } from "./types";
import {
  createSubscribeRequest,
  createUnsubscribeRequest,
  createPingRequest,
  bookUpdateParams,
  tradesParams,
  userParams,
  priceHistoryParams,
  marketParams,
} from "./types";
import { authenticateWithKeypair, type AuthCredentials } from "./auth";

/** Default WebSocket URL for Lightcone */
export const DEFAULT_WS_URL = "wss://ws.lightcone.xyz/ws";

/**
 * WebSocket client configuration.
 */
export interface WebSocketConfig {
  /** Number of reconnect attempts before giving up */
  reconnectAttempts?: number;
  /** Base delay for exponential backoff (ms) */
  baseDelayMs?: number;
  /** Maximum delay for exponential backoff (ms) */
  maxDelayMs?: number;
  /** Interval for client ping (seconds) */
  pingIntervalSecs?: number;
  /** Whether to automatically reconnect on disconnect */
  autoReconnect?: boolean;
  /** Whether to automatically re-subscribe after reconnect */
  autoResubscribe?: boolean;
  /** Optional authentication token for private user streams */
  authToken?: string;
}

/**
 * Connection state.
 */
export type ConnectionState =
  | "Disconnected"
  | "Connecting"
  | "Connected"
  | "Reconnecting"
  | "Disconnecting";

/**
 * Event callback type.
 */
export type EventCallback = (event: WsEvent) => void;

/**
 * Main WebSocket client for Lightcone.
 *
 * @example
 * ```typescript
 * import { LightconeWebSocketClient } from "@lightcone/sdk/websocket";
 *
 * const client = new LightconeWebSocketClient();
 * await client.connect();
 *
 * client.on("BookUpdate", (event) => {
 *   if (event.type === "BookUpdate") {
 *     const book = client.getOrderbook(event.orderbookId);
 *     console.log("Best bid:", book?.bestBid());
 *   }
 * });
 *
 * await client.subscribeBookUpdates(["market1:ob1"]);
 * ```
 */
export class LightconeWebSocketClient {
  private url: string;
  private config: Required<WebSocketConfig>;
  private state: ConnectionState = "Disconnected";
  private ws: WebSocket | null = null;
  private subscriptions: SubscriptionManager = new SubscriptionManager();
  private handler: MessageHandler = new MessageHandler();
  private reconnectAttempt: number = 0;
  private pingInterval: ReturnType<typeof setInterval> | null = null;
  private eventCallbacks: EventCallback[] = [];
  private authCredentials?: AuthCredentials;
  private subscribedUser: string | null = null;

  constructor(url: string = DEFAULT_WS_URL, config: WebSocketConfig = {}) {
    this.url = url;
    this.config = {
      reconnectAttempts: config.reconnectAttempts ?? 10,
      baseDelayMs: config.baseDelayMs ?? 1000,
      maxDelayMs: config.maxDelayMs ?? 30000,
      pingIntervalSecs: config.pingIntervalSecs ?? 30,
      autoReconnect: config.autoReconnect ?? true,
      autoResubscribe: config.autoResubscribe ?? true,
      authToken: config.authToken ?? "",
    };
  }

  /**
   * Create a client connected to the default URL.
   */
  static async connectDefault(): Promise<LightconeWebSocketClient> {
    const client = new LightconeWebSocketClient();
    await client.connect();
    return client;
  }

  /**
   * Create a client connected to a custom URL.
   */
  static async connect(
    url: string,
    config?: WebSocketConfig
  ): Promise<LightconeWebSocketClient> {
    const client = new LightconeWebSocketClient(url, config);
    await client.connect();
    return client;
  }

  /**
   * Create an authenticated client connected to the default URL.
   */
  static async connectAuthenticated(
    keypair: Keypair
  ): Promise<LightconeWebSocketClient> {
    const credentials = await authenticateWithKeypair(keypair);
    const client = new LightconeWebSocketClient(DEFAULT_WS_URL, {
      authToken: credentials.authToken,
    });
    client.authCredentials = credentials;
    await client.connect();
    return client;
  }

  /**
   * Create an authenticated client with custom config.
   */
  static async connectAuthenticatedWithConfig(
    keypair: Keypair,
    config: WebSocketConfig
  ): Promise<LightconeWebSocketClient> {
    const credentials = await authenticateWithKeypair(keypair);
    config.authToken = credentials.authToken;
    const client = new LightconeWebSocketClient(DEFAULT_WS_URL, config);
    client.authCredentials = credentials;
    await client.connect();
    return client;
  }

  /**
   * Connect to the WebSocket server.
   */
  async connect(): Promise<void> {
    if (this.state === "Connected" || this.state === "Connecting") {
      return;
    }

    this.state = "Connecting";
    await this.establishConnection();
  }

  /**
   * Establish the WebSocket connection.
   */
  private async establishConnection(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        // Build WebSocket options with Cookie header if authenticated
        const options: WebSocket.ClientOptions = {};
        if (this.config.authToken) {
          options.headers = {
            Cookie: `auth_token=${this.config.authToken}`,
          };
        }

        this.ws = new WebSocket(this.url, options);
      } catch (e) {
        this.state = "Disconnected";
        reject(WebSocketError.connectionFailed(String(e)));
        return;
      }

      this.ws.onopen = () => {
        this.state = "Connected";
        this.reconnectAttempt = 0;
        this.startPingInterval();
        this.emitEvent({ type: "Connected" });
        resolve();
      };

      this.ws.onmessage = (event) => {
        const events = this.handler.handleMessage(event.data as string);
        for (const wsEvent of events) {
          this.emitEvent(wsEvent);
        }
      };

      this.ws.onerror = (event) => {
        console.error("WebSocket error:", event);
        this.emitEvent({
          type: "Error",
          error: WebSocketError.connectionFailed("WebSocket error"),
        });
      };

      this.ws.onclose = (event) => {
        this.stopPingInterval();
        const reason = `code: ${event.code}, reason: ${event.reason || "no reason"}`;
        this.emitEvent({ type: "Disconnected", reason });

        // Check if rate limited
        if (event.code === 1008) {
          this.emitEvent({
            type: "Error",
            error: WebSocketError.rateLimited(),
          });
        }

        // Try to reconnect if enabled
        if (
          this.config.autoReconnect &&
          this.reconnectAttempt < this.config.reconnectAttempts &&
          this.state !== "Disconnecting"
        ) {
          this.attemptReconnect();
        } else {
          this.state = "Disconnected";
        }
      };
    });
  }

  /**
   * Attempt to reconnect.
   */
  private async attemptReconnect(): Promise<void> {
    this.reconnectAttempt++;
    this.state = "Reconnecting";
    this.emitEvent({ type: "Reconnecting", attempt: this.reconnectAttempt });

    const delay =
      this.config.baseDelayMs *
      Math.pow(2, this.reconnectAttempt - 1);
    const cappedDelay = Math.min(delay, this.config.maxDelayMs);

    await this.sleep(cappedDelay);

    try {
      await this.establishConnection();

      // Re-subscribe if enabled
      if (this.config.autoResubscribe) {
        this.handler.clearAll();
        const subs = this.subscriptions.getAllSubscriptions();
        for (const sub of subs) {
          const request = createSubscribeRequest(subscriptionToParams(sub));
          this.send(request);
        }
      }
    } catch (e) {
      console.error("Reconnect failed:", e);
      this.emitEvent({
        type: "Error",
        error: e instanceof WebSocketError
          ? e
          : WebSocketError.connectionFailed(String(e)),
      });
    }
  }

  /**
   * Start the ping interval.
   */
  private startPingInterval(): void {
    this.stopPingInterval();
    this.pingInterval = setInterval(() => {
      this.ping();
    }, this.config.pingIntervalSecs * 1000);
  }

  /**
   * Stop the ping interval.
   */
  private stopPingInterval(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }

  /**
   * Sleep for a given number of milliseconds.
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Send a WebSocket message.
   */
  private send(message: WsRequest): void {
    if (!this.ws || this.state !== "Connected") {
      throw WebSocketError.notConnected();
    }
    this.ws.send(JSON.stringify(message));
  }

  /**
   * Emit an event to all callbacks.
   */
  private emitEvent(event: WsEvent): void {
    for (const callback of this.eventCallbacks) {
      try {
        callback(event);
      } catch (e) {
        console.error("Event callback error:", e);
      }
    }
  }

  // ============================================================================
  // SUBSCRIBE METHODS
  // ============================================================================

  /**
   * Subscribe to orderbook updates.
   */
  subscribeBookUpdates(orderbookIds: string[]): void {
    // Initialize state for each orderbook
    for (const id of orderbookIds) {
      this.handler.initOrderbook(id);
    }

    // Track subscription
    this.subscriptions.addBookUpdate(orderbookIds);

    // Send subscribe request
    const request = createSubscribeRequest(bookUpdateParams(orderbookIds));
    this.send(request);
  }

  /**
   * Subscribe to trade executions.
   */
  subscribeTrades(orderbookIds: string[]): void {
    this.subscriptions.addTrades(orderbookIds);
    const request = createSubscribeRequest(tradesParams(orderbookIds));
    this.send(request);
  }

  /**
   * Subscribe to user events.
   */
  subscribeUser(user: string): void {
    this.subscribedUser = user;
    this.handler.initUserState(user);
    this.subscriptions.addUser(user);
    const request = createSubscribeRequest(userParams(user));
    this.send(request);
  }

  /**
   * Subscribe to price history.
   */
  subscribePriceHistory(
    orderbookId: string,
    resolution: string,
    includeOhlcv: boolean
  ): void {
    this.handler.initPriceHistory(orderbookId, resolution, includeOhlcv);
    this.subscriptions.addPriceHistory(orderbookId, resolution, includeOhlcv);
    const request = createSubscribeRequest(
      priceHistoryParams(orderbookId, resolution, includeOhlcv)
    );
    this.send(request);
  }

  /**
   * Subscribe to market events.
   */
  subscribeMarket(marketPubkey: string): void {
    this.subscriptions.addMarket(marketPubkey);
    const request = createSubscribeRequest(marketParams(marketPubkey));
    this.send(request);
  }

  // ============================================================================
  // UNSUBSCRIBE METHODS
  // ============================================================================

  /**
   * Unsubscribe from orderbook updates.
   */
  unsubscribeBookUpdates(orderbookIds: string[]): void {
    this.subscriptions.removeBookUpdate(orderbookIds);
    const request = createUnsubscribeRequest(bookUpdateParams(orderbookIds));
    this.send(request);
  }

  /**
   * Unsubscribe from trades.
   */
  unsubscribeTrades(orderbookIds: string[]): void {
    this.subscriptions.removeTrades(orderbookIds);
    const request = createUnsubscribeRequest(tradesParams(orderbookIds));
    this.send(request);
  }

  /**
   * Unsubscribe from user events.
   */
  unsubscribeUser(user: string): void {
    if (this.subscribedUser === user) {
      this.subscribedUser = null;
    }
    this.handler.clearSubscribedUser(user);
    this.subscriptions.removeUser(user);
    const request = createUnsubscribeRequest(userParams(user));
    this.send(request);
  }

  /**
   * Unsubscribe from price history.
   */
  unsubscribePriceHistory(
    orderbookId: string,
    resolution: string
  ): void {
    this.subscriptions.removePriceHistory(orderbookId, resolution);
    const request = createUnsubscribeRequest(
      priceHistoryParams(orderbookId, resolution, false)
    );
    this.send(request);
  }

  /**
   * Unsubscribe from market events.
   */
  unsubscribeMarket(marketPubkey: string): void {
    this.subscriptions.removeMarket(marketPubkey);
    const request = createUnsubscribeRequest(marketParams(marketPubkey));
    this.send(request);
  }

  // ============================================================================
  // CONTROL METHODS
  // ============================================================================

  /**
   * Send a ping request.
   */
  ping(): void {
    this.send(createPingRequest());
  }

  /**
   * Disconnect from the server.
   */
  async disconnect(): Promise<void> {
    this.state = "Disconnecting";
    this.stopPingInterval();
    if (this.ws) {
      this.ws.close(1000, "Client disconnect");
      this.ws = null;
    }
    this.state = "Disconnected";
  }

  /**
   * Check if connected.
   */
  isConnected(): boolean {
    return this.state === "Connected";
  }

  /**
   * Check if authenticated.
   */
  isAuthenticated(): boolean {
    return !!this.config.authToken;
  }

  /**
   * Get the current connection state.
   */
  connectionState(): ConnectionState {
    return this.state;
  }

  // ============================================================================
  // STATE ACCESS
  // ============================================================================

  /**
   * Get orderbook state.
   */
  getOrderbook(orderbookId: string): LocalOrderbook | undefined {
    return this.handler.getOrderbook(orderbookId);
  }

  /**
   * Get user state.
   */
  getUserState(user: string): UserState | undefined {
    return this.handler.getUserState(user);
  }

  /**
   * Get price history state.
   */
  getPriceHistory(
    orderbookId: string,
    resolution: string
  ): PriceHistory | undefined {
    return this.handler.getPriceHistory(orderbookId, resolution);
  }

  /**
   * Get auth credentials (if authenticated).
   */
  getAuthCredentials(): AuthCredentials | undefined {
    return this.authCredentials;
  }

  /**
   * Get user public key (if authenticated).
   */
  userPubkey(): string | undefined {
    return this.authCredentials?.userPubkey;
  }

  // ============================================================================
  // EVENT HANDLING
  // ============================================================================

  /**
   * Register an event callback.
   */
  on(callback: EventCallback): void {
    this.eventCallbacks.push(callback);
  }

  /**
   * Remove an event callback.
   */
  off(callback: EventCallback): void {
    const index = this.eventCallbacks.indexOf(callback);
    if (index !== -1) {
      this.eventCallbacks.splice(index, 1);
    }
  }

  /**
   * Get the WebSocket URL.
   */
  getUrl(): string {
    return this.url;
  }

  /**
   * Get the configuration.
   */
  getConfig(): Required<WebSocketConfig> {
    return this.config;
  }
}
