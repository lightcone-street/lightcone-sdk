// Orderbook domain types — the Hyperliquid-style book-aggregation view and the
// best-bid/ask ticker. Prices stay as Decimal strings.

// ── Book aggregation (Hyperliquid-style) ──────────────────────────────────────
// `(None, None)` is full precision. `nSigFigs` must be 2, 3, 4, or 5; `mantissa`
// must be 1, 2, or 5 and is only valid with `nSigFigs = 5`. `(Some(5), None)`
// normalizes to `(Some(5), Some(1))`.
module Aggregation = {
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

// ── Ticker ────────────────────────────────────────────────────────────────────
// Best bid/ask/mid for an orderbook — the domain twin of the WS `ticker`
// payload (`Messages.Ticker.t`), with prices as Decimal strings.
module Ticker = {
  type t = {
    orderbookId: Shared.orderBookId,
    bestBid?: string,
    bestAsk?: string,
    midPrice?: string,
  }
}
