// WebSocket connection — the ReScript counterpart of the Rust SDK's `ws::native`
// `WsClient`. Built on the `WebSocketClient` binding (the platform-global
// `WebSocket`), it adds the app-level transport concerns:
//
//   • auto-reconnect with capped exponential backoff + jitter (rate-limit aware),
//   • an application-level ping/pong heartbeat (`Subscriptions.Ping` on an
//     interval; a pong — or any inbound frame — marks the socket alive; a missed
//     pong forces a reconnect),
//   • subscription tracking: every `subscribe` is remembered and re-sent on
//     reconnect (and on the initial open, covering subscribes issued while still
//     connecting),
//   • each inbound text frame parsed through `Messages.decodeMessage`.
//
// Single implementation (no native/browser split) — the binding is the platform
// `WebSocket`, present on Node 22 / Bun / browsers.

// ── Config ────────────────────────────────────────────────────────────────────
type config = {
  reconnect: bool,
  maxReconnectAttempts: int,
  baseReconnectDelayMs: int,
  pingIntervalMs: int,
  pongTimeoutMs: int,
}

// Matches the Rust `WsConfig::default`.
let defaultConfig: config = {
  reconnect: true,
  maxReconnectAttempts: 10,
  baseReconnectDelayMs: 1000,
  pingIntervalMs: 30000,
  pongTimeoutMs: 10000,
}

// ── Connection ────────────────────────────────────────────────────────────────
// All connection state is mutable: the underlying socket is replaced on each
// reconnect, while the tracked subscriptions / callbacks persist.
type connection = {
  url: string,
  config: config,
  onMessage: Messages.messageIn => unit,
  onConnected: unit => unit,
  onDisconnected: (option<int>, string) => unit,
  onError: SdkError.t => unit,
  mutable socket: option<WebSocketClient.t>,
  mutable activeSubscriptions: array<Subscriptions.SubscribeParams.t>,
  mutable reconnectAttempts: int,
  mutable closedByUser: bool,
  mutable pingTimer: option<intervalId>,
  mutable pongTimer: option<timeoutId>,
}

// ── Low-level send ────────────────────────────────────────────────────────────
// Serialize + send a message. `WebSocketClient.send` throws unless the socket is
// OPEN, so we gate on `readyState` and catch the JS exn defensively.
let sendMessage = (connection: connection, message: Subscriptions.messageOut): result<
  unit,
  SdkError.t,
> =>
  switch connection.socket {
  | Some(socket) if WebSocketClient.readyState(socket) == 1 =>
    let text = JSON.stringify(Subscriptions.toJson(message))
    switch WebSocketClient.send(socket, text) {
    | () => Ok()
    | exception JsExn(error) =>
      Error(
        SdkError.Ws(SdkError.SendFailed(error->JsExn.message->Option.getOr("socket send failed"))),
      )
    }
  | _ => Error(SdkError.Ws(SdkError.NotConnected))
  }

// ── Timer management ──────────────────────────────────────────────────────────
let clearPongTimer = (connection: connection): unit => {
  connection.pongTimer->Option.forEach(timer => clearTimeout(timer))
  connection.pongTimer = None
}

let clearTimers = (connection: connection): unit => {
  connection.pingTimer->Option.forEach(timer => clearInterval(timer))
  connection.pingTimer = None
  clearPongTimer(connection)
}

// ── Reconnection backoff ──────────────────────────────────────────────────────
// `base * 2^min(attempts-1, 10) + jitter`, capped. Rate-limit closes (1008) back
// off harder (up to 5 minutes); everything else up to 60 seconds. Mirrors the
// Rust `backoff_sleep`. Assumes `reconnectAttempts` has already been incremented.
let reconnectDelay = (connection: connection, code: int): int => {
  let cappedExponent = {
    let raw = connection.reconnectAttempts - 1
    raw > 10 ? 10 : raw
  }
  let base =
    Int.toFloat(connection.config.baseReconnectDelayMs) *.
    Math.pow(2.0, ~exp=Int.toFloat(cappedExponent))
  let (jitterMax, cap) = code == 1008 ? (1000.0, 300000.0) : (500.0, 60000.0)
  let jitter = Math.random() *. jitterMax
  Math.min(base +. jitter, cap)->Float.toInt
}

// ── Connection lifecycle (mutually recursive: reconnect re-establishes) ───────
let rec establish = (connection: connection): unit => {
  let socket = WebSocketClient.make(connection.url)
  connection.socket = Some(socket)

  socket->WebSocketClient.setOnOpen(() => {
    connection.reconnectAttempts = 0
    // Re-send every tracked subscription — this also covers subscriptions issued
    // before the initial connection opened.
    connection.activeSubscriptions->Array.forEach(params =>
      sendMessage(connection, Subscriptions.Subscribe(params))->ignore
    )
    startHeartbeat(connection)
    connection.onConnected()
  })

  socket->WebSocketClient.setOnMessage(event => {
    // Any inbound frame proves the socket is alive — disarm the pong deadline.
    clearPongTimer(connection)
    let text = WebSocketClient.data(event)
    switch JSON.parseOrThrow(text) {
    | json =>
      switch Messages.decodeMessage(json) {
      | Ok(message) => connection.onMessage(message)
      | Error(error) => connection.onError(error)
      }
    | exception JsExn(error) =>
      connection.onError(
        SdkError.Ws(
          SdkError.DeserializationError(error->JsExn.message->Option.getOr("invalid JSON frame")),
        ),
      )
    }
  })

  socket->WebSocketClient.setOnError(_event =>
    connection.onError(SdkError.Ws(SdkError.ConnectionFailed("websocket error")))
  )

  socket->WebSocketClient.setOnClose(event => {
    let code = WebSocketClient.code(event)
    let reason = WebSocketClient.reason(event)
    clearTimers(connection)
    connection.socket = None
    connection.onDisconnected(Some(code), reason)

    // 1000 is a normal close; never reconnect after it or a user-initiated close.
    if !connection.closedByUser && connection.config.reconnect && code != 1000 {
      scheduleReconnect(connection, code)
    }
  })
}
// Start (or restart) the ping interval. Each tick sends a `Ping` and arms a pong
// deadline; if no frame arrives before it fires, force-close to trigger reconnect.
and startHeartbeat = (connection: connection): unit => {
  connection.pingTimer->Option.forEach(timer => clearInterval(timer))
  let timer = setInterval(() => {
    switch sendMessage(connection, Subscriptions.Ping) {
    | Ok() =>
      clearPongTimer(connection)
      connection.pongTimer = Some(setTimeout(() => {
          connection.socket->Option.forEach(socket => WebSocketClient.close(socket))
        }, connection.config.pongTimeoutMs))
    | Error(_) => ()
    }
  }, connection.config.pingIntervalMs)
  connection.pingTimer = Some(timer)
}
// Schedule the next reconnect attempt, or report exhaustion via `onError`.
and scheduleReconnect = (connection: connection, code: int): unit =>
  if connection.reconnectAttempts >= connection.config.maxReconnectAttempts {
    connection.onError(
      SdkError.Ws(
        SdkError.ConnectionFailed(
          `max reconnect attempts (${Int.toString(
              connection.config.maxReconnectAttempts,
            )}) reached`,
        ),
      ),
    )
  } else {
    connection.reconnectAttempts = connection.reconnectAttempts + 1
    let delay = reconnectDelay(connection, code)
    setTimeout(() =>
      if !connection.closedByUser {
        establish(connection)
      }
    , delay)->ignore
  }

// ── Public API ────────────────────────────────────────────────────────────────
// Open a connection and begin the connect/reconnect loop immediately. Callbacks
// default to no-ops; pass a `~config` to tune reconnect/heartbeat behavior.
let connect = (
  ~url: string,
  ~onMessage: Messages.messageIn => unit,
  ~onConnected: unit => unit=() => (),
  ~onDisconnected: (option<int>, string) => unit=(_, _) => (),
  ~onError: SdkError.t => unit=_ => (),
  ~config: config=defaultConfig,
  (),
): connection => {
  let connection = {
    url,
    config,
    onMessage,
    onConnected,
    onDisconnected,
    onError,
    socket: None,
    activeSubscriptions: [],
    reconnectAttempts: 0,
    closedByUser: false,
    pingTimer: None,
    pongTimer: None,
  }
  establish(connection)
  connection
}

// Whether the underlying socket is currently OPEN.
let isConnected = (connection: connection): bool =>
  switch connection.socket {
  | Some(socket) => WebSocketClient.readyState(socket) == 1
  | None => false
  }

// The socket lifecycle state (Rust `ReadyState`); `Closed` when no socket exists.
type readyState = Connecting | Open | Closing | Closed

let readyState = (connection: connection): readyState =>
  switch connection.socket {
  | None => Closed
  | Some(socket) =>
    switch WebSocketClient.readyState(socket) {
    | 0 => Connecting
    | 1 => Open
    | 2 => Closing
    | _ => Closed
    }
  }

// Drop tracked authenticated-channel subscriptions (the `User` channel) so they
// are not re-sent on the next reconnect — call after logout (the Rust
// `clear_authed_subscriptions`).
let clearAuthedSubscriptions = (connection: connection): unit =>
  connection.activeSubscriptions =
    connection.activeSubscriptions->Array.filter(subscription =>
      switch subscription {
      | Subscriptions.SubscribeParams.User(_) => false
      | _ => true
      }
    )

// Track the subscription (deduped by key, so it survives reconnects) and send it.
// While disconnected the send is a no-op — tracking re-sends it on the next open,
// so this always reports `Ok`.
let subscribe = (connection: connection, params: Subscriptions.SubscribeParams.t): result<
  unit,
  SdkError.t,
> => {
  let key = Subscriptions.subscriptionKey(params)
  let alreadyTracked =
    connection.activeSubscriptions->Array.some(existing =>
      Subscriptions.subscriptionKey(existing) == key
    )
  if !alreadyTracked {
    connection.activeSubscriptions->Array.push(params)
  }
  let _ = sendMessage(connection, Subscriptions.Subscribe(params))
  Ok()
}

// Stop tracking the matching subscription(s) and send the unsubscribe (a no-op on
// the wire if already disconnected — tracking is updated either way).
let unsubscribe = (connection: connection, params: Subscriptions.UnsubscribeParams.t): result<
  unit,
  SdkError.t,
> => {
  connection.activeSubscriptions =
    connection.activeSubscriptions->Array.filter(sub =>
      !Subscriptions.matchesUnsubscribe(sub, params)
    )
  switch sendMessage(connection, Subscriptions.Unsubscribe(params)) {
  | Ok() => Ok()
  | Error(_) => Ok()
  }
}

// Send an application-level ping immediately (the heartbeat sends these on an
// interval; this is for manual liveness checks).
let ping = (connection: connection): result<unit, SdkError.t> =>
  sendMessage(connection, Subscriptions.Ping)

// Close the connection and suppress reconnection. Idempotent.
let disconnect = (connection: connection): unit => {
  connection.closedByUser = true
  clearTimers(connection)
  connection.socket->Option.forEach(socket =>
    socket->WebSocketClient.closeWith(~code=1000, ~reason="Client disconnect")
  )
  connection.socket = None
}
