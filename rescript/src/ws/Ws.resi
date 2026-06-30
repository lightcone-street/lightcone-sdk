// WebSocket connection — the app-level transport over the `WebSocketClient`
// binding: auto-reconnect with capped backoff, a ping/pong heartbeat, and
// subscription tracking that re-subscribes on reconnect. Public surface: the
// tunable `config`, the opaque `connection` handle, and the connect/subscribe/
// unsubscribe/ping/disconnect lifecycle. The socket plumbing — timers, backoff,
// `establish`/heartbeat/reconnect, and the low-level send — is internal.

// ── Config ────────────────────────────────────────────────────────────────────
type config = {
  reconnect: bool,
  maxReconnectAttempts: int,
  baseReconnectDelayMs: int,
  pingIntervalMs: int,
  pongTimeoutMs: int,
}

// Matches the Rust `WsConfig::default`.
let defaultConfig: config

// ── Connection ────────────────────────────────────────────────────────────────
// Opaque handle; all mutable socket/subscription state is internal.
type connection

// Open a connection and begin the connect/reconnect loop immediately. Callbacks
// default to no-ops; pass a `~config` to tune reconnect/heartbeat behavior.
let connect: (
  ~url: string,
  ~onMessage: Messages.messageIn => unit,
  ~onConnected: unit => unit=?,
  ~onDisconnected: (option<int>, string) => unit=?,
  ~onError: SdkError.t => unit=?,
  ~config: config=?,
  unit,
) => connection

// Whether the underlying socket is currently OPEN.
let isConnected: connection => bool

// Track the subscription (deduped, survives reconnects) and send it. Always `Ok`.
let subscribe: (connection, Subscriptions.SubscribeParams.t) => result<unit, SdkError.t>

// Stop tracking the matching subscription(s) and send the unsubscribe.
let unsubscribe: (connection, Subscriptions.UnsubscribeParams.t) => result<unit, SdkError.t>

// Send an application-level ping immediately (manual liveness check).
let ping: connection => result<unit, SdkError.t>

// Close the connection and suppress reconnection. Idempotent.
let disconnect: connection => unit
