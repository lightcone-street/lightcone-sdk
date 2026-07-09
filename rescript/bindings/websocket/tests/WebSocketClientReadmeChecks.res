// Compile-guard for README.md — a compile-only file (no test blocks). If a README
// snippet drifts from the actual binding signature, `rescript build` fails here.

// Quick start: open a socket, wire the four lifecycle handlers, send on open, close.
let _quickStart = (url: string) => {
  let socket = WebSocketClient.make(url)

  socket->WebSocketClient.setOnOpen(() => socket->WebSocketClient.send(`{"type":"subscribe"}`))
  socket->WebSocketClient.setOnMessage(event => {
    let _payload: string = WebSocketClient.data(event) // text frame payload
  })
  socket->WebSocketClient.setOnError(_event => ())
  socket->WebSocketClient.setOnClose(event => {
    let _code: int = WebSocketClient.code(event)
    let _reason: string = WebSocketClient.reason(event)
  })

  socket
}

// `readyState` is an int — compare against the documented constants (0..3).
let _isOpen = (socket: WebSocketClient.t): bool => WebSocketClient.readyState(socket) == 1

// `send` takes a string; `close` / `closeWith` end the connection.
let _send = (socket: WebSocketClient.t, text: string) => socket->WebSocketClient.send(text)
let _close = (socket: WebSocketClient.t) => socket->WebSocketClient.close
let _closeWith = (socket: WebSocketClient.t) =>
  socket->WebSocketClient.closeWith(~code=1000, ~reason="bye")

// Event accessors read their opaque event handles.
let _readMessage = (event: WebSocketClient.messageEvent): string => WebSocketClient.data(event)
let _readClose = (event: WebSocketClient.closeEvent): (int, string) => (
  WebSocketClient.code(event),
  WebSocketClient.reason(event),
)

// Escape hatch: read an event field the binding doesn't cover, via an ad-hoc external.
@get external messageType: WebSocketClient.messageEvent => string = "type"
let _messageType = (event: WebSocketClient.messageEvent): string => messageType(event)
