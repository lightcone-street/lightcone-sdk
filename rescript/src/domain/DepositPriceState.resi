// DepositPriceState — deposit-token candle series per (asset, resolution) + latest per-asset
// price, from WS `DepositPrice` / `DepositAssetPrice` events (mirrors rust price_history/state.rs).
// Candles tail-dedupe by open time; latest-price is resolution-independent (a snapshot stores
// `eventTime = 0.0`, a live tick the real time).

@genType
type latestDepositPrice = {
  price: string,
  eventTime: float,
}

@genType.opaque
type t

@genType
let make: unit => t

// Replace the entire candle series for this (asset, resolution).
@genType
let applySnapshot: (
  t,
  ~depositAsset: Shared.pubkeyStr,
  ~resolution: Shared.Resolution.t,
  ~candles: array<PriceHistory.depositPriceCandle>,
) => unit

// Append the candle, or overwrite the trailing candle's close fields when its open time matches.
@genType
let applyCandle: (
  t,
  ~depositAsset: Shared.pubkeyStr,
  ~resolution: Shared.Resolution.t,
  ~candle: PriceHistory.depositPriceCandle,
) => unit

// A live price tick (real event time) vs a per-asset snapshot (event time 0.0).
@genType
let applyPriceTick: (t, ~depositAsset: Shared.pubkeyStr, ~price: string, ~eventTime: float) => unit
@genType
let applyAssetSnapshot: (t, ~depositAsset: Shared.pubkeyStr, ~price: string) => unit

@genType
let getCandles: (
  t,
  ~depositAsset: Shared.pubkeyStr,
  ~resolution: Shared.Resolution.t,
) => option<array<PriceHistory.depositPriceCandle>>
@genType
let getLatestPrice: (t, ~depositAsset: Shared.pubkeyStr) => option<latestDepositPrice>

@genType
let clear: t => unit
