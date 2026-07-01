open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors the semantics asserted in rust/src/domain/orderbook/state.rs: snapshot-replace /
// last-write-wins, zero-size levels omitted, resync bails without mutating, best bid = max
// price / best ask = min price. Feeds synthetic decoded frames (no network).

let level = (side, price, size): Orderbook.wsBookLevel => {side, price, size}

let bookOf = (~bids, ~asks, ~seq=1.0, ~resync=false, ()): Orderbook.orderBook => {
  id: "ob_1",
  isSnapshot: true,
  seq,
  resync,
  bids,
  asks,
}

describe("OrderbookState.apply", () => {
  test("builds a book; best bid = max price, best ask = min price", () => {
    let state = OrderbookState.make("ob_1")
    let result = OrderbookState.apply(
      state,
      bookOf(
        ~bids=[level(Bid, "100.5", "2"), level(Bid, "100", "1")],
        ~asks=[level(Ask, "101", "3"), level(Ask, "101.5", "1")],
        (),
      ),
    )
    let applied = switch result {
    | Applied => true
    | RefreshRequired(_) => false
    }
    expect(applied)->toBe(true)
    expect(OrderbookState.bestBid(state))->toBe(Some("100.5"))
    expect(OrderbookState.bestAsk(state))->toBe(Some("101"))
    // bids high→low, asks low→high.
    expect(OrderbookState.bids(state)->Array.map(((price, _)) => price))->toEqual(["100.5", "100"])
    expect(OrderbookState.asks(state)->Array.map(((price, _)) => price))->toEqual(["101", "101.5"])
  })

  test("midPrice = (bid+ask)/2 and spread = ask−bid", () => {
    let state = OrderbookState.make("ob_1")
    OrderbookState.apply(
      state,
      bookOf(~bids=[level(Bid, "100.5", "1")], ~asks=[level(Ask, "101", "1")], ()),
    )->ignore
    expect(OrderbookState.midPrice(state))->toBe(Some("100.75"))
    expect(OrderbookState.spread(state))->toBe(Some("0.5"))
  })

  test("zero-size levels are omitted", () => {
    let state = OrderbookState.make("ob_1")
    OrderbookState.apply(
      state,
      bookOf(
        ~bids=[level(Bid, "100", "1"), level(Bid, "99", "0")],
        ~asks=[level(Ask, "101", "1")],
        (),
      ),
    )->ignore
    expect(Array.length(OrderbookState.bids(state)))->toBe(1)
    expect(OrderbookState.bestBid(state))->toBe(Some("100"))
  })

  test("duplicate price within a frame: last size wins", () => {
    let state = OrderbookState.make("ob_1")
    OrderbookState.apply(
      state,
      bookOf(
        ~bids=[level(Bid, "100", "1"), level(Bid, "100", "5")],
        ~asks=[level(Ask, "101", "1")],
        (),
      ),
    )->ignore
    expect(Array.length(OrderbookState.bids(state)))->toBe(1)
    let (_price, size) = OrderbookState.bids(state)->Array.getUnsafe(0)
    expect(size)->toBe("5")
  })

  test("a second snapshot replaces the book wholesale and updates seq", () => {
    let state = OrderbookState.make("ob_1")
    OrderbookState.apply(
      state,
      bookOf(~bids=[level(Bid, "100", "1")], ~asks=[level(Ask, "101", "1")], ()),
    )->ignore
    OrderbookState.apply(
      state,
      bookOf(~bids=[level(Bid, "200", "1")], ~asks=[level(Ask, "201", "1")], ~seq=7.0, ()),
    )->ignore
    expect(OrderbookState.bestBid(state))->toBe(Some("200"))
    expect(OrderbookState.bestAsk(state))->toBe(Some("201"))
    expect(OrderbookState.seq(state))->toBe(7.0)
  })

  test("resync frame returns RefreshRequired and leaves the book untouched", () => {
    let state = OrderbookState.make("ob_1")
    OrderbookState.apply(
      state,
      bookOf(~bids=[level(Bid, "100", "1")], ~asks=[level(Ask, "101", "1")], ()),
    )->ignore
    let result = OrderbookState.apply(state, bookOf(~bids=[], ~asks=[], ~resync=true, ()))
    let isRefresh = switch result {
    | RefreshRequired(ServerResync) => true
    | Applied => false
    }
    expect(isRefresh)->toBe(true)
    expect(OrderbookState.bestBid(state))->toBe(Some("100")) // unchanged
  })

  test("empty book: best bid / mid / spread are None", () => {
    let state = OrderbookState.make("ob_1")
    expect(OrderbookState.isEmpty(state))->toBe(true)
    expect(OrderbookState.bestBid(state))->toBe(None)
    expect(OrderbookState.midPrice(state))->toBe(None)
    expect(OrderbookState.spread(state))->toBe(None)
  })
})
