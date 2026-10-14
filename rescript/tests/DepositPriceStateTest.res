open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors rust/src/domain/price_history/state.rs (DepositPriceState + LatestDepositPrice):
// candles tail-dedupe by OPEN time (overwrite tc+c, keep t); latest-price is per-asset
// (resolution-independent) — a snapshot stores eventTime 0.0, a live tick the real time.

let candle = (~t, ~tc, ~c): PriceHistory.Raw.DepositCandle.t => {t, tc, c}

describe("PriceHistory.DepositState — candles", () => {
  test("applyCandle overwrites close fields on a matching open time", () => {
    let state = PriceHistory.DepositState.make()
    PriceHistory.DepositState.applySnapshot(state, ~depositAsset="mint", ~resolution=Hour1, ~candles=[candle(~t=1.0, ~tc=2.0, ~c="100")])
    PriceHistory.DepositState.applyCandle(state, ~depositAsset="mint", ~resolution=Hour1, ~candle=candle(~t=1.0, ~tc=3.0, ~c="105"))
    let got = PriceHistory.DepositState.getCandles(state, ~depositAsset="mint", ~resolution=Hour1)->Option.getOr([])
    expect(Array.length(got))->toBe(1)
    expect((got->Array.getUnsafe(0)).c)->toBe("105")
    expect((got->Array.getUnsafe(0)).tc)->toBe(3.0)
    expect((got->Array.getUnsafe(0)).t)->toBe(1.0) // open time preserved
  })

  test("applyCandle appends on a new open time", () => {
    let state = PriceHistory.DepositState.make()
    PriceHistory.DepositState.applySnapshot(state, ~depositAsset="mint", ~resolution=Hour1, ~candles=[candle(~t=1.0, ~tc=2.0, ~c="100")])
    PriceHistory.DepositState.applyCandle(state, ~depositAsset="mint", ~resolution=Hour1, ~candle=candle(~t=2.0, ~tc=3.0, ~c="101"))
    expect(
      PriceHistory.DepositState.getCandles(state, ~depositAsset="mint", ~resolution=Hour1)->Option.map(Array.length),
    )->toBe(Some(2))
  })
})

describe("PriceHistory.DepositState — latest price", () => {
  test("asset snapshot stores eventTime 0.0; a later tick overwrites with the real time", () => {
    let state = PriceHistory.DepositState.make()
    PriceHistory.DepositState.applyAssetSnapshot(state, ~depositAsset="mint", ~price="50")
    let snap = PriceHistory.DepositState.getLatestPrice(state, ~depositAsset="mint")
    expect(snap->Option.map(price => price.price))->toBe(Some("50"))
    expect(snap->Option.map(price => price.eventTime))->toBe(Some(0.0))

    PriceHistory.DepositState.applyPriceTick(state, ~depositAsset="mint", ~price="55", ~eventTime=123.0)
    let tick = PriceHistory.DepositState.getLatestPrice(state, ~depositAsset="mint")
    expect(tick->Option.map(price => price.price))->toBe(Some("55"))
    expect(tick->Option.map(price => price.eventTime))->toBe(Some(123.0))
  })

  test("latest price is per-asset (resolution-independent) and absent for unknown assets", () => {
    let state = PriceHistory.DepositState.make()
    PriceHistory.DepositState.applyPriceTick(state, ~depositAsset="a", ~price="1", ~eventTime=10.0)
    expect(PriceHistory.DepositState.getLatestPrice(state, ~depositAsset="a")->Option.map(price => price.price))->toBe(Some("1"))
    expect(PriceHistory.DepositState.getLatestPrice(state, ~depositAsset="b"))->toBe(None)
  })
})
