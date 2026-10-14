// Binding to the platform global `WebSocket` (Node 22, Bun, and browsers all ship
// it). This is the app-level Lightcone WebSocket (NOT a Solana RPC subscription).
// Single implementation — no node `ws` package, no node/browser split.

type t
type messageEvent
type closeEvent
type errorEvent

@new external make: string => t = "WebSocket"

@send external send: (t, string) => unit = "send"
@send external close: t => unit = "close"
@send external closeWith: (t, ~code: int, ~reason: string) => unit = "close"

@set external setOnOpen: (t, unit => unit) => unit = "onopen"
@set external setOnMessage: (t, messageEvent => unit) => unit = "onmessage"
@set external setOnError: (t, errorEvent => unit) => unit = "onerror"
@set external setOnClose: (t, closeEvent => unit) => unit = "onclose"

// readyState: 0 CONNECTING, 1 OPEN, 2 CLOSING, 3 CLOSED.
@get external readyState: t => int = "readyState"

// Text frames deliver their payload as a string on `event.data`.
@get external data: messageEvent => string = "data"
@get external code: closeEvent => int = "code"
@get external reason: closeEvent => string = "reason"
