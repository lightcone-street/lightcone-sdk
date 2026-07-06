// Trade wire types — the exact JSON the backend sends for trade-history
// queries, plus the wire→domain conversions. Prices/sizes stay as wire strings;
// timestamps/ids are floats (JS numbers, ms since epoch).

// A single wire trade row.
module TradeResponse = {
  @spice
  type t = {
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

  // Wire → domain trade. REST trades carry no WS sequence, so `sequence` is 0.
  let toTrade = (response: t): Trade__Model.t => {
    orderbookId: response.orderbookId,
    tradeId: response.tradeId,
    cursorId: response.id,
    timestamp: response.executedAt,
    price: response.price,
    size: response.size,
    side: response.side,
    sequence: 0.0,
  }
}

// Response for `GET /api/trades?orderbook_id=…`.
module TradesResponse = {
  @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    trades: array<TradeResponse.t>,
    @spice.key("next_cursor") nextCursor?: float,
    @spice.default(false) @spice.key("has_more") hasMore: bool,
  }

  let toPage = (response: t): Trade__Model.Page.t => {
    trades: response.trades->Array.map(TradeResponse.toTrade),
    nextCursor: ?response.nextCursor,
    hasMore: response.hasMore,
  }
}

// Response for `GET /api/trades/market?market_pubkey=…`.
module MarketTradesResponse = {
  @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    trades: array<TradeResponse.t>,
    @spice.key("next_cursor") nextCursor?: float,
    @spice.default(false) @spice.key("has_more") hasMore: bool,
  }

  let toPage = (response: t): Trade__Model.Page.t => {
    trades: response.trades->Array.map(TradeResponse.toTrade),
    nextCursor: ?response.nextCursor,
    hasMore: response.hasMore,
  }
}
