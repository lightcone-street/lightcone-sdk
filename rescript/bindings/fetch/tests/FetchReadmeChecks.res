// Compile-guard for README.md — a compile-only file (no test blocks). If a README
// snippet drifts from the actual binding signature, `rescript build` fails here.

// Quick start: fetch, check `ok`, read the body as JSON.t.
let _quickStart = async (url: string) => {
  let response = await Fetch.fetch(url, {method: "GET"})
  if Fetch.ok(response) {
    let body = await Fetch.json(response) // body : JSON.t — pattern-match to read it
    Some(body)
  } else {
    None
  }
}

// `fetch` rejects on a network/URL failure — recover the message via `JsExn`.
let _withRecovery = async (url: string) =>
  switch await Fetch.fetch(url, {method: "GET"}) {
  | response => Ok(response)
  | exception JsExn(error) => Error(error->JsExn.message->Option.getOr("network error"))
  }

// `response` is an opaque handle — read it with accessor functions, never field access.
let _readResponse = (response: Fetch.response) => {
  let _status: int = Fetch.status(response)
  let _ok: bool = Fetch.ok(response)
  let _statusText: string = Fetch.statusText(response)
  let _headers: Fetch.headers = Fetch.responseHeaders(response)
}

// Bodies are promises — `await` them; the body can be read once.
let _readBodies = async (response: Fetch.response) => {
  let _text: string = await Fetch.text(response)
  let _json: JSON.t = await Fetch.json(response)
}

// `JSON.t` is decoded by pattern-matching (or `JSON.Decode.*`).
let _decodeOk = (json: JSON.t) =>
  switch json {
  | JSON.Object(fields) =>
    switch fields->Dict.get("ok") {
    | Some(JSON.Boolean(value)) => Some(value)
    | _ => None
    }
  | _ => None
  }

// `getHeader` returns `Null.t<string>` — convert with `Null.toOption`.
let _contentType = (response: Fetch.response): option<string> =>
  Fetch.responseHeaders(response)->Fetch.getHeader("content-type")->Null.toOption

// `requestInit` is an all-optional record — set only the fields you need.
let _justMethod: Fetch.requestInit = {method: "GET"}
let _bare: Fetch.requestInit = {method: ?None} // compiles to {}
let _full: Fetch.requestInit = {
  method: "POST",
  headers: Dict.fromArray([("content-type", "application/json")]),
  body: `{"ping":true}`,
  credentials: "include", // cookie forwarding
  signal: Fetch.timeoutSignal(180000), // self-aborting timeout
}

// Abort / timeout primitives.
let _abort = () => {
  let controller = Fetch.makeAbortController()
  let _signal: Fetch.abortSignal = Fetch.signal(controller)
  controller->Fetch.abort
}
let _timeout: Fetch.abortSignal = Fetch.timeoutSignal(5000)

// Escape hatch: read a header helper the binding doesn't cover, via an ad-hoc %raw.
let _readSetCookies: Fetch.headers => array<string> = %raw("(h) => h.getSetCookie?.() ?? []")
