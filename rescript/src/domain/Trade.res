// Trades domain — trade-history queries (mirrors the Rust `domain/trade`).
//
// Reference shape for every read domain:
//   1. wire types (`@spice`)  — exact JSON the backend sends
//   2. domain types (`@genType`) — the clean shape exported to TypeScript
//   3. `…OfResponse` conversions (mirror the Rust `From<Wire> for Domain`)
//   4. client functions taking a `Client.t`, returning `promise<result<_, sdkError>>`
//
// Prices/sizes stay as wire strings (no precision loss, gentype-clean — wrap in
// `Decimal` for math). Timestamps/ids are floats (JS numbers, ms since epoch).

// ── Wire types ────────────────────────────────────────────────────────────────
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

@spice
type marketTradesResponse = {
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  trades: array<tradeResponse>,
  @spice.key("next_cursor") nextCursor?: float,
  @spice.default(false) @spice.key("has_more") hasMore: bool,
}

// ── Domain types ──────────────────────────────────────────────────────────────
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

type tradesPage = {
  trades: array<trade>,
  nextCursor?: float,
  hasMore: bool,
}

// ── Conversions ───────────────────────────────────────────────────────────────
let tradeOfResponse = (response: tradeResponse): trade => {
  orderbookId: response.orderbookId,
  tradeId: response.tradeId,
  cursorId: response.id,
  timestamp: response.executedAt,
  price: response.price,
  size: response.size,
  side: response.side,
  sequence: 0.0,
}

let pageOfTrades = (trades: array<tradeResponse>, nextCursor: option<float>, hasMore: bool): tradesPage => {
  trades: trades->Array.map(tradeOfResponse),
  nextCursor: ?nextCursor,
  hasMore,
}

// ── Client functions ──────────────────────────────────────────────────────────
let optionalQuery = (query, key, value) =>
  value->Option.forEach(value => query->Array.push((key, value)))

// Trades for one orderbook. `cursor` is a numeric REST row id (pass a prior
// `nextCursor` to page).
let get = async (
  client: Client.t,
  ~orderbookId: string,
  ~limit: option<int>=?,
  ~cursor: option<float>=?,
): result<tradesPage, SdkError.t> => {
  let query = [("orderbook_id", orderbookId)]
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  optionalQuery(query, "cursor", cursor->Option.map(value => Float.toString(value)))
  (await Http.get(client.http, ~path="/api/trades", ~query, ~decode=tradesResponse_decode))->Result.map(
    response => pageOfTrades(response.trades, response.nextCursor, response.hasMore),
  )
}

// Trades across every orderbook in a market, interleaved by time.
let getByMarket = async (
  client: Client.t,
  ~marketPubkey: string,
  ~limit: option<int>=?,
  ~cursor: option<float>=?,
): result<tradesPage, SdkError.t> => {
  let query = [("market_pubkey", marketPubkey)]
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  optionalQuery(query, "cursor", cursor->Option.map(value => Float.toString(value)))
  (await Http.get(client.http, ~path="/api/trades/market", ~query, ~decode=marketTradesResponse_decode))->Result.map(
    response => pageOfTrades(response.trades, response.nextCursor, response.hasMore),
  )
}
