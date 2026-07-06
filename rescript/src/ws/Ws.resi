// WebSocket connection — the app-level transport over the `WebSocketClient`
// binding: auto-reconnect with capped backoff, a ping/pong heartbeat, and
// subscription tracking that re-subscribes on reconnect. Public surface: the
// tunable `Config`, the opaque connection handle `t`, and the connect/subscribe/
// unsubscribe/ping/disconnect lifecycle. The socket plumbing — timers, backoff,
// `establish`/heartbeat/reconnect, and the low-level send — is internal.

// ── Config ────────────────────────────────────────────────────────────────────
module Config: {
  type t = {
    reconnect: bool,
    maxReconnectAttempts: int,
    baseReconnectDelayMs: int,
    pingIntervalMs: int,
    pongTimeoutMs: int,
  }
}

// Default connection config.
let defaultConfig: Config.t

// ── Connection ────────────────────────────────────────────────────────────────
// Opaque handle; all mutable socket/subscription state is internal.
type t

// Open a connection and begin the connect/reconnect loop immediately. Callbacks
// default to no-ops; pass a `~config` to tune reconnect/heartbeat behavior.
let connect: (
  ~url: string,
  ~onMessage: Messages.t => unit,
  ~onConnected: unit => unit=?,
  ~onDisconnected: (option<int>, string) => unit=?,
  ~onError: SdkError.t => unit=?,
  ~config: Config.t=?,
  unit,
) => t

// Whether the underlying socket is currently OPEN.
let isConnected: t => bool

// Track the subscription (deduped, survives reconnects) and send it. Always `Ok`.
let subscribe: (t, Subscriptions.SubscribeParams.t) => result<unit, SdkError.t>

// Stop tracking the matching subscription(s) and send the unsubscribe.
let unsubscribe: (t, Subscriptions.UnsubscribeParams.t) => result<unit, SdkError.t>

// Send an application-level ping immediately (manual liveness check).
let ping: t => result<unit, SdkError.t>

// Close the connection and suppress reconnection. Idempotent.
let disconnect: t => unit

// The socket lifecycle state; `Closed` when no socket exists.
module ReadyState: {
  @genType
  type t = Connecting | Open | Closing | Closed
}

@genType
let readyState: t => ReadyState.t

// Drop tracked authenticated-channel subscriptions (the `User` channel) so they
// are not re-sent on the next reconnect — call after logout.
@genType
let clearAuthedSubscriptions: t => unit
