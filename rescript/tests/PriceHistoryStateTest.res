open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors rust/src/domain/price_history/state.rs (PriceHistoryState): snapshot replaces the
// series; update tail-dedupes by timestamp (same time → overwrite trailing point in place,
// new time → append). Values come from `OrderbookCandle.toLineData` (midpoint else close).

let candle = (~t, ~m): PriceHistory.Raw.OrderbookCandle.t => {t, m}

describe("PriceHistory.State", () => {
  test("applySnapshot replaces the whole series for a key", () => {
    let state = PriceHistory.State.make()
    PriceHistory.State.applySnapshot(
      state,
      ~orderbookId="ob",
      ~resolution=Hour1,
      ~candles=[candle(~t=1.0, ~m="100"), candle(~t=2.0, ~m="101")],
    )
    expect(
      PriceHistory.State.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.map(Array.length),
    )->toBe(Some(2))
    // A later snapshot replaces (not appends).
    PriceHistory.State.applySnapshot(state, ~orderbookId="ob", ~resolution=Hour1, ~candles=[candle(~t=3.0, ~m="102")])
    expect(
      PriceHistory.State.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.map(Array.length),
    )->toBe(Some(1))
  })

  test("applyUpdate overwrites the trailing point on a matching timestamp", () => {
    let state = PriceHistory.State.make()
    PriceHistory.State.applySnapshot(state, ~orderbookId="ob", ~resolution=Hour1, ~candles=[candle(~t=1.0, ~m="100")])
    PriceHistory.State.applyUpdate(state, ~orderbookId="ob", ~resolution=Hour1, ~candle=candle(~t=1.0, ~m="105"))
    let series = PriceHistory.State.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.getOr([])
    expect(Array.length(series))->toBe(1)
    expect((series->Array.getUnsafe(0)).value)->toBe("105")
  })

  test("applyUpdate appends on a new timestamp (candle roll-over)", () => {
    let state = PriceHistory.State.make()
    PriceHistory.State.applySnapshot(state, ~orderbookId="ob", ~resolution=Hour1, ~candles=[candle(~t=1.0, ~m="100")])
    PriceHistory.State.applyUpdate(state, ~orderbookId="ob", ~resolution=Hour1, ~candle=candle(~t=2.0, ~m="101"))
    let series = PriceHistory.State.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.getOr([])
    expect(Array.length(series))->toBe(2)
    expect((series->Array.getUnsafe(1)).time)->toBe(2.0)
  })

  test("applyUpdate on a missing series creates it", () => {
    let state = PriceHistory.State.make()
    PriceHistory.State.applyUpdate(state, ~orderbookId="ob", ~resolution=Minute5, ~candle=candle(~t=1.0, ~m="100"))
    expect(
      PriceHistory.State.get(state, ~orderbookId="ob", ~resolution=Minute5)->Option.map(Array.length),
    )->toBe(Some(1))
  })

  test("keys are (orderbook, resolution) — distinct resolutions don't collide", () => {
    let state = PriceHistory.State.make()
    PriceHistory.State.applySnapshot(state, ~orderbookId="ob", ~resolution=Hour1, ~candles=[candle(~t=1.0, ~m="1")])
    PriceHistory.State.applySnapshot(state, ~orderbookId="ob", ~resolution=Minute1, ~candles=[candle(~t=1.0, ~m="2"), candle(~t=2.0, ~m="3")])
    expect(PriceHistory.State.get(state, ~orderbookId="ob", ~resolution=Hour1)->Option.map(Array.length))->toBe(Some(1))
    expect(PriceHistory.State.get(state, ~orderbookId="ob", ~resolution=Minute1)->Option.map(Array.length))->toBe(Some(2))
    expect(PriceHistory.State.get(state, ~orderbookId="other", ~resolution=Hour1))->toBe(None)
  })
})
