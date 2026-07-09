// Market wire types + wire→domain conversions. `@spice` is kept where the REST
// client decodes across the file boundary; `@genType` exports the wire shapes
// returned directly to TS (deposit mints, search results). The timestamp /
// validation helpers and the per-piece wire→domain conversions are internal.

// ── Wire types ────────────────────────────────────────────────────────────────
module OutcomeResponse: {
  @spice
  type t = {
    index: float,
    name: string,
    @spice.key("name_long") nameLong?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
  }
}

module ConditionalTokenResponse: {
  @genType
  type t = {
    id: float,
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("token_address") tokenAddress: string,
    symbol?: string,
    uri?: string,
    outcome?: string,
    @spice.key("deposit_symbol") depositSymbol?: string,
    @spice.key("short_symbol") shortSymbol?: string,
    description?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    @spice.key("metadata_uri") metadataUri?: string,
    decimals?: float,
    @spice.key("created_at") createdAt: string,
  }
}

module DepositAssetResponse: {
  @genType
  type t = {
    @spice.key("display_name") displayName?: string,
    @spice.key("token_symbol") tokenSymbol?: string,
    symbol?: string,
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    id: float,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    vault: string,
    @spice.key("num_outcomes") numOutcomes: float,
    description?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    @spice.key("metadata_uri") metadataUri?: string,
    decimals?: float,
    @spice.key("min_order_size") minOrderSize?: string,
    @spice.key("conditional_mints") conditionalMints: array<ConditionalTokenResponse.t>,
    @spice.key("created_at") createdAt: string,
  }
}

// Deposit assets registered for one market (returned directly; no domain
// conversion).
module DepositMintsResponse: {
  @genType @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("deposit_assets") depositAssets: array<DepositAssetResponse.t>,
    total: float,
  }
}

// REST single-orderbook wire type. Consumed by the market conversion to build
// `Market__Model.OrderBookPair.t`; not returned directly by any client
// function.
module OrderbookResponse: {
  @spice
  type t = {
    id: float,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    @spice.key("base_token") baseToken: string,
    @spice.key("quote_token") quoteToken: string,
    @spice.key("outcome_index") outcomeIndex?: float,
    @spice.key("tick_size") tickSize: float,
    @spice.key("total_bids") totalBids: float,
    @spice.key("total_asks") totalAsks: float,
    @spice.key("last_trade_price") lastTradePrice?: string,
    @spice.key("last_trade_time") lastTradeTime?: string,
    active: bool,
    @spice.key("created_at") createdAt: string,
    @spice.key("updated_at") updatedAt: string,
  }
}

module SearchOrderbook: {
  @genType
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    @spice.key("outcome_name") outcomeName: string,
    @spice.key("outcome_name_long") outcomeNameLong?: string,
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("deposit_base_asset") depositBaseAsset: Shared.pubkeyStr,
    @spice.key("deposit_quote_asset") depositQuoteAsset: Shared.pubkeyStr,
    @spice.key("deposit_base_symbol") depositBaseSymbol: string,
    @spice.key("deposit_quote_symbol") depositQuoteSymbol: string,
    @spice.key("base_icon_url_low") baseIconUrlLow?: string,
    @spice.key("base_icon_url_medium") baseIconUrlMedium?: string,
    @spice.key("base_icon_url_high") baseIconUrlHigh?: string,
    @spice.key("quote_icon_url_low") quoteIconUrlLow?: string,
    @spice.key("quote_icon_url_medium") quoteIconUrlMedium?: string,
    @spice.key("quote_icon_url_high") quoteIconUrlHigh?: string,
    @spice.key("conditional_base_mint") conditionalBaseMint: Shared.pubkeyStr,
    @spice.key("conditional_quote_mint") conditionalQuoteMint: Shared.pubkeyStr,
    @spice.key("outcome_icon_url_low") outcomeIconUrlLow?: string,
    @spice.key("outcome_icon_url_medium") outcomeIconUrlMedium?: string,
    @spice.key("outcome_icon_url_high") outcomeIconUrlHigh?: string,
    @spice.key("conditional_base_symbol") conditionalBaseSymbol?: string,
    @spice.key("conditional_quote_symbol") conditionalQuoteSymbol?: string,
    @spice.key("latest_mid_price") latestMidPrice?: string,
  }
}

// Search / featured wire row (returned directly; no domain conversion).
module MarketSearchResult: {
  @genType @spice
  type t = {
    slug: string,
    @spice.key("market_name") marketName: string,
    @spice.key("market_status") marketStatus: Market__Model.Status.t,
    category?: string,
    @spice.default([]) tags: array<string>,
    @spice.key("featured_rank") featuredRank: float,
    description?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    orderbooks: array<SearchOrderbook.t>,
  }
}

// ── Market wire type + conversion ─────────────────────────────────────────────
module MarketResponse: {
  @spice
  type t = {
    @spice.key("market_name") marketName?: string,
    slug?: string,
    description?: string,
    definition?: string,
    outcomes: array<OutcomeResponse.t>,
    @spice.key("banner_image_url_low") bannerImageUrlLow?: string,
    @spice.key("banner_image_url_medium") bannerImageUrlMedium?: string,
    @spice.key("banner_image_url_high") bannerImageUrlHigh?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    category?: string,
    tags?: array<string>,
    @spice.key("featured_rank") featuredRank?: float,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("market_id") marketId: float,
    oracle: string,
    @spice.key("question_id") questionId: string,
    @spice.key("condition_id") conditionId: string,
    @spice.key("market_status") marketStatus: string,
    resolution?: Market__Model.Resolution.t,
    @spice.key("created_at") createdAt: string,
    @spice.key("activated_at") activatedAt?: string,
    @spice.key("settled_at") settledAt?: string,
    @spice.key("deposit_assets") depositAssets: array<DepositAssetResponse.t>,
    orderbooks: array<OrderbookResponse.t>,
  }

  // Wire → domain market. Every validation problem is collected; any error
  // makes the whole market invalid.
  let toMarket: t => result<Market__Model.t, string>
}

// Response for `GET /api/markets`.
module MarketsResponse: {
  @spice
  type t = {
    markets: array<MarketResponse.t>,
    @spice.key("next_cursor") nextCursor?: float,
    @spice.key("has_more") hasMore: bool,
  }
}

// Response for the single-market endpoints (by slug / by pubkey).
module SingleMarketResponse: {
  @spice
  type t = {market: MarketResponse.t}
}

// ── Global deposit-asset wire types + conversion ──────────────────────────────
module GlobalDepositAssetResponse: {
  @spice
  type t = {
    id: float,
    mint: Shared.pubkeyStr,
    @spice.key("display_name") displayName?: string,
    symbol?: string,
    description?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    decimals?: float,
    @spice.key("whitelist_index") whitelistIndex: float,
    active: bool,
  }

  let toGlobalDepositAsset: t => result<Market__Model.GlobalDepositAsset.t, string>
}

// Response for `GET /api/global-deposit-assets`.
module GlobalDepositAssetsListResponse: {
  @spice
  type t = {
    assets: array<GlobalDepositAssetResponse.t>,
    total: float,
  }
}
