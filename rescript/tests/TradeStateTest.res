open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors the semantics asserted in rust/src/domain/trade/state.rs: newest-first
// ordering by sequence, REST (sequence 0) prepends, capacity eviction from the
// tail, too-old trades dropped when full, replace truncates.

let trade = (~id, ~sequence): Trade.t => {
  orderbookId: "ob1",
  tradeId: id,
  timestamp: 1700000000000.0,
  price: "0.5",
  size: "1",
  side: Shared.Side.Bid,
  sequence,
}

let ids = (state: Trade.State.t): array<string> =>
  Trade.State.trades(state)->Array.map(entry => entry.tradeId)

describe("Trade.State", () => {
  test("WS trades insert newest-first by sequence, even out of order", () => {
    let state = Trade.State.make(~orderbookId="ob1", ~maxSize=5)
    Trade.State.push(state, trade(~id="a", ~sequence=1.0))
    Trade.State.push(state, trade(~id="c", ~sequence=3.0))
    Trade.State.push(state, trade(~id="b", ~sequence=2.0))
    expect(ids(state))->toEqual(["c", "b", "a"])
    expect(Trade.State.latest(state)->Option.mapOr("", entry => entry.tradeId))->toBe("c")
  })

  test("REST trades (sequence 0) prepend and evict the oldest", () => {
    let state = Trade.State.make(~orderbookId="ob1", ~maxSize=2)
    Trade.State.push(state, trade(~id="a", ~sequence=0.0))
    Trade.State.push(state, trade(~id="b", ~sequence=0.0))
    Trade.State.push(state, trade(~id="c", ~sequence=0.0))
    expect(ids(state))->toEqual(["c", "b"])
  })

  test("a full buffer drops trades older than everything retained", () => {
    let state = Trade.State.make(~orderbookId="ob1", ~maxSize=2)
    Trade.State.push(state, trade(~id="b", ~sequence=2.0))
    Trade.State.push(state, trade(~id="c", ~sequence=3.0))
    Trade.State.push(state, trade(~id="a", ~sequence=1.0)) // older than the window
    expect(ids(state))->toEqual(["c", "b"])
    // A newer trade still lands and evicts the tail.
    Trade.State.push(state, trade(~id="d", ~sequence=4.0))
    expect(ids(state))->toEqual(["d", "c"])
  })

  test("replace truncates to capacity; clear empties", () => {
    let state = Trade.State.make(~orderbookId="ob1", ~maxSize=2)
    Trade.State.replace(state, [
      trade(~id="a", ~sequence=3.0),
      trade(~id="b", ~sequence=2.0),
      trade(~id="c", ~sequence=1.0),
    ])
    expect(Trade.State.size(state))->toBe(2)
    expect(ids(state))->toEqual(["a", "b"])
    Trade.State.clear(state)
    expect(Trade.State.isEmpty(state))->toBe(true)
  })

  test("a zero-capacity history is disabled", () => {
    let state = Trade.State.make(~orderbookId="ob1", ~maxSize=0)
    Trade.State.push(state, trade(~id="a", ~sequence=1.0))
    expect(Trade.State.isEmpty(state))->toBe(true)
  })
})

describe("Market token/pair helpers", () => {
  test("stablecoin detection + currency symbol", () => {
    expect(Market.isUsdStablecoin(Market.usdcMainnet))->toBe(true)
    expect(Market.isUsdStablecoin(Market.usdtMainnet))->toBe(true)
    expect(Market.isUsdStablecoin(Market.usdcDevnetLc))->toBe(true)
    expect(Market.isUsdStablecoin("So11111111111111111111111111111111111111112"))->toBe(false)
    expect(Market.currencySymbol(Market.usdcMainnet))->toBe("$")
    expect(Market.currencySymbol("So11111111111111111111111111111111111111112"))->toBe("")
  })

  test("sortByDisplayPriority: BTC/ETH/SOL groups first, then alphabetical", () => {
    let sorted = Market.sortByDisplayPriority(
      ["ZZZ", "SOL", "AAA", "ETH", "WBTC", "BTC"],
      ~symbolOf=symbol => symbol,
    )
    expect(sorted)->toEqual(["BTC", "WBTC", "ETH", "SOL", "AAA", "ZZZ"])
  })

  test("impact / impactPct", () => {
    let (pct, sign) = Market.Impact.pct(~depositPrice="100", ~conditionalPrice="110")
    expect(pct)->toBe(10.0)
    expect(sign)->toBe("+")
    let impact = Market.Impact.make(~depositAssetPrice="100", ~conditionalPrice="90")
    expect(impact.sign)->toBe("-")
    expect(impact.isPositive)->toBe(false)
    expect(impact.pct)->toBe(10.0)
    expect(impact.dollar)->toBe("10")
    // Zero deposit price → zero impact.
    let zero = Market.Impact.make(~depositAssetPrice="0", ~conditionalPrice="90")
    expect(zero.pct)->toBe(0.0)
    expect(zero.sign)->toBe("")
  })
})

describe("Shared Side/Denominator helpers", () => {
  test("applyImpactProtection pads in the fill direction", () => {
    expect(
      Shared.Side.Bid->Shared.Side.applyImpactProtection(
        ~worstFillPrice="100",
        ~protectionPercent="10",
      ),
    )->toBe(Some("110"))
    expect(
      Shared.Side.Ask->Shared.Side.applyImpactProtection(
        ~worstFillPrice="100",
        ~protectionPercent="10",
      ),
    )->toBe(Some("90"))
    expect(
      Shared.Side.Bid->Shared.Side.applyImpactProtection(
        ~worstFillPrice="0",
        ~protectionPercent="10",
      ),
    )->toBe(None)
  })

  test("Denominator.convertTo crosses at the price; identity needs no price", () => {
    open Shared.Denominator
    expect(Base->convertTo(~target=Quote, ~amount="2", ~basePriceInQuote="0.5"))->toBe(Some("1"))
    expect(Quote->convertTo(~target=Base, ~amount="1", ~basePriceInQuote="0.5"))->toBe(Some("2"))
    expect(Base->convertTo(~target=Base, ~amount="7", ~basePriceInQuote="0"))->toBe(Some("7"))
    expect(Base->convertTo(~target=Quote, ~amount="7", ~basePriceInQuote="0"))->toBe(None)
  })
})
