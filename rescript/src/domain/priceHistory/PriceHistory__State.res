// Rolling per-(orderbook, resolution) candle series maintained from WS
// `PriceHistory` events. A snapshot replaces the whole series for its key; an
// update tail-dedupes by candle timestamp (same `time` → roll up the last point
// in place; new `time` → append). Heartbeat events carry no data and are a
// no-op (the consumer simply doesn't call apply for them).
//
// Values stay as decimal strings (no Decimal math). Points use the domain
// `PriceHistory__Model.LineData.t` (`{time, value}`); the wire candle →
// line-data projection reuses `PriceHistory__Raw.OrderbookCandle.toLineData`
// (midpoint, else close, else empty), exactly as the REST path does.

type t = {mutable data: Dict.t<array<PriceHistory__Model.LineData.t>>}

let make = (): t => {data: Dict.make()}

let keyOf = (orderbookId: Shared.orderBookId, resolution: Shared.Resolution.t): string =>
  `${orderbookId}:${Shared.Resolution.toString(resolution)}`

// Replace the entire series for this (orderbook, resolution).
let applySnapshot = (
  state: t,
  ~orderbookId: Shared.orderBookId,
  ~resolution: Shared.Resolution.t,
  ~candles: array<PriceHistory__Raw.OrderbookCandle.t>,
): unit =>
  state.data->Dict.set(
    keyOf(orderbookId, resolution),
    candles->Array.map(PriceHistory__Raw.OrderbookCandle.toLineData),
  )

// Append or (same timestamp) roll up the trailing point.
let applyUpdate = (
  state: t,
  ~orderbookId: Shared.orderBookId,
  ~resolution: Shared.Resolution.t,
  ~candle: PriceHistory__Raw.OrderbookCandle.t,
): unit => {
  let key = keyOf(orderbookId, resolution)
  let point = PriceHistory__Raw.OrderbookCandle.toLineData(candle)
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
): option<array<PriceHistory__Model.LineData.t>> => state.data->Dict.get(keyOf(orderbookId, resolution))

let clear = (state: t): unit => state.data = Dict.make()
