// Orderbook domain types — the Hyperliquid-style book-aggregation view and the
// best-bid/ask ticker.

// ── Book aggregation (Hyperliquid-style) ──────────────────────────────────────
// `(None, None)` is full precision. `nSigFigs` must be 2, 3, 4, or 5; `mantissa`
// must be 1, 2, or 5 and is only valid with `nSigFigs = 5`. `(Some(5), None)`
// normalizes to `(Some(5), Some(1))`.
module Aggregation: {
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

// ── Ticker ────────────────────────────────────────────────────────────────────
// Best bid/ask/mid for an orderbook — the domain twin of the WS `ticker`
// payload, with prices as Decimal strings.
module Ticker: {
  @genType
  type t = {
    orderbookId: Shared.orderBookId,
    bestBid?: string,
    bestAsk?: string,
    midPrice?: string,
  }
}
