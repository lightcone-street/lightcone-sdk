/**
 * WebSocket error types for the Lightcone WebSocket client.
 */

/**
 * WebSocket error variants.
 */
export type WebSocketErrorVariant =
  | "ConnectionFailed"
  | "ConnectionClosed"
  | "RateLimited"
  | "MessageParseError"
  | "SequenceGap"
  | "ResyncRequired"
  | "PingTimeout"
  | "ServerError"
  | "AuthenticationFailed"
  | "ChannelClosed"
  | "NotConnected"
  | "InvalidUrl"
  | "Protocol"
  | "HttpError";

/**
 * WebSocket error class.
 */
export class WebSocketError extends Error {
  readonly variant: WebSocketErrorVariant;
  readonly code?: string;
  readonly details?: Record<string, unknown>;

  constructor(
    variant: WebSocketErrorVariant,
    message: string,
    code?: string,
    details?: Record<string, unknown>
  ) {
    super(message);
    this.name = "WebSocketError";
    this.variant = variant;
    this.code = code;
    this.details = details;
  }

  /** Failed to establish connection */
  static connectionFailed(message: string): WebSocketError {
    return new WebSocketError("ConnectionFailed", `Connection failed: ${message}`);
  }

  /** Connection was closed */
  static connectionClosed(code: number, reason: string): WebSocketError {
    return new WebSocketError(
      "ConnectionClosed",
      `Connection closed: code ${code}, reason: ${reason}`,
      String(code)
    );
  }

  /** Rate limit exceeded */
  static rateLimited(): WebSocketError {
    return new WebSocketError("RateLimited", "Rate limited by server");
  }

  /** Failed to parse message */
  static messageParseError(message: string): WebSocketError {
    return new WebSocketError("MessageParseError", `Failed to parse message: ${message}`);
  }

  /** Sequence gap detected in orderbook updates */
  static sequenceGap(expected: number, received: number): WebSocketError {
    return new WebSocketError(
      "SequenceGap",
      `Sequence gap: expected ${expected}, received ${received}`,
      undefined,
      { expected, received }
    );
  }

  /** Resync required for orderbook */
  static resyncRequired(orderbookId: string): WebSocketError {
    return new WebSocketError(
      "ResyncRequired",
      `Resync required for orderbook: ${orderbookId}`,
      undefined,
      { orderbookId }
    );
  }

  /** Ping timeout */
  static pingTimeout(): WebSocketError {
    return new WebSocketError("PingTimeout", "Ping timeout - no response from server");
  }

  /** Server error */
  static serverError(code: string, message: string): WebSocketError {
    return new WebSocketError("ServerError", `Server error: ${message}`, code);
  }

  /** Authentication failed */
  static authenticationFailed(message: string): WebSocketError {
    return new WebSocketError("AuthenticationFailed", `Authentication failed: ${message}`);
  }

  /** Channel closed */
  static channelClosed(): WebSocketError {
    return new WebSocketError("ChannelClosed", "Internal channel closed");
  }

  /** Not connected */
  static notConnected(): WebSocketError {
    return new WebSocketError("NotConnected", "Not connected to server");
  }

  /** Invalid URL */
  static invalidUrl(message: string): WebSocketError {
    return new WebSocketError("InvalidUrl", `Invalid URL: ${message}`);
  }

  /** Protocol error */
  static protocol(message: string): WebSocketError {
    return new WebSocketError("Protocol", `Protocol error: ${message}`);
  }

  /** HTTP error during authentication */
  static httpError(message: string): WebSocketError {
    return new WebSocketError("HttpError", `HTTP error: ${message}`);
  }
}

/**
 * Result type alias for WebSocket operations.
 */
export type WsResult<T> = T;
