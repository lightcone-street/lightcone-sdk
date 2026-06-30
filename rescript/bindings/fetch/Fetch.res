// Binding to the platform global `fetch` (Node 22, Bun, and browsers all ship it).
// The SDK uses Fetch for all HTTP — no axios, no node/browser split. Request
// options are a ReScript record whose optional fields compile to a plain JS init
// object. `AbortController`/`AbortSignal.timeout` back the request timeout.

type response
type headers
type abortController
type abortSignal

type requestInit = {
  method?: string,
  headers?: Dict.t<string>,
  body?: string,
  // "include" | "omit" | "same-origin" — credentials govern cookie forwarding.
  credentials?: string,
  signal?: abortSignal,
}

@val external fetch: (string, requestInit) => promise<response> = "fetch"

@get external status: response => int = "status"
@get external ok: response => bool = "ok"
@get external statusText: response => string = "statusText"
@get external responseHeaders: response => headers = "headers"
@send external json: response => promise<JSON.t> = "json"
@send external text: response => promise<string> = "text"

// Headers.get returns `null` when the header is absent.
@send external getHeader: (headers, string) => Null.t<string> = "get"

@new external makeAbortController: unit => abortController = "AbortController"
@get external signal: abortController => abortSignal = "signal"
@send external abort: abortController => unit = "abort"

// AbortSignal.timeout(ms) — a self-aborting signal for request timeouts.
@scope("AbortSignal") @val external timeoutSignal: int => abortSignal = "timeout"
