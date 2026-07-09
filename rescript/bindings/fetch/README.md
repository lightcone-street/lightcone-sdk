# `Fetch` — bindings to the platform global `fetch`

ReScript bindings to the platform global [`fetch`](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)
(Node 22, Bun, and browsers all ship it). The Lightcone SDK uses `Fetch` for **all**
HTTP — there is no axios and no node/browser split (see `src/Http.res`). Request
options are a ReScript record whose optional fields compile to a plain JS init object;
`AbortController` / `AbortSignal.timeout` back request timeouts.

Written for someone who knows the Fetch API but is new to ReScript bindings. We bind
only what the SDK needs; the platform `fetch` has more (streaming bodies, `Request`
objects, `FormData`, …).

> Compatibility: the platform global `fetch` — no `node-fetch`/`undici` install, no
> polyfill. `Headers.getSetCookie()` (used by `src/Http.res`) needs Node 18.7+/Bun.
>
> See [`tests/`](./tests) for the runnable suite and [`tests/README.md`](./tests/README.md)
> for the coverage matrix.

## Setup

The module is part of the SDK package (no extra `rescript.json` dependency).
`fetch`, `AbortController`, and `AbortSignal` must exist on the platform (they do on
Node 22 / Bun / browsers). Reference the module directly — `Fetch.fetch`.

`fetch` always takes **two** arguments — the URL and a `requestInit` record. There are
no optional trailing arguments, so there is no trailing-`()` calling convention to
remember here. Build the record by setting only the fields you use (below).

## How to read returned values

- **`fetch(url, init)` returns `promise<response>`** — `await` it. A network/URL
  failure **rejects**, so wrap the `await` in a `switch` with an `| exception JsExn(e)`
  arm to recover the message:

  ```rescript
  switch await Fetch.fetch(url, {method: "GET"}) {
  | response => Ok(response)
  | exception JsExn(error) => Error(error->JsExn.message->Option.getOr("network error"))
  }
  ```

- **`response` is an opaque handle** — read it with accessor functions, never field
  access: `Fetch.status(response)` (`int`), `Fetch.ok(response)` (`bool`),
  `Fetch.statusText(response)` (`string`), `Fetch.responseHeaders(response)` (a
  `headers` handle).

- **Bodies are promises** — `Fetch.text(response)` returns `promise<string>` and
  `Fetch.json(response)` returns `promise<JSON.t>`; `await` either. The body can be read
  **once**.

- **`JSON.t` is decoded by pattern-matching** (or `JSON.Decode.*`):

  ```rescript
  switch await Fetch.json(response) {
  | JSON.Object(fields) =>
    switch fields->Dict.get("ok") {
    | Some(JSON.Boolean(value)) => Some(value)
    | _ => None
    }
  | _ => None
  }
  ```

- **`headers` is opaque** — `Fetch.getHeader(headers, name)` returns `Null.t<string>`
  (JS `null` when the header is absent). Convert with `Null.toOption` to get an
  `option<string>`:

  ```rescript
  Fetch.responseHeaders(response)->Fetch.getHeader("content-type")->Null.toOption
  // Some("application/json;charset=utf-8") | None
  ```

- **`requestInit` is an all-optional record** — build it by setting only the fields you
  need; omit the rest. To splice an `option` directly into a field, use the `?`
  punning (`{field: ?someOption}`):

  ```rescript
  let init: Fetch.requestInit = {method: "GET"}                       // just one field
  let bare: Fetch.requestInit = {method: ?None}                       // compiles to {}
  let full: Fetch.requestInit = {                                     // every field
    method: "POST",
    headers: Dict.fromArray([("content-type", "application/json")]),
    body: `{"ping":true}`,
    credentials: "include",                                           // cookie forwarding
    signal: Fetch.timeoutSignal(180000),                             // self-aborting timeout
  }
  ```

## Quick start

```rescript
let response = await Fetch.fetch("https://api.example.com/health", {method: "GET"})
if Fetch.ok(response) {
  let body = await Fetch.json(response)   // body : JSON.t — pattern-match to read it
  Some(body)
} else {
  None
}
```

## Reference

### Request

| Binding | Signature | Notes |
|---|---|---|
| `fetch` | `(string, requestInit) => promise<response>` | The headline call. Rejects on network/URL errors — catch with `\| exception JsExn(e)`. |
| `requestInit` | record | All fields optional: `method?: string`, `headers?: Dict.t<string>`, `body?: string`, `credentials?: string` (`"include"` \| `"omit"` \| `"same-origin"`), `signal?: abortSignal`. |

### Response

| Binding | Signature | Notes |
|---|---|---|
| `status` | `response => int` | HTTP status code, e.g. `200`. |
| `ok` | `response => bool` | `true` for 2xx. |
| `statusText` | `response => string` | e.g. `"OK"`. |
| `responseHeaders` | `response => headers` | Opaque `headers` handle — read with `getHeader`. |
| `text` | `response => promise<string>` | `await` it. Body is consumed once. |
| `json` | `response => promise<JSON.t>` | `await`, then pattern-match the `JSON.t`. |

### Headers

| Binding | Signature | Notes |
|---|---|---|
| `getHeader` | `(headers, string) => Null.t<string>` | `null` when absent — pipe through `Null.toOption`. |

### Abort / timeout

| Binding | Signature | Notes |
|---|---|---|
| `makeAbortController` | `unit => abortController` | `new AbortController()`. |
| `signal` | `abortController => abortSignal` | The controller's `.signal` — put it in `requestInit.signal`. |
| `abort` | `abortController => unit` | Aborts the in-flight request. |
| `timeoutSignal` | `int => abortSignal` | `AbortSignal.timeout(ms)` — a self-aborting signal for request timeouts. |

## Escape hatches

- **Header helpers the binding doesn't cover** (e.g. `getSetCookie()`): add an ad-hoc
  `%raw` reader, as `src/Http.res` does:
  `let readSetCookies: Fetch.headers => array<string> = %raw("(h) => h.getSetCookie?.() ?? []")`.
- **Other init fields** (`mode`, `cache`, `redirect`, `keepalive`): add them to the
  `requestInit` record, or pass a `%raw` init object cast with `%identity`.
- **Streaming / `Request` objects / `FormData`**: not bound here — reach for the
  official `@rescript/webapi` / `rescript-webapi` `Request` / `Response` bindings.
