// TradeState — a rolling, capped trade-history buffer per orderbook from WS
// `Trades` frames + REST backfills (mirrors rust trade/state.rs `TradeHistory`).
// Newest-first: WS trades insert by descending `sequence`; REST trades
// (`sequence == 0`) prepend. A `maxSize` of 0 disables the buffer.

@genType.opaque
type t

@genType
let make: (~orderbookId: Shared.orderBookId, ~maxSize: int) => t

// Push a new trade, keeping the buffer sorted newest-first and capped. Trades
// older than everything in a full buffer are dropped.
@genType
let push: (t, Trade.trade) => unit

// Replace all trades (e.g. from a REST fetch), truncated to capacity.
@genType
let replace: (t, array<Trade.trade>) => unit

// Newest-first view of the buffer.
@genType
let trades: t => array<Trade.trade>

// The most recent trade, if any.
@genType
let latest: t => option<Trade.trade>

@genType
let clear: t => unit
@genType
let size: t => int
@genType
let isEmpty: t => bool
