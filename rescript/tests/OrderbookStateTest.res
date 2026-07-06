open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors the semantics asserted in rust/src/domain/orderbook/state.rs: snapshot-replace /
// last-write-wins, zero-size levels omitted, resync bails without mutating, best bid = max
// price / best ask = min price. Feeds synthetic decoded frames (no network).

let level = (side, price, size): Orderbook.Raw.WsLevel.t => {side, price, size}

let bookOf = (~bids, ~asks, ~seq=1.0, ~resync=false, ()): Orderbook.Raw.Book.t => {
  id: "ob_1",
  isSnapshot: true,
  seq,
  resync,
  bids,
  asks,
}

describe("Orderbook.State.apply", () => {
  test("builds a book; best bid = max price, best ask = min price", () => {
    let state = Orderbook.State.make("ob_1")
    let result = Orderbook.State.apply(
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
    expect(Orderbook.State.bestBid(state))->toBe(Some("100.5"))
    expect(Orderbook.State.bestAsk(state))->toBe(Some("101"))
    // bids high→low, asks low→high.
    expect(Orderbook.State.bids(state)->Array.map(((price, _)) => price))->toEqual(["100.5", "100"])
    expect(Orderbook.State.asks(state)->Array.map(((price, _)) => price))->toEqual(["101", "101.5"])
  })

  test("midPrice = (bid+ask)/2 and spread = ask−bid", () => {
    let state = Orderbook.State.make("ob_1")
    Orderbook.State.apply(
      state,
      bookOf(~bids=[level(Bid, "100.5", "1")], ~asks=[level(Ask, "101", "1")], ()),
    )->ignore
    expect(Orderbook.State.midPrice(state))->toBe(Some("100.75"))
    expect(Orderbook.State.spread(state))->toBe(Some("0.5"))
  })

  test("zero-size levels are omitted", () => {
    let state = Orderbook.State.make("ob_1")
    Orderbook.State.apply(
      state,
      bookOf(
        ~bids=[level(Bid, "100", "1"), level(Bid, "99", "0")],
        ~asks=[level(Ask, "101", "1")],
        (),
      ),
    )->ignore
    expect(Array.length(Orderbook.State.bids(state)))->toBe(1)
    expect(Orderbook.State.bestBid(state))->toBe(Some("100"))
  })

  test("duplicate price within a frame: last size wins", () => {
    let state = Orderbook.State.make("ob_1")
    Orderbook.State.apply(
      state,
      bookOf(
        ~bids=[level(Bid, "100", "1"), level(Bid, "100", "5")],
        ~asks=[level(Ask, "101", "1")],
        (),
      ),
    )->ignore
    expect(Array.length(Orderbook.State.bids(state)))->toBe(1)
    let (_price, size) = Orderbook.State.bids(state)->Array.getUnsafe(0)
    expect(size)->toBe("5")
  })

  test("a second snapshot replaces the book wholesale and updates seq", () => {
    let state = Orderbook.State.make("ob_1")
    Orderbook.State.apply(
      state,
      bookOf(~bids=[level(Bid, "100", "1")], ~asks=[level(Ask, "101", "1")], ()),
    )->ignore
    Orderbook.State.apply(
      state,
      bookOf(~bids=[level(Bid, "200", "1")], ~asks=[level(Ask, "201", "1")], ~seq=7.0, ()),
    )->ignore
    expect(Orderbook.State.bestBid(state))->toBe(Some("200"))
    expect(Orderbook.State.bestAsk(state))->toBe(Some("201"))
    expect(Orderbook.State.seq(state))->toBe(7.0)
  })

  test("resync frame returns RefreshRequired and leaves the book untouched", () => {
    let state = Orderbook.State.make("ob_1")
    Orderbook.State.apply(
      state,
      bookOf(~bids=[level(Bid, "100", "1")], ~asks=[level(Ask, "101", "1")], ()),
    )->ignore
    let result = Orderbook.State.apply(state, bookOf(~bids=[], ~asks=[], ~resync=true, ()))
    let isRefresh = switch result {
    | RefreshRequired(ServerResync) => true
    | Applied => false
    }
    expect(isRefresh)->toBe(true)
    expect(Orderbook.State.bestBid(state))->toBe(Some("100")) // unchanged
  })

  test("empty book: best bid / mid / spread are None", () => {
    let state = Orderbook.State.make("ob_1")
    expect(Orderbook.State.isEmpty(state))->toBe(true)
    expect(Orderbook.State.bestBid(state))->toBe(None)
    expect(Orderbook.State.midPrice(state))->toBe(None)
    expect(Orderbook.State.spread(state))->toBe(None)
  })
})
