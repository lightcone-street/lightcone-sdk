// DepositPriceState — deposit-token candle series per (asset, resolution) plus the latest
// per-asset price. Mirrors rust/src/domain/price_history/state.rs (`DepositPriceState` +
// `LatestDepositPrice`), fed by the WS `DepositPrice` and `DepositAssetPrice` channels.
//
// `candles` is keyed by (asset, resolution); a snapshot replaces the series, a candle update
// tail-dedupes by OPEN time `t` (same `t` → overwrite `tc`+`c` of the trailing candle; new
// `t` → append). `latest` is keyed by asset alone (resolution-independent — ticks broadcast
// to all resolutions): a live tick carries the real `eventTime`; a per-asset snapshot carries
// only a price, so it stores `eventTime = 0.0` (a later tick overwrites it). No Decimal math.

type latestDepositPrice = {
  price: string,
  eventTime: float,
}

type t = {
  mutable candles: Dict.t<array<PriceHistory.depositPriceCandle>>,
  mutable latest: Dict.t<latestDepositPrice>,
}

let make = (): t => {candles: Dict.make(), latest: Dict.make()}

let keyOf = (depositAsset: Shared.pubkeyStr, resolution: Shared.Resolution.t): string =>
  `${depositAsset}:${Shared.Resolution.toString(resolution)}`

// Replace the entire candle series for this (asset, resolution).
let applySnapshot = (
  state: t,
  ~depositAsset: Shared.pubkeyStr,
  ~resolution: Shared.Resolution.t,
  ~candles: array<PriceHistory.depositPriceCandle>,
): unit => state.candles->Dict.set(keyOf(depositAsset, resolution), candles)

// Append the candle, or (same open time) overwrite the trailing candle's close fields.
let applyCandle = (
  state: t,
  ~depositAsset: Shared.pubkeyStr,
  ~resolution: Shared.Resolution.t,
  ~candle: PriceHistory.depositPriceCandle,
): unit => {
  let key = keyOf(depositAsset, resolution)
  let existing = state.candles->Dict.get(key)->Option.getOr([])
  let updated = switch existing->Array.last {
  | Some(last) if last.t == candle.t =>
    Array.concat(existing->Array.slice(~start=0, ~end=Array.length(existing) - 1), [candle])
  | _ => existing->Array.concat([candle])
  }
  state.candles->Dict.set(key, updated)
}

// A live price tick — carries the real event time.
let applyPriceTick = (
  state: t,
  ~depositAsset: Shared.pubkeyStr,
  ~price: string,
  ~eventTime: float,
): unit => state.latest->Dict.set(depositAsset, {price, eventTime})

// A per-asset price snapshot — no event time on the wire, so store 0.0 (a later tick wins).
let applyAssetSnapshot = (state: t, ~depositAsset: Shared.pubkeyStr, ~price: string): unit =>
  state.latest->Dict.set(depositAsset, {price, eventTime: 0.0})

let getCandles = (
  state: t,
  ~depositAsset: Shared.pubkeyStr,
  ~resolution: Shared.Resolution.t,
): option<array<PriceHistory.depositPriceCandle>> =>
  state.candles->Dict.get(keyOf(depositAsset, resolution))

let getLatestPrice = (state: t, ~depositAsset: Shared.pubkeyStr): option<latestDepositPrice> =>
  state.latest->Dict.get(depositAsset)

let clear = (state: t): unit => {
  state.candles = Dict.make()
  state.latest = Dict.make()
}
