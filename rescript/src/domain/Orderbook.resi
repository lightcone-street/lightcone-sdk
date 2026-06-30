// Orderbook domain — live depth (REST) + the WS snapshot frame. Public surface:
// the `BookAggregation` view utilities, the gentype-exported level / depth / book
// types, the frame→aggregation helper, and the depth client functions. `orderBook`
// keeps its spice codec (`orderBook_decode` is consumed by the WS layer); the
// other codecs and the query helper are internal.

// ── Book aggregation (Hyperliquid-style) ──────────────────────────────────────
// `(None, None)` is full precision. `nSigFigs` must be 2, 3, 4, or 5; `mantissa`
// must be 1, 2, or 5 and is only valid with `nSigFigs = 5`. `(Some(5), None)`
// normalizes to `(Some(5), Some(1))`.
module BookAggregation: {
  @genType
  type t = {
    nSigFigs?: int,
    mantissa?: int,
  }

  // Full precision (no aggregation).
  let full: t

  // Validate against the backend contract, returning the normalized form.
  let validate: (option<int>, option<int>) => result<t, string>

  // Normalized form: `(5, None)` becomes `(5, Some(1))`; otherwise unchanged.
  let normalized: t => t

  // Aggregation identified by an incoming frame's tags (untagged = full).
  let fromFrame: (option<int>, option<int>) => t

  // Whether this is the full-precision (no aggregation) view.
  let isFull: t => bool

  // Stable subscription-key suffix: "full", "sig2".."sig4", or "sig5m1"/2/5.
  let keySuffix: t => string
}

// ── Level types ───────────────────────────────────────────────────────────────
// A single WS price level — `side` is explicit in WS frames.
@genType
type wsBookLevel = {
  side: Shared.Side.t,
  price: string,
  size: string,
}

// A single REST depth level — side is implicit from the bids/asks array.
@genType
type restBookLevel = {
  price: string,
  size: string,
  orders?: float,
}

// ── REST depth types ──────────────────────────────────────────────────────────
// Price/size display decimals returned by the depth endpoint.
@genType
type orderbookDepthDecimals = {
  price: float,
  size: float,
}

// Live orderbook depth (returned directly; no domain conversion).
@genType
type orderbookDepthResponse = {
  orderbookId: Shared.orderBookId,
  marketPubkey?: string,
  bestBid?: string,
  bestAsk?: string,
  spread?: string,
  tickSize?: string,
  bids: array<restBookLevel>,
  asks: array<restBookLevel>,
  decimals?: orderbookDepthDecimals,
}

// ── WS snapshot frame ─────────────────────────────────────────────────────────
// Snapshot-only stream: every data frame carries the full top-20 levels per side
// and replaces the previous book wholesale. `seq` is strictly increasing but
// non-contiguous; the initial snapshot after each (re)subscribe is `seq: 0`.
// Keeps its spice codec — `orderBook_decode` is consumed by the WS message layer.
@genType @spice
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
let aggregationOfOrderBook: orderBook => BookAggregation.t

// ── Client functions ──────────────────────────────────────────────────────────
// Live orderbook depth, optionally aggregated. `depth` is capped server-side at
// 20 levels per side. Invalid aggregation combinations are rejected client-side
// before any request is made.
let get: (
  Client.t,
  ~orderbookId: string,
  ~depth: int=?,
  ~aggregation: BookAggregation.t=?,
  ~cookieHeader: string=?,
) => promise<result<orderbookDepthResponse, SdkError.t>>

// Convenience: depth with an explicit aggregation view.
let getWithAggregation: (
  Client.t,
  ~orderbookId: string,
  ~aggregation: BookAggregation.t,
  ~depth: int=?,
  ~cookieHeader: string=?,
) => promise<result<orderbookDepthResponse, SdkError.t>>

// Convenience: depth with an explicit auth cookie (Node/Bun, no cookie jar).
let getWithCookies: (
  Client.t,
  ~orderbookId: string,
  ~cookieHeader: string,
  ~depth: int=?,
  ~aggregation: BookAggregation.t=?,
) => promise<result<orderbookDepthResponse, SdkError.t>>
