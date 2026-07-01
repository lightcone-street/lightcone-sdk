// PriceHistoryState — rolling candle series per (orderbook, resolution) from WS `PriceHistory`
// events (mirrors rust price_history/state.rs). Snapshot replaces the series; update
// tail-dedupes by timestamp. Points are `PriceHistory.lineData` (midpoint-else-close values).

@genType.opaque
type t

@genType
let make: unit => t

// Replace the entire series for this (orderbook, resolution).
@genType
let applySnapshot: (
  t,
  ~orderbookId: Shared.orderBookId,
  ~resolution: Shared.Resolution.t,
  ~candles: array<PriceHistory.orderbookPriceCandle>,
) => unit

// Append the candle, or roll up the trailing point when its timestamp matches.
@genType
let applyUpdate: (
  t,
  ~orderbookId: Shared.orderBookId,
  ~resolution: Shared.Resolution.t,
  ~candle: PriceHistory.orderbookPriceCandle,
) => unit

@genType
let get: (
  t,
  ~orderbookId: Shared.orderBookId,
  ~resolution: Shared.Resolution.t,
) => option<array<PriceHistory.lineData>>

@genType
let clear: t => unit
