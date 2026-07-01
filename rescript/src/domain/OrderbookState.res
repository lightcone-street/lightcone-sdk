// OrderbookState — the live sorted book maintained from WS `BookUpdate` frames. Mirrors
// rust/src/domain/orderbook/state.rs. The stream is snapshot-only / last-write-wins: every
// non-`resync` frame rebuilds both sides wholesale (a zero-size level is omitted, so
// "remove a level" is realized as "don't insert it"); `seq` / `isSnapshot` never gate — only
// `resync` bails out with `RefreshRequired`, and the caller must unsubscribe/resubscribe.
//
// Bids and asks are both price-keyed `SortedBtree`s ordered ascending (via `Decimal.cmp`);
// the bid/ask asymmetry is realized at read time — best bid = max key, best ask = min key.
// Wire prices/sizes are decimal strings; they are parsed to `Decimal` for correct numeric
// ordering + mid/spread math, and the accessors return decimal strings (the SDK convention).

type refreshReason = ServerResync
type applyResult = Applied | RefreshRequired(refreshReason)

type t = {
  orderbookId: Shared.orderBookId,
  mutable seq: float,
  bids: SortedBtree.t<Decimal.t, Decimal.t>,
  asks: SortedBtree.t<Decimal.t, Decimal.t>,
}

let make = (orderbookId: Shared.orderBookId): t => {
  orderbookId,
  seq: 0.0,
  bids: SortedBtree.make(~compare=Decimal.cmp, ()),
  asks: SortedBtree.make(~compare=Decimal.cmp, ()),
}

// Parse a wire decimal string; drop (rather than throw on) anything malformed.
let parseDecimal = (value: string): option<Decimal.t> =>
  switch Decimal.fromString(value) {
  | decimal => Some(decimal)
  | exception JsExn(_) => None
  }

// Rebuild one side from a frame's levels: clear, then insert every non-zero-size level
// (duplicate prices → last wins, matching `BTreeMap::insert`).
let rebuild = (book: SortedBtree.t<Decimal.t, Decimal.t>, levels: array<Orderbook.wsBookLevel>): unit => {
  SortedBtree.clear(book)
  levels->Array.forEach(level =>
    switch (parseDecimal(level.price), parseDecimal(level.size)) {
    | (Some(price), Some(size)) if !Decimal.isZero(size) => SortedBtree.set(book, price, size)->ignore
    | _ => ()
    }
  )
}

let apply = (state: t, book: Orderbook.orderBook): applyResult =>
  if book.resync {
    RefreshRequired(ServerResync)
  } else {
    rebuild(state.bids, book.bids)
    rebuild(state.asks, book.asks)
    state.seq = book.seq
    Applied
  }

let bestBid = (state: t): option<string> =>
  SortedBtree.maxKey(state.bids)->Option.map(Decimal.toString)

let bestAsk = (state: t): option<string> =>
  SortedBtree.minKey(state.asks)->Option.map(Decimal.toString)

let midPrice = (state: t): option<string> =>
  switch (SortedBtree.maxKey(state.bids), SortedBtree.minKey(state.asks)) {
  | (Some(bid), Some(ask)) => Some(Decimal.div(Decimal.plus(bid, ask), Decimal.fromInt(2))->Decimal.toString)
  | _ => None
  }

let spread = (state: t): option<string> =>
  switch (SortedBtree.maxKey(state.bids), SortedBtree.minKey(state.asks)) {
  | (Some(bid), Some(ask)) => Some(Decimal.minus(ask, bid)->Decimal.toString)
  | _ => None
  }

let levelToStrings = ((price, size): (Decimal.t, Decimal.t)): (string, string) => (
  Decimal.toString(price),
  Decimal.toString(size),
)

// Bids high→low (display order); asks low→high.
let bids = (state: t): array<(string, string)> =>
  SortedBtree.toArray(state.bids)->Array.toReversed->Array.map(levelToStrings)

let asks = (state: t): array<(string, string)> =>
  SortedBtree.toArray(state.asks)->Array.map(levelToStrings)

let isEmpty = (state: t): bool =>
  SortedBtree.size(state.bids) == 0 && SortedBtree.size(state.asks) == 0

let seq = (state: t): float => state.seq
let orderbookId = (state: t): Shared.orderBookId => state.orderbookId

let clear = (state: t): unit => {
  SortedBtree.clear(state.bids)
  SortedBtree.clear(state.asks)
  state.seq = 0.0
}
