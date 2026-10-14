// One normalized subscription per state. Accepted newer frames replace both sides;
// a new generation resets sequence admission while preserving visible levels.
// Resync requests preserve levels and require a subscription refresh.

@genType
type refreshReason = ServerResync

@genType
type applyResult = Applied | DiscardedStale | SubscriptionMismatch | RefreshRequired(refreshReason)

@genType.opaque
type t

// A fresh, empty book for an orderbook id.
@genType
let make: Shared.orderBookId => t

// Apply a WS book frame: rebuilds both sides wholesale (zero-size levels dropped, last
// duplicate price wins). A `resync` frame returns `RefreshRequired` without mutating.
@genType
let apply: (t, Orderbook__Raw.Book.t) => applyResult

// Best bid = highest bid price; best ask = lowest ask price.
@genType
let bestBid: t => option<string>
@genType
let bestAsk: t => option<string>

// (bestBid + bestAsk) / 2, and bestAsk − bestBid; `None` unless both sides are non-empty.
@genType
let midPrice: t => option<string>
@genType
let spread: t => option<string>

// Levels as (price, size) strings: bids high→low, asks low→high.
@genType
let bids: t => array<(string, string)>
@genType
let asks: t => array<(string, string)>

@genType
let isEmpty: t => bool
// The sequence of the last accepted data frame.
@genType
let seq: t => bigint
@genType
let orderbookId: t => Shared.orderBookId
@genType
let clear: t => unit

@genType let withAggregation: (Shared.orderBookId, Orderbook__Model.Aggregation.t) => t
@genType let beginGeneration: t => unit
@genType let bidsTruncated: t => bool
@genType let asksTruncated: t => bool
@genType let bidLevels: t => array<Orderbook__Raw.WsLevel.t>
@genType let askLevels: t => array<Orderbook__Raw.WsLevel.t>
