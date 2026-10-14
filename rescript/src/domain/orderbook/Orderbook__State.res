// A state belongs to one normalized subscription. Each accepted frame replaces
// both sides; sequence ordering applies only within the current generation.
type refreshReason = ServerResync
type applyResult = Applied | DiscardedStale | SubscriptionMismatch | RefreshRequired(refreshReason)

type t = {
  orderbookId: Shared.orderBookId,
  mutable seq: bigint,
  mutable lastSeq: option<bigint>,
  aggregation: Orderbook__Model.Aggregation.t,
  mutable bidsTruncated: bool,
  mutable asksTruncated: bool,
  mutable bidLevels: array<Orderbook__Raw.WsLevel.t>,
  mutable askLevels: array<Orderbook__Raw.WsLevel.t>,
  bids: SortedBtree.t<Decimal.t, Decimal.t>,
  asks: SortedBtree.t<Decimal.t, Decimal.t>,
}

let withAggregation = (
  orderbookId: Shared.orderBookId,
  aggregation: Orderbook__Model.Aggregation.t,
): t => {
  orderbookId,
  seq: 0n,
  lastSeq: None,
  aggregation: Orderbook__Model.Aggregation.normalized(aggregation),
  bidsTruncated: false,
  asksTruncated: false,
  bidLevels: [],
  askLevels: [],
  bids: SortedBtree.make(~compare=Decimal.cmp, ()),
  asks: SortedBtree.make(~compare=Decimal.cmp, ()),
}

let make = orderbookId => withAggregation(orderbookId, Orderbook__Model.Aggregation.full)

// Parse a wire decimal string; drop (rather than throw on) anything malformed.
let parseDecimal = (value: string): option<Decimal.t> =>
  switch Decimal.fromString(value) {
  | decimal => Some(decimal)
  | exception JsExn(_) => None
  }

// Rebuild one side from a frame's levels: clear, then insert every non-zero-size level
// (duplicate prices → last wins).
let rebuild = (
  book: SortedBtree.t<Decimal.t, Decimal.t>,
  levels: array<Orderbook__Raw.WsLevel.t>,
): unit => {
  SortedBtree.clear(book)
  levels->Array.forEach(level =>
    switch (parseDecimal(level.price), parseDecimal(level.size)) {
    | (Some(price), Some(size)) if !Decimal.isZero(size) =>
      SortedBtree.set(book, price, size)->ignore
    | _ => ()
    }
  )
}

let apply = (state: t, book: Orderbook__Raw.Book.t): applyResult =>
  if (
    book.id != state.orderbookId ||
      Orderbook__Model.Aggregation.keySuffix(Orderbook__Raw.Book.toAggregation(book)) !=
        Orderbook__Model.Aggregation.keySuffix(state.aggregation)
  ) {
    SubscriptionMismatch
  } else if book.resync {
    RefreshRequired(ServerResync)
  } else if state.lastSeq->Option.mapOr(false, previous => book.seq <= previous) {
    DiscardedStale
  } else {
    rebuild(state.bids, book.bids)
    rebuild(state.asks, book.asks)
    state.seq = book.seq
    state.lastSeq = Some(book.seq)
    state.bidsTruncated = book.bidsTruncated
    state.asksTruncated = book.asksTruncated
    state.bidLevels = book.bids->Array.copy
    state.askLevels = book.asks->Array.copy
    Applied
  }

let bestBid = (state: t): option<string> =>
  SortedBtree.maxKey(state.bids)->Option.map(Decimal.toString)

let bestAsk = (state: t): option<string> =>
  SortedBtree.minKey(state.asks)->Option.map(Decimal.toString)

let midPrice = (state: t): option<string> =>
  switch (SortedBtree.maxKey(state.bids), SortedBtree.minKey(state.asks)) {
  | (Some(bid), Some(ask)) =>
    Some(Decimal.div(Decimal.plus(bid, ask), Decimal.fromInt(2))->Decimal.toString)
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

let seq = (state: t): bigint => state.seq
let orderbookId = (state: t): Shared.orderBookId => state.orderbookId

let clear = (state: t): unit => {
  SortedBtree.clear(state.bids)
  SortedBtree.clear(state.asks)
  state.seq = 0n
  state.lastSeq = None
  state.bidsTruncated = false
  state.asksTruncated = false
  state.bidLevels = []
  state.askLevels = []
}

let beginGeneration = state => state.lastSeq = None
let bidsTruncated = state => state.bidsTruncated
let asksTruncated = state => state.asksTruncated
let bidLevels = state => Array.copy(state.bidLevels)
let askLevels = state => Array.copy(state.askLevels)
