open RescriptBun.Test
open RescriptBun.Test.Expect

// SMOKE tests for the platform `WebSocket` binding — run the compiled .res.mjs under Bun
// to prove the JS names (the `WebSocket` ctor, the `onopen`/`onmessage`/`onerror`/`onclose`
// setters, the `readyState`/`data`/`code`/`reason` getters, `close`) and their arg shapes.
//
// We construct against an unreachable URL (`ws://localhost:1`) and never await a live
// connection. The message round-trip (`send` -> server -> `onmessage` with `data`) needs a
// real server and is intentionally NOT runtime-tested here (see tests/README.md). `send` is
// also omitted at runtime on purpose: per the WHATWG spec it throws `InvalidStateError` while
// the socket is still `CONNECTING`. We set `onerror` to a no-op so the (expected) refused
// connection doesn't surface as an unhandled error.

describe("WebSocketClient (smoke)", () => {
  test("make constructs a socket and readyState is an int in 0..3", () => {
    let socket = WebSocketClient.make("ws://localhost:1")
    socket->WebSocketClient.setOnError(_event => ()) // swallow the expected refusal
    let state = WebSocketClient.readyState(socket)
    expect(state >= 0 && state <= 3)->toBe(true) // 0 CONNECTING right after construction
    socket->WebSocketClient.close // callable on a CONNECTING socket (aborts the attempt)
  })

  test("every handler setter is callable and event accessors type-check", () => {
    let socket = WebSocketClient.make("ws://localhost:1")
    socket->WebSocketClient.setOnError(_event => ()) // swallow the expected refusal
    socket->WebSocketClient.setOnOpen(() => ())
    socket->WebSocketClient.setOnMessage(event => {
      let _payload: string = WebSocketClient.data(event)
    })
    socket->WebSocketClient.setOnClose(event => {
      let _code: int = WebSocketClient.code(event)
      let _reason: string = WebSocketClient.reason(event)
    })
    expect(WebSocketClient.readyState(socket) >= 0)->toBe(true)
    socket->WebSocketClient.close
  })
})
