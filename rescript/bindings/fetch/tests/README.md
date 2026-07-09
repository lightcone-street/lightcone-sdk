# `Fetch` binding tests

Runtime tests for the platform `fetch` binding. They exercise the **actual binding** (as a
ReScript consumer would) and run the compiled output under **Bun** — catching both type errors
(`rescript build`) and runtime errors (wrong JS name / arg order / return shape, at `bun test`).

The suite is deterministic and **offline**: it drives `fetch` against `data:` URLs, which Node /
Bun / browsers all resolve with zero network. `data:` URLs echo their declared content-type header
and body verbatim, so status / headers / body assertions are fully reproducible.

## Run

```bash
# from the rescript SDK root, build first, then run (note the ./ prefix)
./node_modules/.bin/rescript build
bun test ./bindings/fetch/tests/FetchTest.res.mjs
```

The `./` prefix is required: `bun test` treats a bare path as a name filter.

## Coverage matrix

**Behaviorally tested:**
- `fetch` — driven against `data:` URLs; also asserted to **reject** on an invalid URL
  (`http://`), caught as `| exception JsExn(e)`.
- `status` / `ok` / `statusText` — asserted (`200` / `true` / `"OK"`) on a 200 `data:` response.
- `text` — asserted to read the body as a string.
- `json` — asserted to decode the body to `JSON.t`, then pattern-matched (`ok:bool`, `n:number`).
- `responseHeaders` / `getHeader` — asserted to read a present header (`content-type`) and to
  return `null` (`-> None`) for an absent one.
- `requestInit` — both a populated record (`method` + `headers`) and a bare `{method: ?None}`
  (compiles to `{}`) are passed through a real call.

**Smoke only** (callable + marshals into a resolving `fetch`):
- `makeAbortController` / `signal` / `abort` — a controller's signal is attached to a real
  request that resolves, and `abort` is called; the abort **effect** is not asserted.
- `timeoutSignal` — attached to a real request that resolves; the timeout **effect** is not
  asserted.

**Not runtime-tested (reason):** the abort/timeout **effect** (a request actually being torn
down) — `data:` URLs resolve instantly and don't honor in-process abort, so asserting it would
need a slow live server. The values are proven to marshal correctly into `fetch`; the cancellation
behavior is the platform's.

`FetchReadmeChecks.res` compile-guards every README snippet against the real signatures, so the
docs cannot silently drift from the API.
