open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors rust/src/domain/price_history/state.rs (PriceHistoryState): snapshot replaces the
// series; update tail-dedupes by timestamp (same time → overwrite trailing point in place,
// new time → append). Values come from `lineDataOfOrderbookCandle` (midpoint else close).

let candle = (~t, ~m): PriceHistory.orderbookPriceCandle => {t, m}

describe("PriceHistoryState", () => {
  test("applySnapshot replaces the whole series for a key", () => {
    let state = PriceHistoryState.make()
    PriceHistoryState.applySnapshot(
      state,
      ~orderbookId="ob",
      ~resolution=Hour1,
      ~candles=[candle(~t=1.0, ~m="100"), candle(~t=2.0, ~m="101")],
    )
    expect(
      PriceHistoryState.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.map(Array.length),
    )->toBe(Some(2))
    // A later snapshot replaces (not appends).
    PriceHistoryState.applySnapshot(state, ~orderbookId="ob", ~resolution=Hour1, ~candles=[candle(~t=3.0, ~m="102")])
    expect(
      PriceHistoryState.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.map(Array.length),
    )->toBe(Some(1))
  })

  test("applyUpdate overwrites the trailing point on a matching timestamp", () => {
    let state = PriceHistoryState.make()
    PriceHistoryState.applySnapshot(state, ~orderbookId="ob", ~resolution=Hour1, ~candles=[candle(~t=1.0, ~m="100")])
    PriceHistoryState.applyUpdate(state, ~orderbookId="ob", ~resolution=Hour1, ~candle=candle(~t=1.0, ~m="105"))
    let series = PriceHistoryState.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.getOr([])
    expect(Array.length(series))->toBe(1)
    expect((series->Array.getUnsafe(0)).value)->toBe("105")
  })

  test("applyUpdate appends on a new timestamp (candle roll-over)", () => {
    let state = PriceHistoryState.make()
    PriceHistoryState.applySnapshot(state, ~orderbookId="ob", ~resolution=Hour1, ~candles=[candle(~t=1.0, ~m="100")])
    PriceHistoryState.applyUpdate(state, ~orderbookId="ob", ~resolution=Hour1, ~candle=candle(~t=2.0, ~m="101"))
    let series = PriceHistoryState.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.getOr([])
    expect(Array.length(series))->toBe(2)
    expect((series->Array.getUnsafe(1)).time)->toBe(2.0)
  })

  test("applyUpdate on a missing series creates it", () => {
    let state = PriceHistoryState.make()
    PriceHistoryState.applyUpdate(state, ~orderbookId="ob", ~resolution=Minute5, ~candle=candle(~t=1.0, ~m="100"))
    expect(
      PriceHistoryState.get(state, ~orderbookId="ob", ~resolution=Minute5)->Option.map(Array.length),
    )->toBe(Some(1))
  })

  test("keys are (orderbook, resolution) — distinct resolutions don't collide", () => {
    let state = PriceHistoryState.make()
    PriceHistoryState.applySnapshot(state, ~orderbookId="ob", ~resolution=Hour1, ~candles=[candle(~t=1.0, ~m="1")])
    PriceHistoryState.applySnapshot(state, ~orderbookId="ob", ~resolution=Minute1, ~candles=[candle(~t=1.0, ~m="2"), candle(~t=2.0, ~m="3")])
    expect(PriceHistoryState.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.map(Array.length))->toBe(Some(1))
    expect(PriceHistoryState.get(state, ~orderbookId="ob", ~resolution=Minute1)->Option.map(Array.length))->toBe(Some(2))
    expect(PriceHistoryState.get(state, ~orderbookId="other", ~resolution=Hour1))->toBe(None)
  })
})
