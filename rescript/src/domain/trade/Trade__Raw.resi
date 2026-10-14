// Trade wire types + wire→domain conversions. `@spice` is kept so the codecs
// are public (the REST client decodes across the file boundary); `@genType`
// exports the TS shapes.

// A single wire trade row.
module TradeResponse: {
  @genType @spice
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
  let toTrade: t => Trade__Model.t
}

// Response for `GET /api/trades?orderbook_id=…`.
module TradesResponse: {
  @genType @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    trades: array<TradeResponse.t>,
    @spice.key("next_cursor") nextCursor?: float,
    @spice.default(false) @spice.key("has_more") hasMore: bool,
  }

  let toPage: t => Trade__Model.Page.t
}

// Response for `GET /api/trades/market?market_pubkey=…`.
module MarketTradesResponse: {
  @genType @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    trades: array<TradeResponse.t>,
    @spice.key("next_cursor") nextCursor?: float,
    @spice.default(false) @spice.key("has_more") hasMore: bool,
  }

  let toPage: t => Trade__Model.Page.t
}
