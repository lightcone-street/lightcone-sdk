import WebSocket from "ws";
import {
  parseMessageIn,
  ping,
  readyStateFrom,
  ReadyState,
  type MessageIn,
  type MessageOut,
  type WsConfig,
  type WsEvent,
  WS_DEFAULT_CONFIG,
} from "./index";
import {
  subscriptionKey,
  unsubscribeMatches,
  type SubscribeParams,
  type UnsubscribeParams,
} from "./subscriptions";
import type { IWsClient } from "./types";

export class WsClient implements IWsClient {
  private readonly config: WsConfig;
  private readonly authTokenRef?: () => Promise<string | undefined>;
  private socket?: WebSocket;
  private reconnectAttempts = 0;
  private userInitiatedClose = false;
  private pingTimer?: ReturnType<typeof setInterval>;
  private pongTimer?: ReturnType<typeof setTimeout>;
  private callbacks: Array<(event: WsEvent) => void> = [];
  private readonly activeSubscriptions: SubscribeParams[] = [];
  private readonly pendingMessages: MessageOut[] = [];

  constructor(config?: Partial<WsConfig>, authTokenRef?: () => Promise<string | undefined>) {
    this.config = { ...WS_DEFAULT_CONFIG, ...config };
    this.authTokenRef = authTokenRef;
  }

  async connect(): Promise<void> {
    if (this.socket && this.readyState() === ReadyState.Open) {
      return;
    }

    await this.connectInternal();
  }

  async disconnect(): Promise<void> {
    if (!this.socket) {
      return;
    }

    this.stopHeartbeat();

    const socket = this.socket;
    this.socket = undefined;
    this.userInitiatedClose = true;

    await new Promise<void>((resolve) => {
      socket.once("close", () => {
        resolve();
      });
      socket.close(1000, "Client disconnect");
    });
  }

  send(message: MessageOut): void {
    this.trackMessage(message);

    if (!this.socket || this.readyState() !== ReadyState.Open) {
      this.pendingMessages.push(message);
      return;
    }

    this.socket.send(JSON.stringify(message));
  }

  subscribe(params: SubscribeParams): void {
    this.send({ method: "subscribe", params });
  }

  unsubscribe(params: UnsubscribeParams): void {
    this.send({ method: "unsubscribe", params });
  }

  isConnected(): boolean {
    return this.readyState() === ReadyState.Open;
  }

  readyState(): ReadyState {
    if (!this.socket) {
      return ReadyState.Closed;
    }
    return readyStateFrom(this.socket.readyState);
  }

  on(callback: (event: WsEvent) => void): () => void {
    this.callbacks.push(callback);
    return () => {
      this.callbacks = this.callbacks.filter((cb) => cb !== callback);
    };
  }

  private emit(event: WsEvent): void {
    for (const callback of this.callbacks) {
      try {
        callback(event);
      } catch (err) {
        console.warn("WsClient: listener threw", err instanceof Error ? err.message : String(err));
      }
    }
  }

  private async connectInternal(): Promise<void> {
    const headers: Record<string, string> = {};
    const token = this.authTokenRef ? await this.authTokenRef() : undefined;
    if (token) {
      headers.Cookie = `auth_token=${token}`;
    }

    const socket = new WebSocket(this.config.url, { headers });
    this.socket = socket;

    await new Promise<void>((resolve, reject) => {
      socket.once("open", () => {
        this.reconnectAttempts = 0;
        this.startHeartbeat();
        this.flushPendingMessages();
        this.resubscribeAll();
        this.emit({ type: "Connected" });
        resolve();
      });

      socket.once("error", (error) => {
        reject(error);
      });

      socket.on("message", (raw) => {
        this.handleIncoming(raw.toString());
      });

      socket.on("close", (code, reason) => {
        const manualClose = this.userInitiatedClose;
        this.userInitiatedClose = false;
        this.stopHeartbeat();
        this.emit({ type: "Disconnected", code, reason: reason.toString() });
        if (!manualClose) {
          this.handleClose(code);
        }
      });
    });
  }

  private handleIncoming(raw: string): void {
    let message: MessageIn;

    try {
      message = parseMessageIn(raw);
    } catch (error) {
      this.emit({
        type: "Error",
        error: `Deserialization error: ${error instanceof Error ? error.message : String(error)}`,
      });
      return;
    }

    if (message.type === "pong") {
      this.resetPongTimeout();
    }

    this.emit({ type: "Message", message });
  }

  private handleClose(code: number): void {
    if (!this.config.reconnect) {
      return;
    }

    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      this.emit({ type: "MaxReconnectReached" });
      return;
    }

    this.reconnectAttempts += 1;
    const isRateLimited = code === 1008;
    const backoffMs = this.backoffDelayMs(this.reconnectAttempts, isRateLimited);

    setTimeout(() => {
      void this.connectInternal().catch((error) => {
        this.emit({
          type: "Error",
          error: `Connection failed: ${error instanceof Error ? error.message : String(error)}`,
        });
        this.handleClose(1011);
      });
    }, backoffMs);
  }

  private backoffDelayMs(attempt: number, isRateLimited: boolean): number {
    const exponent = Math.max(0, attempt - 1);
    const raw = this.config.baseReconnectDelayMs * 2 ** exponent;
    const jitter = Math.random() * raw;
    const rateLimitPenalty = isRateLimited ? this.config.baseReconnectDelayMs : 0;
    return Math.floor(Math.min(raw + jitter + rateLimitPenalty, 60_000));
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();

    this.pingTimer = setInterval(() => {
      this.send(ping());
      this.resetPongTimeout();
    }, this.config.pingIntervalMs);
  }

  private stopHeartbeat(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = undefined;
    }
    if (this.pongTimer) {
      clearTimeout(this.pongTimer);
      this.pongTimer = undefined;
    }
  }

  private resetPongTimeout(): void {
    if (this.pongTimer) {
      clearTimeout(this.pongTimer);
    }

    this.pongTimer = setTimeout(() => {
      if (this.socket && this.readyState() === ReadyState.Open) {
        this.socket.close(4000, "Pong timeout");
      }
    }, this.config.pongTimeoutMs);
  }

  private trackMessage(message: MessageOut): void {
    if (message.method === "subscribe") {
      const key = subscriptionKey(message.params);
      const existing = this.activeSubscriptions.find((params) => subscriptionKey(params) === key);
      if (!existing) {
        this.activeSubscriptions.push(message.params);
      }
      return;
    }

    if (message.method === "unsubscribe") {
      const next = this.activeSubscriptions.filter(
        (params) => !unsubscribeMatches(params, message.params)
      );
      this.activeSubscriptions.length = 0;
      this.activeSubscriptions.push(...next);
    }
  }

  private flushPendingMessages(): void {
    if (!this.socket || this.readyState() !== ReadyState.Open) {
      return;
    }

    while (this.pendingMessages.length > 0) {
      const message = this.pendingMessages.shift();
      if (!message) {
        continue;
      }
      this.socket.send(JSON.stringify(message));
    }
  }

  private resubscribeAll(): void {
    for (const params of this.activeSubscriptions) {
      this.pendingMessages.push({ method: "subscribe", params });
    }
    this.flushPendingMessages();
  }
}
