# `WebSocketClient` binding tests

Runtime tests for the platform `WebSocket` binding. They exercise the **actual binding** (as a
ReScript consumer would) and run the compiled output under **Bun** — catching both type errors
(`rescript build`) and runtime errors (wrong JS name / arg shape, at `bun test`).

These are **smoke** tests: they construct a socket against an unreachable URL
(`ws://localhost:1`), prove the constructor / setters / getters / `close` are callable with the
right JS names and arg shapes, and tear it down. No live server is involved, so the message
**round-trip is not exercised** (see the matrix below).

## Run

```bash
# from the rescript SDK root, build first, then run (note the ./ prefix)
./node_modules/.bin/rescript build
bun test ./bindings/websocket/tests/WebSocketClientTest.res.mjs
```

The `./` prefix is required: `bun test` treats a bare path as a name filter.

## Coverage matrix

**Behaviorally tested:** none — without a live peer there is no observable effect to assert. The
suite proves the JS surface (names + arg shapes), not behavior.

**Smoke only** (callable with the correct JS name + the instance still works):
- `make` — constructs a `WebSocket`; the returned handle is usable.
- `readyState` — asserted to be an `int` in `0..3` (`0` CONNECTING right after construction).
- `setOnOpen` / `setOnMessage` / `setOnError` / `setOnClose` — all four setters are callable with
  correctly-typed callbacks; the `data` / `code` / `reason` event accessors type-check inside
  them. (`onerror` is set to a no-op so the expected connection refusal isn't an unhandled error.)
- `close` — callable on a CONNECTING socket (aborts the attempt).

**Not runtime-tested (reason):**
- The **message round-trip** (`send` -> server -> `onmessage` delivering `data`) — needs a **live
  WebSocket server**. `send` is additionally unsafe to call at runtime while the socket is
  CONNECTING (the WHATWG spec throws `InvalidStateError`), so it stays compile-only here.
- `closeWith(~code, ~reason)` — needs an OPEN connection to observe a clean close frame; not
  reachable against an unreachable URL.

`WebSocketClientReadmeChecks.res` compile-guards every README snippet — including `send` and
`closeWith` — against the real signatures, so the docs cannot silently drift from the API.
