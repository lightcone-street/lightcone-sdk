// OrderbookState — the live sorted book from WS `BookUpdate` frames (mirrors
// rust/src/domain/orderbook/state.rs). Snapshot-only / last-write-wins; a `resync` frame
// bails with `RefreshRequired` and leaves the book untouched (caller must resubscribe).
// Prices/sizes are decimal strings; bids are returned high→low, asks low→high.

@genType
type refreshReason = ServerResync

@genType
type applyResult = Applied | RefreshRequired(refreshReason)

@genType.opaque
type t

// A fresh, empty book for an orderbook id.
@genType
let make: Shared.orderBookId => t

// Apply a WS book frame: rebuilds both sides wholesale (zero-size levels dropped, last
// duplicate price wins). A `resync` frame returns `RefreshRequired` without mutating.
@genType
let apply: (t, Orderbook.orderBook) => applyResult

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
// The seq of the last applied frame (informational; never gates).
@genType
let seq: t => float
@genType
let orderbookId: t => Shared.orderBookId
@genType
let clear: t => unit
