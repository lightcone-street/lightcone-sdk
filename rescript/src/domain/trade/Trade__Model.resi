// Trade domain types — a single executed trade and one page of trade history.
// Prices/sizes stay as wire strings; timestamps/ids are floats (JS numbers,
// ms since epoch).

// A single executed trade.
@genType
type t = {
  orderbookId: Shared.orderBookId,
  tradeId: string,
  // Numeric REST row id used for cursor pagination; absent on WS trades.
  cursorId?: float,
  // Unix milliseconds.
  timestamp: float,
  price: string,
  size: string,
  side: Shared.Side.t,
  // Monotonic per-orderbook sequence; 0 for REST trades.
  sequence: float,
}

// One page of trade history with cursor pagination.
module Page: {
  @genType
  type nonrec t = {
    trades: array<t>,
    nextCursor?: float,
    hasMore: bool,
  }
}
