// Trades domain — trade-history queries. Public surface: the gentype-exported
// `trade` / `tradesPage` domain types and the two client functions. The raw wire
// `*Response` types, the `*OfResponse` conversions, and the spice codecs are
// internal.

// ── Domain types ──────────────────────────────────────────────────────────────
@genType
type trade = {
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

@genType
type tradesPage = {
  trades: array<trade>,
  nextCursor?: float,
  hasMore: bool,
}

// ── Client functions ──────────────────────────────────────────────────────────
// Trades for one orderbook. `cursor` is a numeric REST row id (pass a prior
// `nextCursor` to page).
let get: (
  Client.t,
  ~orderbookId: string,
  ~limit: int=?,
  ~cursor: float=?,
) => promise<result<tradesPage, SdkError.t>>

// Trades across every orderbook in a market, interleaved by time.
let getByMarket: (
  Client.t,
  ~marketPubkey: string,
  ~limit: int=?,
  ~cursor: float=?,
) => promise<result<tradesPage, SdkError.t>>

// Wire type + page converter — exposed (ReScript-only; no @genType, so absent from
// the TS surface) for the white-box conversion test in tests/TradeTest.res.
@spice
type tradeResponse = {
  id: float,
  @spice.key("trade_id") tradeId: string,
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  @spice.key("taker_pubkey") takerPubkey: string,
  @spice.key("maker_pubkey") makerPubkey: string,
  side: Shared.Side.t,
  size: string,
  price: string,
  @spice.key("taker_fee") takerFee?: string,
  @spice.key("maker_fee") makerFee?: string,
  @spice.key("executed_at") executedAt: float,
}

@spice
type tradesResponse = {
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  trades: array<tradeResponse>,
  @spice.key("next_cursor") nextCursor?: float,
  @spice.default(false) @spice.key("has_more") hasMore: bool,
}

let pageOfTrades: (array<tradeResponse>, option<float>, bool) => tradesPage
