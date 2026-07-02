// Orderbook domain — live depth (REST) + book aggregation + the WS snapshot
// frame (mirrors the Rust `domain/orderbook`).
//
// Reference shape (same as Trade.res). Prices/sizes stay as wire strings;
// counts/indices/seq are floats. The `orderBookPair` market-structure type and
// its conversion live in Market.res (`Market.orderBookPair`) because they depend
// on the market token types — keeping them there avoids a module cycle.
//
// Depth is capped server-side at 20 levels per side. Pass `BookAggregation.full`
// for the raw (unaggregated) book.

// ── Book aggregation (Hyperliquid-style) ──────────────────────────────────────
// `(None, None)` is full precision. `nSigFigs` must be 2, 3, 4, or 5; `mantissa`
// must be 1, 2, or 5 and is only valid with `nSigFigs = 5`. `(Some(5), None)`
// normalizes to `(Some(5), Some(1))`.
module BookAggregation = {
  type t = {
    nSigFigs?: int,
    mantissa?: int,
  }

  // Full precision (no aggregation).
  let full: t = {}

  // Validate against the backend contract, returning the normalized form.
  // Invalid combinations are rejected server-side, so validate before sending.
  let validate = (nSigFigs: option<int>, mantissa: option<int>): result<t, string> =>
    switch (nSigFigs, mantissa) {
    | (None, None) => Ok(full)
    | (None, Some(_)) => Error("mantissa is only valid when nSigFigs is 5")
    | (Some(value), None) if value >= 2 && value <= 4 => Ok({nSigFigs: value})
    | (Some(value), Some(_)) if value >= 2 && value <= 4 =>
      Error("mantissa is only valid when nSigFigs is 5")
    | (Some(5), None) => Ok({nSigFigs: 5, mantissa: 1})
    | (Some(5), Some(mantissaValue))
      if mantissaValue == 1 || mantissaValue == 2 || mantissaValue == 5 =>
      Ok({nSigFigs: 5, mantissa: mantissaValue})
    | (Some(5), Some(_)) => Error("mantissa must be 1, 2, or 5")
    | (Some(_), _) => Error("nSigFigs must be 2, 3, 4, 5, or omitted")
    }

  // Normalized form: `(5, None)` becomes `(5, Some(1))`; otherwise unchanged.
  // Lenient — never errors.
  let normalized = (aggregation: t): t =>
    switch (aggregation.nSigFigs, aggregation.mantissa) {
    | (Some(5), None) => {nSigFigs: 5, mantissa: 1}
    | _ => aggregation
    }

  // Aggregation identified by an incoming frame's tags (untagged = full).
  let fromFrame = (nSigFigs: option<int>, mantissa: option<int>): t =>
    normalized({nSigFigs: ?nSigFigs, mantissa: ?mantissa})

  // Whether this is the full-precision (no aggregation) view.
  let isFull = (aggregation: t): bool => {
    let normalizedAggregation = normalized(aggregation)
    normalizedAggregation.nSigFigs == None && normalizedAggregation.mantissa == None
  }

  // Stable subscription-key suffix: "full", "sig2".."sig4", or "sig5m1"/2/5.
  let keySuffix = (aggregation: t): string => {
    let normalizedAggregation = normalized(aggregation)
    switch (normalizedAggregation.nSigFigs, normalizedAggregation.mantissa) {
    | (None, None) => "full"
    | (Some(nSigFigs), None) => `sig${Int.toString(nSigFigs)}`
    | (Some(nSigFigs), Some(mantissa)) => `sig${Int.toString(nSigFigs)}m${Int.toString(mantissa)}`
    | (None, Some(_)) => "invalid"
    }
  }
}

// ── Level types ───────────────────────────────────────────────────────────────
// A single WS price level — `side` is explicit in WS frames.
@spice
type wsBookLevel = {
  side: Shared.Side.t,
  price: string,
  size: string,
}

// A single REST depth level — side is implicit from the bids/asks array.
@spice
type restBookLevel = {
  price: string,
  size: string,
  orders?: float,
}

// ── REST depth wire+domain types ──────────────────────────────────────────────
// Price/size display decimals returned by the depth endpoint.
@spice
type orderbookDepthDecimals = {
  price: float,
  size: float,
}

// Live orderbook depth (returned directly; no domain conversion).
@spice
type orderbookDepthResponse = {
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  @spice.key("market_pubkey") marketPubkey?: string,
  @spice.key("best_bid") bestBid?: string,
  @spice.key("best_ask") bestAsk?: string,
  spread?: string,
  @spice.key("tick_size") tickSize?: string,
  bids: array<restBookLevel>,
  asks: array<restBookLevel>,
  decimals?: orderbookDepthDecimals,
}

// ── WS snapshot frame ─────────────────────────────────────────────────────────
// Snapshot-only stream: every data frame carries the full top-20 levels per side
// and replaces the previous book wholesale. `seq` is strictly increasing but
// non-contiguous; the initial snapshot after each (re)subscribe is `seq: 0`.
@spice
type orderBook = {
  @spice.key("orderbook_id") id: Shared.orderBookId,
  @spice.default(false) @spice.key("is_snapshot") isSnapshot: bool,
  @spice.default(0.0) seq: float,
  @spice.default(false) resync: bool,
  @spice.default([]) bids: array<wsBookLevel>,
  @spice.default([]) asks: array<wsBookLevel>,
  @spice.key("n_sig_figs") nSigFigs?: int,
  mantissa?: int,
}

// The aggregation view a frame belongs to (untagged = full precision).
let aggregationOfOrderBook = (book: orderBook): BookAggregation.t =>
  BookAggregation.fromFrame(book.nSigFigs, book.mantissa)

// ── Ticker (orderbook/ticker.rs) ──────────────────────────────────────────────
// Best bid/ask/mid for an orderbook — the domain twin of the WS `ticker`
// payload (`Messages.wsTicker`), with prices as Decimal strings.
type tickerData = {
  orderbookId: Shared.orderBookId,
  bestBid?: string,
  bestAsk?: string,
  midPrice?: string,
}

// ── Client functions ──────────────────────────────────────────────────────────
let optionalQuery = (query, key, value) =>
  value->Option.forEach(value => query->Array.push((key, value)))

// Live orderbook depth, optionally aggregated. `depth` is capped server-side at
// 20 levels per side. Invalid aggregation combinations are rejected client-side
// before any request is made. Only `depth`, `nSigFigs`, and `mantissa` are sent.
let get = async (
  client: Client.t,
  ~orderbookId: string,
  ~depth: option<int>=?,
  ~aggregation: BookAggregation.t=BookAggregation.full,
  ~cookieHeader: option<string>=?,
): result<orderbookDepthResponse, SdkError.t> =>
  switch BookAggregation.validate(aggregation.nSigFigs, aggregation.mantissa) {
  | Error(message) => Error(SdkError.Validation(message))
  | Ok(validated) => {
      let query: array<(string, string)> = []
      optionalQuery(query, "depth", depth->Option.map(value => Int.toString(value)))
      optionalQuery(query, "nSigFigs", validated.nSigFigs->Option.map(value => Int.toString(value)))
      optionalQuery(query, "mantissa", validated.mantissa->Option.map(value => Int.toString(value)))
      await Http.get(
        client.http,
        ~path=`/api/orderbook/${orderbookId}`,
        ~query,
        ~cookieHeader?,
        ~decode=orderbookDepthResponse_decode,
      )
    }
  }

// Convenience: depth with an explicit aggregation view.
let getWithAggregation = (
  client: Client.t,
  ~orderbookId: string,
  ~aggregation: BookAggregation.t,
  ~depth: option<int>=?,
  ~cookieHeader: option<string>=?,
): promise<result<orderbookDepthResponse, SdkError.t>> =>
  get(client, ~orderbookId, ~depth?, ~aggregation, ~cookieHeader?)

// Convenience: depth with an explicit auth cookie (Node/Bun, no cookie jar).
let getWithCookies = (
  client: Client.t,
  ~orderbookId: string,
  ~cookieHeader: string,
  ~depth: option<int>=?,
  ~aggregation: BookAggregation.t=BookAggregation.full,
): promise<result<orderbookDepthResponse, SdkError.t>> =>
  get(client, ~orderbookId, ~depth?, ~aggregation, ~cookieHeader=?Some(cookieHeader))

// The stateful sorted-book (`OrderbookState`, mirroring orderbook/state.rs) lives in its own
// module `src/domain/OrderbookState.res`, fed from a `Ws.connect` `~onMessage` closure.
