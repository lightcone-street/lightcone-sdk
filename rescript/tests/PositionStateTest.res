open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors the semantics of rust/src/domain/position/mod.rs: the balance-index
// `From` conversions (zero outcomes dropped, empty markets → None, market-level
// merge on extend), the ConditionalBalanceDelta math, and the computed display
// values (which format through Fmt.display).

let outcome = (~token, ~idle, ~onBook): Order.userOutcomeBalance => {
  outcomeIndex: 0.0,
  conditionalToken: token,
  balance: "0",
  balanceIdle: idle,
  balanceOnBook: onBook,
}

let marketBalance = (~market, ~depositAsset, ~outcomes): Order.userMarketBalance => {
  marketPubkey: market,
  depositAssets: [{depositAsset, outcomes}],
}

let delta = (~orderbookId: option<string>=?, ~idle="1.5", ~onBook="2.5"): Position.ConditionalBalanceDelta.t => {
  marketPubkey: "mkt1",
  orderbookId: ?orderbookId,
  outcomeIndex: 1.0,
  conditionalToken: "ct1",
  idle,
  onBook,
}

describe("Position.ConditionalBalanceDelta", () => {
  test("total = idle + on-book (Decimal string)", () =>
    expect(Position.ConditionalBalanceDelta.total(delta()))->toBe("4")
  )

  test("isZero: only positive balances count", () => {
    expect(Position.ConditionalBalanceDelta.isZero(delta()))->toBe(false)
    expect(Position.ConditionalBalanceDelta.isZero(delta(~idle="0", ~onBook="0")))->toBe(true)
    expect(Position.ConditionalBalanceDelta.isZero(delta(~idle="0", ~onBook="0.0001")))->toBe(false)
  })

  test("tokenBalanceOfConditionalBalanceDelta classifies as conditional token", () => {
    let balance = Position.tokenBalanceOfConditionalBalanceDelta(delta(~orderbookId="ob1"))
    expect(balance.mint)->toBe("ct1")
    expect(balance.idle)->toBe("1.5")
    switch balance.tokenType {
    | ConditionalToken({orderbookId, marketPubkey, outcomeIndex}) => {
        expect(orderbookId)->toBe("ob1")
        expect(marketPubkey)->toBe("mkt1")
        expect(outcomeIndex)->toBe(1.0)
      }
    | DepositAsset => expect("deposit asset")->toBe("conditional token")
    }
    // A missing orderbook id becomes the empty default.
    switch Position.tokenBalanceOfConditionalBalanceDelta(delta()).tokenType {
    | ConditionalToken({orderbookId}) => expect(orderbookId)->toBe("")
    | DepositAsset => expect("deposit asset")->toBe("conditional token")
    }
  })

  test("userOutcomeBalanceOfConditionalBalanceDelta sets balance = total", () => {
    let balance = Position.userOutcomeBalanceOfConditionalBalanceDelta(delta())
    expect(balance.balance)->toBe("4")
    expect(balance.balanceIdle)->toBe("1.5")
    expect(balance.balanceOnBook)->toBe("2.5")
    expect(balance.conditionalToken)->toBe("ct1")
  })
})

describe("Position computed display values", () => {
  test("computedBase formats size / value / price through Fmt.display", () => {
    let balance: Position.tokenBalance = {
      mint: "ct1",
      idle: "2",
      onBook: "1",
      tokenType: DepositAsset,
    }
    let computed = Position.computedBase(balance, ~conditionalPrice="0.5")
    expect(computed.size)->toBe("3.0000")
    expect(computed.value)->toBe("1.5000")
    expect(computed.price)->toBe("0.5000")
  })

  test("computedQuote formats idle + on-book", () => {
    let balance: Position.tokenBalance = {
      mint: "usdc",
      idle: "1000",
      onBook: "500",
      tokenType: DepositAsset,
    }
    expect(Position.computedQuote(balance))->toBe("1,500.0")
  })
})

describe("Position.UserMarketBalanceIndex", () => {
  test("ofMarketBalance drops zero outcomes and empty deposit assets", () => {
    let balance = marketBalance(
      ~market="mkt1",
      ~depositAsset="usdc",
      ~outcomes=[
        outcome(~token="ct1", ~idle="1", ~onBook="0"),
        outcome(~token="ct2", ~idle="0", ~onBook="0"),
      ],
    )
    switch Position.UserMarketBalanceIndex.ofMarketBalance(balance) {
    | Some(index) =>
      switch Position.UserMarketBalanceIndex.get(index, ~marketPubkey="mkt1") {
      | Some(byAsset) =>
        switch byAsset->Dict.get("usdc") {
        | Some(byToken) => {
            expect(Array.length(Dict.keysToArray(byToken)))->toBe(1)
            expect(byToken->Dict.get("ct1")->Option.isSome)->toBe(true)
            expect(byToken->Dict.get("ct2")->Option.isNone)->toBe(true)
          }
        | None => expect("usdc")->toBe("present")
        }
      | None => expect("mkt1")->toBe("present")
      }
    | None => expect("index")->toBe("present")
    }
  })

  test("ofMarketBalance is None when every outcome is zero", () => {
    let balance = marketBalance(
      ~market="mkt1",
      ~depositAsset="usdc",
      ~outcomes=[outcome(~token="ct1", ~idle="0", ~onBook="0")],
    )
    expect(Position.UserMarketBalanceIndex.ofMarketBalance(balance)->Option.isNone)->toBe(true)
  })

  test("ofMarketBalances indexes every market; marketPubkeys is sorted", () => {
    let index = Position.UserMarketBalanceIndex.ofMarketBalances([
      marketBalance(~market="zzz", ~depositAsset="usdc", ~outcomes=[outcome(~token="ct1", ~idle="1", ~onBook="0")]),
      marketBalance(~market="aaa", ~depositAsset="usdc", ~outcomes=[outcome(~token="ct2", ~idle="2", ~onBook="0")]),
      marketBalance(~market="mmm", ~depositAsset="usdc", ~outcomes=[outcome(~token="ct3", ~idle="0", ~onBook="0")]),
    ])
    // The all-zero market is dropped entirely.
    expect(Position.UserMarketBalanceIndex.marketPubkeys(index))->toEqual(["aaa", "zzz"])
  })

  test("extend merges at the market level; per-deposit-asset entries win wholesale", () => {
    let index = Position.UserMarketBalanceIndex.ofMarketBalances([
      marketBalance(~market="mkt1", ~depositAsset="usdc", ~outcomes=[outcome(~token="ct1", ~idle="1", ~onBook="0")]),
    ])
    let update = Position.UserMarketBalanceIndex.ofMarketBalances([
      marketBalance(~market="mkt1", ~depositAsset="usdc", ~outcomes=[outcome(~token="ct2", ~idle="2", ~onBook="0")]),
      marketBalance(~market="mkt1", ~depositAsset="wsol", ~outcomes=[outcome(~token="ct3", ~idle="3", ~onBook="0")]),
    ])
    Position.UserMarketBalanceIndex.extend(index, update)
    switch Position.UserMarketBalanceIndex.get(index, ~marketPubkey="mkt1") {
    | Some(byAsset) => {
        // usdc replaced wholesale: ct1 is gone, ct2 is in.
        switch byAsset->Dict.get("usdc") {
        | Some(byToken) => {
            expect(byToken->Dict.get("ct1")->Option.isNone)->toBe(true)
            expect(byToken->Dict.get("ct2")->Option.isSome)->toBe(true)
          }
        | None => expect("usdc")->toBe("present")
        }
        // wsol merged in alongside.
        expect(byAsset->Dict.get("wsol")->Option.isSome)->toBe(true)
      }
    | None => expect("mkt1")->toBe("present")
    }
  })

  test("insert and remove operate per market", () => {
    let index = Position.UserMarketBalanceIndex.make()
    let entry: Position.UserMarketBalanceIndex.depositAssetBalanceIndex = Dict.make()
    Position.UserMarketBalanceIndex.insert(index, ~marketPubkey="mkt1", entry)
    expect(Position.UserMarketBalanceIndex.get(index, ~marketPubkey="mkt1")->Option.isSome)->toBe(true)
    Position.UserMarketBalanceIndex.remove(index, ~marketPubkey="mkt1")
    expect(Position.UserMarketBalanceIndex.get(index, ~marketPubkey="mkt1")->Option.isNone)->toBe(true)
  })
})
