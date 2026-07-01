// Generic HTTP transport — Fetch-based (no axios), single implementation for
// Node 22 / Bun / browser. Mirrors the Rust SDK's `LightconeHttp`: x-request-id
// generation, cookie auth, retry policies, and `ApiResponse` unwrapping.
//
// Cookies: browsers manage the jar automatically (we send `credentials: include`).
// Node/Bun have no jar, so we capture the `lightcone-token` Set-Cookie after login
// and re-send it as a `Cookie` header — exactly what the Rust native path does.

let userCookieName = "lightcone-token"
let defaultTimeoutMs = 180000

type retryPolicy =
  | NoRetry
  | Idempotent

type cookieStore = {mutable token: option<string>}

// The transport handle (opaque to TS — see the `.resi`).
type t = {
  baseUrl: string,
  cookies: cookieStore,
}

// ── Small platform helpers ────────────────────────────────────────────────────
// `setTimeout` and `encodeURIComponent` are stdlib globals (Stdlib includes
// Stdlib_Global); `crypto.randomUUID` comes from the WebCrypto binding.
let sleep = (ms: int): promise<unit> =>
  Promise.make((resolve, _reject) => setTimeout(() => resolve(), ms)->ignore)

let parseJson = (text: string): option<JSON.t> =>
  switch JSON.parseOrThrow(text) {
  | json => Some(json)
  | exception _ => None
  }

// undici/Bun expose `headers.getSetCookie()`; browsers don't (and don't need to).
let readSetCookies: Fetch.headers => array<string> = %raw(`function (headers) {
  return (headers && typeof headers.getSetCookie === "function") ? headers.getSetCookie() : [];
}`)

// ── Construction & session ────────────────────────────────────────────────────
let make = (baseUrl: string): t => {
  baseUrl: baseUrl->String.replaceRegExp(/\/+$/, ""),
  cookies: {token: None},
}

let baseUrl = (http: t): string => http.baseUrl
let authToken = (http: t): option<string> => http.cookies.token
let setAuthToken = (http: t, token: string): unit => http.cookies.token = Some(token)
let clearAuthToken = (http: t): unit => http.cookies.token = None

// ── Internals ─────────────────────────────────────────────────────────────────
let buildUrl = (baseUrl: string, path: string, query: array<(string, string)>): string =>
  switch query {
  | [] => baseUrl ++ path
  | params =>
    let queryString =
      params
      ->Array.map(((key, value)) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`)
      ->Array.join("&")
    `${baseUrl}${path}?${queryString}`
  }

let isRetryable = (status: int): bool =>
  switch status {
  | 429 | 502 | 503 | 504 => true
  | _ => false
  }

// 200ms initial, ×2 backoff, capped at 10s, ±25% jitter (matches RetryConfig).
let delayForAttempt = (attempt: int): int => {
  let base = 200.0 *. Math.pow(2.0, ~exp=Int.toFloat(attempt))
  let capped = Math.min(base, 10000.0)
  let jitterRange = capped *. 0.25
  let jitter = (Math.random() -. 0.5) *. 2.0 *. jitterRange
  Math.max(capped +. jitter, 0.0)->Float.toInt
}

let statusToError = (status: int, body: string): SdkError.t =>
  switch status {
  | 401 => Http(Unauthorized)
  | 404 => Http(NotFound(body))
  | 400 => Http(BadRequest(body))
  | 429 => Http(RateLimited({retryAfterMs: None}))
  | _ => Http(ServerError({status, body}))
  }

let captureAuthCookie = (http: t, response: Fetch.response): unit => {
  let prefix = userCookieName ++ "="
  readSetCookies(Fetch.responseHeaders(response))->Array.forEach(cookie =>
    if String.startsWith(cookie, prefix) {
      let rest = String.slice(cookie, ~start=String.length(prefix), ~end=String.length(cookie))
      let value = switch String.indexOf(rest, ";") {
      | -1 => rest
      | index => String.slice(rest, ~start=0, ~end=index)
      }
      http.cookies.token = Some(value)
    }
  )
}

let sendRequest = async (url: string, init: Fetch.requestInit): result<Fetch.response, string> =>
  switch await Fetch.fetch(url, init) {
  | response => Ok(response)
  | exception JsExn(error) => Error(error->JsExn.message->Option.getOr("network error"))
  }

let readText = async (response: Fetch.response): option<string> =>
  switch await Fetch.text(response) {
  | text => Some(text)
  | exception _ => None
  }

let request = async (
  http: t,
  ~method: string,
  ~path: string,
  ~query: array<(string, string)>=[],
  ~body: option<JSON.t>=?,
  ~retry: retryPolicy=NoRetry,
  ~cookieHeader: option<string>=?,
  ~decode: JSON.t => result<'a, Spice.decodeError>,
): result<'a, SdkError.t> => {
  let url = buildUrl(http.baseUrl, path, query)
  let maxAttempts = switch retry {
  | NoRetry => 1
  | Idempotent => 4
  }

  let rec attempt = async (n: int): result<'a, SdkError.t> => {
    let headers = Dict.make()
    Dict.set(headers, "x-request-id", WebCrypto.randomUUID())
    switch body {
    | Some(_) => Dict.set(headers, "content-type", "application/json")
    | None => ()
    }
    switch cookieHeader {
    | Some(raw) => Dict.set(headers, "cookie", raw)
    | None =>
      switch http.cookies.token {
      | Some(token) => Dict.set(headers, "cookie", `${userCookieName}=${token}`)
      | None => ()
      }
    }
    let init: Fetch.requestInit = {
      method,
      headers,
      body: ?(body->Option.map(value => JSON.stringify(value))),
      credentials: "include",
      signal: Fetch.timeoutSignal(defaultTimeoutMs),
    }

    switch await sendRequest(url, init) {
    | Error(networkMessage) =>
      if n < maxAttempts {
        await sleep(delayForAttempt(n - 1))
        await attempt(n + 1)
      } else {
        Error(Http(Network(networkMessage)))
      }
    | Ok(response) =>
      let status = Fetch.status(response)
      captureAuthCookie(http, response)
      if isRetryable(status) && n < maxAttempts {
        await sleep(delayForAttempt(n - 1))
        await attempt(n + 1)
      } else {
        let text = await readText(response)
        switch text->Option.flatMap(parseJson) {
        | Some(json) => SdkError.parseApiResponse(decode, json)
        | None =>
          if status >= 200 && status < 300 {
            Error(Decode("expected a JSON response body"))
          } else {
            Error(statusToError(status, text->Option.getOr("")))
          }
        }
      }
    }
  }
  await attempt(1)
}

// ── Public verbs ──────────────────────────────────────────────────────────────
// GET defaults to the idempotent retry policy; POST defaults to no retries.
let get = (
  http: t,
  ~path: string,
  ~query: array<(string, string)>=[],
  ~retry: retryPolicy=Idempotent,
  ~cookieHeader: option<string>=?,
  ~decode: JSON.t => result<'a, Spice.decodeError>,
): promise<result<'a, SdkError.t>> =>
  request(http, ~method="GET", ~path, ~query, ~retry, ~cookieHeader?, ~decode)

let post = (
  http: t,
  ~path: string,
  ~body: JSON.t,
  ~retry: retryPolicy=NoRetry,
  ~cookieHeader: option<string>=?,
  ~decode: JSON.t => result<'a, Spice.decodeError>,
): promise<result<'a, SdkError.t>> =>
  request(http, ~method="POST", ~path, ~body, ~retry, ~cookieHeader?, ~decode)
