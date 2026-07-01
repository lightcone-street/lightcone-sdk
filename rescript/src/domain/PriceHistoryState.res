// PriceHistoryState — rolling per-(orderbook, resolution) candle series maintained from WS
// `PriceHistory` events. Mirrors rust/src/domain/price_history/state.rs (`PriceHistoryState`).
// A snapshot replaces the whole series for its key; an update tail-dedupes by candle
// timestamp (same `time` → roll up the last point in place; new `time` → append). Heartbeat
// events carry no data and are a no-op (the consumer simply doesn't call apply for them).
//
// Values stay as decimal strings (no Decimal math). Points use the domain `PriceHistory.lineData`
// (`{time, value}`); the wire candle → line-data projection reuses `lineDataOfOrderbookCandle`
// (midpoint, else close, else empty), exactly as the REST path does.

type t = {mutable data: Dict.t<array<PriceHistory.lineData>>}

let make = (): t => {data: Dict.make()}

let keyOf = (orderbookId: Shared.orderBookId, resolution: Shared.Resolution.t): string =>
  `${orderbookId}:${Shared.Resolution.toString(resolution)}`

// Replace the entire series for this (orderbook, resolution).
let applySnapshot = (
  state: t,
  ~orderbookId: Shared.orderBookId,
  ~resolution: Shared.Resolution.t,
  ~candles: array<PriceHistory.orderbookPriceCandle>,
): unit =>
  state.data->Dict.set(
    keyOf(orderbookId, resolution),
    candles->Array.map(PriceHistory.lineDataOfOrderbookCandle),
  )

// Append or (same timestamp) roll up the trailing point.
let applyUpdate = (
  state: t,
  ~orderbookId: Shared.orderBookId,
  ~resolution: Shared.Resolution.t,
  ~candle: PriceHistory.orderbookPriceCandle,
): unit => {
  let key = keyOf(orderbookId, resolution)
  let point = PriceHistory.lineDataOfOrderbookCandle(candle)
  let existing = state.data->Dict.get(key)->Option.getOr([])
  let updated = switch existing->Array.last {
  | Some(last) if last.time == point.time =>
    Array.concat(existing->Array.slice(~start=0, ~end=Array.length(existing) - 1), [point])
  | _ => existing->Array.concat([point])
  }
  state.data->Dict.set(key, updated)
}

let get = (
  state: t,
  ~orderbookId: Shared.orderBookId,
  ~resolution: Shared.Resolution.t,
): option<array<PriceHistory.lineData>> => state.data->Dict.get(keyOf(orderbookId, resolution))

let clear = (state: t): unit => state.data = Dict.make()
