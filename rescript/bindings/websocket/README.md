# `WebSocketClient` — bindings to the platform global `WebSocket`

ReScript bindings to the platform global [`WebSocket`](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
(Node 22, Bun, and browsers all ship it). This is the **app-level Lightcone WebSocket** (the
realtime market/order feed) — **not** a Solana RPC subscription. There is one implementation: no
node `ws` package, no node/browser split.

Written for someone who knows the WebSocket API but is new to ReScript bindings. We bind only what
the SDK needs (open, the four lifecycle handlers, send, close, `readyState`); the platform
`WebSocket` has more (binary frames, `bufferedAmount`, sub-protocols, …).

> Compatibility: the platform global `WebSocket` — no `ws` install, no polyfill. Text frames only
> (the SDK speaks JSON); binary frames are an escape hatch (below).
>
> See [`tests/`](./tests) for the runnable suite and [`tests/README.md`](./tests/README.md) for
> the coverage matrix.

## Setup

This is a standalone package depending only on `@rescript/runtime` (the platform supplies
`WebSocket`). The global must exist (it does on Node 22 / Bun / browsers). Reference the module
directly — `WebSocketClient.make`.

No trailing-`()` calling convention is needed here: no binding takes a trailing optional argument.
`closeWith` uses **labeled** arguments (`~code`, `~reason`).

## How to read returned values

- **`make(url)` returns an opaque socket handle `t`** — never field-access it. Drive it with the
  accessor/setter functions: `socket->WebSocketClient.send(text)`, `WebSocketClient.readyState(socket)`,
  `socket->WebSocketClient.close`.

- **`readyState` is an `int`, not a variant** — compare it against the four documented constants:

  | Value | Meaning |
  |---|---|
  | `0` | `CONNECTING` (right after `make`, before `onopen`) |
  | `1` | `OPEN` (safe to `send`) |
  | `2` | `CLOSING` |
  | `3` | `CLOSED` |

  ```rescript
  let isOpen = WebSocketClient.readyState(socket) == 1
  ```

- **Handlers are installed with setter functions**, each taking a callback. The callback for a
  *message* / *close* receives an **opaque event handle** — read it with the event accessors,
  never field access:

  ```rescript
  socket->WebSocketClient.setOnMessage(event => {
    let payload = WebSocketClient.data(event) // string — the text frame body
    Console.log(payload)
  })
  socket->WebSocketClient.setOnClose(event => {
    let code = WebSocketClient.code(event)     // int
    let reason = WebSocketClient.reason(event) // string
    Console.log2(code, reason)
  })
  ```

- **`send` throws if the socket is not `OPEN`** — wait for `onopen` (or check `readyState == 1`)
  before sending. A throw surfaces as a JS exception; catch it with `| exception JsExn(e)`:

  ```rescript
  switch socket->WebSocketClient.send(text) {
  | () => Ok()
  | exception JsExn(error) => Error(error->JsExn.message->Option.getOr("socket not open"))
  }
  ```

## Quick start

```rescript
let socket = WebSocketClient.make("wss://api.example.com/ws")

socket->WebSocketClient.setOnOpen(() => socket->WebSocketClient.send(`{"type":"subscribe"}`))
socket->WebSocketClient.setOnMessage(event => Console.log(WebSocketClient.data(event)))
socket->WebSocketClient.setOnClose(event =>
  Console.log2(WebSocketClient.code(event), WebSocketClient.reason(event))
)
socket->WebSocketClient.setOnError(_event => Console.log("socket error"))
// later: socket->WebSocketClient.close
```

## Reference

### Construction & lifecycle

| Binding | Signature | Notes |
|---|---|---|
| `make` | `string => t` | `new WebSocket(url)`. Connection is async — the socket starts `CONNECTING` (`readyState == 0`). |
| `send` | `(t, string) => unit` | Sends a text frame. **Throws** (`| exception JsExn(e)`) if the socket isn't `OPEN`. |
| `close` | `t => unit` | Closes (or aborts a pending connection). |
| `closeWith` | `(t, ~code: int, ~reason: string) => unit` | Close with a status code (e.g. `1000`) and reason string. |
| `readyState` | `t => int` | `0` CONNECTING, `1` OPEN, `2` CLOSING, `3` CLOSED. |

### Handlers (setters)

Each replaces the corresponding `on*` property; the callback receives an opaque event handle
(read with the accessors below).

| Binding | Signature | Notes |
|---|---|---|
| `setOnOpen` | `(t, unit => unit) => unit` | Fires once when the connection opens. |
| `setOnMessage` | `(t, messageEvent => unit) => unit` | Fires per inbound frame; read the payload with `data`. |
| `setOnError` | `(t, errorEvent => unit) => unit` | Fires on a connection/protocol error. |
| `setOnClose` | `(t, closeEvent => unit) => unit` | Fires once on close; read `code` / `reason`. |

### Event accessors

| Binding | Signature | Notes |
|---|---|---|
| `data` | `messageEvent => string` | The text frame payload (`event.data`). |
| `code` | `closeEvent => int` | The close status code (`event.code`). |
| `reason` | `closeEvent => string` | The close reason (`event.reason`). |

## Escape hatches

- **Event fields the binding doesn't cover** (e.g. `event.type`, `errorEvent.message`): add an
  ad-hoc `@get` external next to these — `@get external messageType: WebSocketClient.messageEvent => string = "type"`.
- **Binary frames** (`ArrayBuffer` / `Blob` payloads, `binaryType`): not bound here (the SDK speaks
  text JSON). Add `@get`/`@set` externals mirroring these, or read `event.data` with a `%raw` cast.
- **Other socket properties** (`bufferedAmount`, `protocol`, `url`, sub-protocol ctor arg): add
  ad-hoc `@get`/`@new` externals as needed.
