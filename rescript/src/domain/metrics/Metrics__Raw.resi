// Metrics wire types — platform / market / orderbook / category / deposit-token
// volume metrics, plus deposit-token volume history, open-interest history,
// unique-trader history, the market leaderboard, time-series history, and
// per-wallet aggregates. The wire shapes ARE the values the client returns (no
// domain conversion). `@spice` is kept so the codecs are public (the client
// decodes across the file boundary); `@genType` exports the TS shapes.

// ── Enums ──────────────────────────────────────────────────────────────────────
// Scope vocabulary for `unique-traders/history`.
module UniqueTradersHistoryScope: {
  @genType @spice
  type t =
    | @as("platform") @spice.as("platform") Platform
    | @as("market") @spice.as("market") Market
    | @as("orderbook") @spice.as("orderbook") Orderbook
    | @as("category") @spice.as("category") Category
    | @as("outcome") @spice.as("outcome") Outcome

  let toString: t => string
}

// ── Deposit-token volume (shared sub-shape) ────────────────────────────────────
// Entry in `deposit-tokens`, also nested in platform / market-detail / category.
module DepositTokenVolume: {
  @genType @spice
  type t = {
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    symbol?: string,
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("volume_7d_usd") volume7dUsd: string,
    @spice.key("volume_30d_usd") volume30dUsd: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("taker_bid_volume_24h_usd") takerBidVolume24hUsd: string,
    @spice.key("taker_bid_volume_7d_usd") takerBidVolume7dUsd: string,
    @spice.key("taker_bid_volume_30d_usd") takerBidVolume30dUsd: string,
    @spice.key("taker_bid_volume_total_usd") takerBidVolumeTotalUsd: string,
    @spice.key("taker_ask_volume_24h_usd") takerAskVolume24hUsd: string,
    @spice.key("taker_ask_volume_7d_usd") takerAskVolume7dUsd: string,
    @spice.key("taker_ask_volume_30d_usd") takerAskVolume30dUsd: string,
    @spice.key("taker_ask_volume_total_usd") takerAskVolumeTotalUsd: string,
    @spice.key("taker_bid_ask_imbalance_24h_pct") takerBidAskImbalance24hPct: string,
    @spice.key("taker_bid_ask_imbalance_7d_pct") takerBidAskImbalance7dPct: string,
    @spice.key("taker_bid_ask_imbalance_30d_pct") takerBidAskImbalance30dPct: string,
    @spice.key("taker_bid_ask_imbalance_total_pct") takerBidAskImbalanceTotalPct: string,
    @spice.key("volume_share_24h_pct") volumeShare24hPct: string,
  }
}

// ── Orderbook tickers (batch BBO + midpoint over REST) ─────────────────────────
module OrderbookTickerEntry: {
  @genType @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("outcome_index") outcomeIndex?: float,
    @spice.key("outcome_name") outcomeName?: string,
    @spice.key("outcome_name_long") outcomeNameLong?: string,
    @spice.key("base_deposit_asset") baseDepositAsset: Shared.pubkeyStr,
    @spice.key("quote_deposit_asset") quoteDepositAsset: Shared.pubkeyStr,
    @spice.key("best_bid") bestBid?: string,
    @spice.key("best_ask") bestAsk?: string,
    midpoint?: string,
    // RFC3339 string.
    @spice.key("computed_at") computedAt?: string,
  }
}

module OrderbookTickersResponse: {
  @genType @spice
  type t = {tickers: array<OrderbookTickerEntry.t>}
}

// ── Platform ───────────────────────────────────────────────────────────────────
module Platform: {
  @genType @spice
  type t = {
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("volume_7d_usd") volume7dUsd: string,
    @spice.key("volume_30d_usd") volume30dUsd: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("taker_bid_volume_24h_usd") takerBidVolume24hUsd: string,
    @spice.key("taker_bid_volume_7d_usd") takerBidVolume7dUsd: string,
    @spice.key("taker_bid_volume_30d_usd") takerBidVolume30dUsd: string,
    @spice.key("taker_bid_volume_total_usd") takerBidVolumeTotalUsd: string,
    @spice.key("taker_ask_volume_24h_usd") takerAskVolume24hUsd: string,
    @spice.key("taker_ask_volume_7d_usd") takerAskVolume7dUsd: string,
    @spice.key("taker_ask_volume_30d_usd") takerAskVolume30dUsd: string,
    @spice.key("taker_ask_volume_total_usd") takerAskVolumeTotalUsd: string,
    @spice.key("taker_bid_ask_imbalance_24h_pct") takerBidAskImbalance24hPct: string,
    @spice.key("taker_bid_ask_imbalance_7d_pct") takerBidAskImbalance7dPct: string,
    @spice.key("taker_bid_ask_imbalance_30d_pct") takerBidAskImbalance30dPct: string,
    @spice.key("taker_bid_ask_imbalance_total_pct") takerBidAskImbalanceTotalPct: string,
    @spice.key("open_interest_usd") openInterestUsd: string,
    @spice.key("fees_24h_usd") fees24hUsd: string,
    @spice.key("fees_7d_usd") fees7dUsd: string,
    @spice.key("fees_30d_usd") fees30dUsd: string,
    @spice.key("unique_traders_24h") uniqueTraders24h: float,
    @spice.key("unique_traders_7d") uniqueTraders7d: float,
    @spice.key("unique_traders_30d") uniqueTraders30d: float,
    @spice.key("active_markets") activeMarkets: float,
    @spice.key("active_orderbooks") activeOrderbooks: float,
    @spice.key("deposit_token_volumes") depositTokenVolumes: array<DepositTokenVolume.t>,
    // RFC3339 string.
    @spice.key("updated_at") updatedAt?: string,
  }
}

// ── Market (listing) ───────────────────────────────────────────────────────────
module MarketVolume: {
  @genType @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    slug?: string,
    @spice.key("market_name") marketName?: string,
    category?: string,
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("volume_7d_usd") volume7dUsd: string,
    @spice.key("volume_30d_usd") volume30dUsd: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("taker_bid_volume_24h_usd") takerBidVolume24hUsd: string,
    @spice.key("taker_bid_volume_7d_usd") takerBidVolume7dUsd: string,
    @spice.key("taker_bid_volume_30d_usd") takerBidVolume30dUsd: string,
    @spice.key("taker_bid_volume_total_usd") takerBidVolumeTotalUsd: string,
    @spice.key("taker_ask_volume_24h_usd") takerAskVolume24hUsd: string,
    @spice.key("taker_ask_volume_7d_usd") takerAskVolume7dUsd: string,
    @spice.key("taker_ask_volume_30d_usd") takerAskVolume30dUsd: string,
    @spice.key("taker_ask_volume_total_usd") takerAskVolumeTotalUsd: string,
    @spice.key("taker_bid_ask_imbalance_24h_pct") takerBidAskImbalance24hPct: string,
    @spice.key("taker_bid_ask_imbalance_7d_pct") takerBidAskImbalance7dPct: string,
    @spice.key("taker_bid_ask_imbalance_30d_pct") takerBidAskImbalance30dPct: string,
    @spice.key("taker_bid_ask_imbalance_total_pct") takerBidAskImbalanceTotalPct: string,
    @spice.key("unique_traders_24h") uniqueTraders24h: float,
    @spice.key("unique_traders_7d") uniqueTraders7d: float,
    @spice.key("unique_traders_30d") uniqueTraders30d: float,
    @spice.key("category_volume_share_24h_pct") categoryVolumeShare24hPct: string,
    @spice.key("platform_volume_share_24h_pct") platformVolumeShare24hPct: string,
  }
}

module Markets: {
  @genType @spice
  type t = {
    markets: array<MarketVolume.t>,
    total: float,
  }
}

// ── Market detail (per-outcome / per-orderbook breakdowns) ─────────────────────
module OutcomeVolume: {
  @genType @spice
  type t = {
    @spice.key("outcome_index") outcomeIndex?: float,
    @spice.key("outcome_name") outcomeName?: string,
    @spice.key("outcome_name_long") outcomeNameLong?: string,
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("volume_7d_usd") volume7dUsd: string,
    @spice.key("volume_30d_usd") volume30dUsd: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("taker_bid_volume_24h_usd") takerBidVolume24hUsd: string,
    @spice.key("taker_bid_volume_7d_usd") takerBidVolume7dUsd: string,
    @spice.key("taker_bid_volume_30d_usd") takerBidVolume30dUsd: string,
    @spice.key("taker_bid_volume_total_usd") takerBidVolumeTotalUsd: string,
    @spice.key("taker_ask_volume_24h_usd") takerAskVolume24hUsd: string,
    @spice.key("taker_ask_volume_7d_usd") takerAskVolume7dUsd: string,
    @spice.key("taker_ask_volume_30d_usd") takerAskVolume30dUsd: string,
    @spice.key("taker_ask_volume_total_usd") takerAskVolumeTotalUsd: string,
    @spice.key("taker_bid_ask_imbalance_24h_pct") takerBidAskImbalance24hPct: string,
    @spice.key("taker_bid_ask_imbalance_7d_pct") takerBidAskImbalance7dPct: string,
    @spice.key("taker_bid_ask_imbalance_30d_pct") takerBidAskImbalance30dPct: string,
    @spice.key("taker_bid_ask_imbalance_total_pct") takerBidAskImbalanceTotalPct: string,
    @spice.key("unique_traders_24h") uniqueTraders24h: float,
    @spice.key("unique_traders_7d") uniqueTraders7d: float,
    @spice.key("unique_traders_30d") uniqueTraders30d: float,
    @spice.key("volume_share_24h_pct") volumeShare24hPct: string,
  }
}

module MarketOrderbookVolume: {
  @genType @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    @spice.key("outcome_index") outcomeIndex?: float,
    @spice.key("outcome_name") outcomeName?: string,
    @spice.key("outcome_name_long") outcomeNameLong?: string,
    @spice.key("base_deposit_asset") baseDepositAsset: Shared.pubkeyStr,
    @spice.key("base_deposit_symbol") baseDepositSymbol?: string,
    @spice.key("quote_deposit_asset") quoteDepositAsset: Shared.pubkeyStr,
    @spice.key("quote_deposit_symbol") quoteDepositSymbol?: string,
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("volume_7d_usd") volume7dUsd: string,
    @spice.key("volume_30d_usd") volume30dUsd: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("volume_24h_base") volume24hBase: string,
    @spice.key("volume_7d_base") volume7dBase: string,
    @spice.key("volume_30d_base") volume30dBase: string,
    @spice.key("volume_total_base") volumeTotalBase: string,
    @spice.key("volume_24h_quote") volume24hQuote: string,
    @spice.key("volume_7d_quote") volume7dQuote: string,
    @spice.key("volume_30d_quote") volume30dQuote: string,
    @spice.key("volume_total_quote") volumeTotalQuote: string,
    @spice.key("taker_bid_volume_24h_usd") takerBidVolume24hUsd: string,
    @spice.key("taker_bid_volume_7d_usd") takerBidVolume7dUsd: string,
    @spice.key("taker_bid_volume_30d_usd") takerBidVolume30dUsd: string,
    @spice.key("taker_bid_volume_total_usd") takerBidVolumeTotalUsd: string,
    @spice.key("taker_bid_volume_24h_base") takerBidVolume24hBase: string,
    @spice.key("taker_bid_volume_7d_base") takerBidVolume7dBase: string,
    @spice.key("taker_bid_volume_30d_base") takerBidVolume30dBase: string,
    @spice.key("taker_bid_volume_total_base") takerBidVolumeTotalBase: string,
    @spice.key("taker_bid_volume_24h_quote") takerBidVolume24hQuote: string,
    @spice.key("taker_bid_volume_7d_quote") takerBidVolume7dQuote: string,
    @spice.key("taker_bid_volume_30d_quote") takerBidVolume30dQuote: string,
    @spice.key("taker_bid_volume_total_quote") takerBidVolumeTotalQuote: string,
    @spice.key("taker_ask_volume_24h_usd") takerAskVolume24hUsd: string,
    @spice.key("taker_ask_volume_7d_usd") takerAskVolume7dUsd: string,
    @spice.key("taker_ask_volume_30d_usd") takerAskVolume30dUsd: string,
    @spice.key("taker_ask_volume_total_usd") takerAskVolumeTotalUsd: string,
    @spice.key("taker_ask_volume_24h_base") takerAskVolume24hBase: string,
    @spice.key("taker_ask_volume_7d_base") takerAskVolume7dBase: string,
    @spice.key("taker_ask_volume_30d_base") takerAskVolume30dBase: string,
    @spice.key("taker_ask_volume_total_base") takerAskVolumeTotalBase: string,
    @spice.key("taker_ask_volume_24h_quote") takerAskVolume24hQuote: string,
    @spice.key("taker_ask_volume_7d_quote") takerAskVolume7dQuote: string,
    @spice.key("taker_ask_volume_30d_quote") takerAskVolume30dQuote: string,
    @spice.key("taker_ask_volume_total_quote") takerAskVolumeTotalQuote: string,
    @spice.key("taker_bid_ask_imbalance_24h_pct") takerBidAskImbalance24hPct: string,
    @spice.key("taker_bid_ask_imbalance_7d_pct") takerBidAskImbalance7dPct: string,
    @spice.key("taker_bid_ask_imbalance_30d_pct") takerBidAskImbalance30dPct: string,
    @spice.key("taker_bid_ask_imbalance_total_pct") takerBidAskImbalanceTotalPct: string,
    @spice.key("volume_share_24h_pct") volumeShare24hPct: string,
  }
}

module MarketDetail: {
  @genType @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    slug?: string,
    @spice.key("market_name") marketName?: string,
    category?: string,
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("volume_7d_usd") volume7dUsd: string,
    @spice.key("volume_30d_usd") volume30dUsd: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("taker_bid_volume_24h_usd") takerBidVolume24hUsd: string,
    @spice.key("taker_bid_volume_7d_usd") takerBidVolume7dUsd: string,
    @spice.key("taker_bid_volume_30d_usd") takerBidVolume30dUsd: string,
    @spice.key("taker_bid_volume_total_usd") takerBidVolumeTotalUsd: string,
    @spice.key("taker_ask_volume_24h_usd") takerAskVolume24hUsd: string,
    @spice.key("taker_ask_volume_7d_usd") takerAskVolume7dUsd: string,
    @spice.key("taker_ask_volume_30d_usd") takerAskVolume30dUsd: string,
    @spice.key("taker_ask_volume_total_usd") takerAskVolumeTotalUsd: string,
    @spice.key("taker_bid_ask_imbalance_24h_pct") takerBidAskImbalance24hPct: string,
    @spice.key("taker_bid_ask_imbalance_7d_pct") takerBidAskImbalance7dPct: string,
    @spice.key("taker_bid_ask_imbalance_30d_pct") takerBidAskImbalance30dPct: string,
    @spice.key("taker_bid_ask_imbalance_total_pct") takerBidAskImbalanceTotalPct: string,
    @spice.key("unique_traders_24h") uniqueTraders24h: float,
    @spice.key("unique_traders_7d") uniqueTraders7d: float,
    @spice.key("unique_traders_30d") uniqueTraders30d: float,
    @spice.key("category_volume_share_24h_pct") categoryVolumeShare24hPct: string,
    @spice.key("platform_volume_share_24h_pct") platformVolumeShare24hPct: string,
    @spice.key("outcome_volumes") outcomeVolumes: array<OutcomeVolume.t>,
    @spice.key("orderbook_volumes") orderbookVolumes: array<MarketOrderbookVolume.t>,
    @spice.key("deposit_token_volumes") depositTokenVolumes: array<DepositTokenVolume.t>,
  }
}

// ── Orderbook (detail) ─────────────────────────────────────────────────────────
module OrderbookVolume: {
  @genType @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("outcome_index") outcomeIndex?: float,
    @spice.key("outcome_name") outcomeName?: string,
    @spice.key("outcome_name_long") outcomeNameLong?: string,
    @spice.key("base_deposit_asset") baseDepositAsset: Shared.pubkeyStr,
    @spice.key("base_deposit_symbol") baseDepositSymbol?: string,
    @spice.key("quote_deposit_asset") quoteDepositAsset: Shared.pubkeyStr,
    @spice.key("quote_deposit_symbol") quoteDepositSymbol?: string,
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("volume_7d_usd") volume7dUsd: string,
    @spice.key("volume_30d_usd") volume30dUsd: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("volume_24h_base") volume24hBase: string,
    @spice.key("volume_7d_base") volume7dBase: string,
    @spice.key("volume_30d_base") volume30dBase: string,
    @spice.key("volume_total_base") volumeTotalBase: string,
    @spice.key("volume_24h_quote") volume24hQuote: string,
    @spice.key("volume_7d_quote") volume7dQuote: string,
    @spice.key("volume_30d_quote") volume30dQuote: string,
    @spice.key("volume_total_quote") volumeTotalQuote: string,
    @spice.key("taker_bid_volume_24h_usd") takerBidVolume24hUsd: string,
    @spice.key("taker_bid_volume_7d_usd") takerBidVolume7dUsd: string,
    @spice.key("taker_bid_volume_30d_usd") takerBidVolume30dUsd: string,
    @spice.key("taker_bid_volume_total_usd") takerBidVolumeTotalUsd: string,
    @spice.key("taker_bid_volume_24h_base") takerBidVolume24hBase: string,
    @spice.key("taker_bid_volume_7d_base") takerBidVolume7dBase: string,
    @spice.key("taker_bid_volume_30d_base") takerBidVolume30dBase: string,
    @spice.key("taker_bid_volume_total_base") takerBidVolumeTotalBase: string,
    @spice.key("taker_bid_volume_24h_quote") takerBidVolume24hQuote: string,
    @spice.key("taker_bid_volume_7d_quote") takerBidVolume7dQuote: string,
    @spice.key("taker_bid_volume_30d_quote") takerBidVolume30dQuote: string,
    @spice.key("taker_bid_volume_total_quote") takerBidVolumeTotalQuote: string,
    @spice.key("taker_ask_volume_24h_usd") takerAskVolume24hUsd: string,
    @spice.key("taker_ask_volume_7d_usd") takerAskVolume7dUsd: string,
    @spice.key("taker_ask_volume_30d_usd") takerAskVolume30dUsd: string,
    @spice.key("taker_ask_volume_total_usd") takerAskVolumeTotalUsd: string,
    @spice.key("taker_ask_volume_24h_base") takerAskVolume24hBase: string,
    @spice.key("taker_ask_volume_7d_base") takerAskVolume7dBase: string,
    @spice.key("taker_ask_volume_30d_base") takerAskVolume30dBase: string,
    @spice.key("taker_ask_volume_total_base") takerAskVolumeTotalBase: string,
    @spice.key("taker_ask_volume_24h_quote") takerAskVolume24hQuote: string,
    @spice.key("taker_ask_volume_7d_quote") takerAskVolume7dQuote: string,
    @spice.key("taker_ask_volume_30d_quote") takerAskVolume30dQuote: string,
    @spice.key("taker_ask_volume_total_quote") takerAskVolumeTotalQuote: string,
    @spice.key("taker_bid_ask_imbalance_24h_pct") takerBidAskImbalance24hPct: string,
    @spice.key("taker_bid_ask_imbalance_7d_pct") takerBidAskImbalance7dPct: string,
    @spice.key("taker_bid_ask_imbalance_30d_pct") takerBidAskImbalance30dPct: string,
    @spice.key("taker_bid_ask_imbalance_total_pct") takerBidAskImbalanceTotalPct: string,
    @spice.key("unique_traders_24h") uniqueTraders24h: float,
    @spice.key("unique_traders_7d") uniqueTraders7d: float,
    @spice.key("unique_traders_30d") uniqueTraders30d: float,
    @spice.key("market_volume_share_24h_pct") marketVolumeShare24hPct: string,
  }
}

// ── Category ───────────────────────────────────────────────────────────────────
module CategoryVolume: {
  @genType @spice
  type t = {
    category: string,
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("volume_7d_usd") volume7dUsd: string,
    @spice.key("volume_30d_usd") volume30dUsd: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("taker_bid_volume_24h_usd") takerBidVolume24hUsd: string,
    @spice.key("taker_bid_volume_7d_usd") takerBidVolume7dUsd: string,
    @spice.key("taker_bid_volume_30d_usd") takerBidVolume30dUsd: string,
    @spice.key("taker_bid_volume_total_usd") takerBidVolumeTotalUsd: string,
    @spice.key("taker_ask_volume_24h_usd") takerAskVolume24hUsd: string,
    @spice.key("taker_ask_volume_7d_usd") takerAskVolume7dUsd: string,
    @spice.key("taker_ask_volume_30d_usd") takerAskVolume30dUsd: string,
    @spice.key("taker_ask_volume_total_usd") takerAskVolumeTotalUsd: string,
    @spice.key("taker_bid_ask_imbalance_24h_pct") takerBidAskImbalance24hPct: string,
    @spice.key("taker_bid_ask_imbalance_7d_pct") takerBidAskImbalance7dPct: string,
    @spice.key("taker_bid_ask_imbalance_30d_pct") takerBidAskImbalance30dPct: string,
    @spice.key("taker_bid_ask_imbalance_total_pct") takerBidAskImbalanceTotalPct: string,
    @spice.key("unique_traders_24h") uniqueTraders24h: float,
    @spice.key("unique_traders_7d") uniqueTraders7d: float,
    @spice.key("unique_traders_30d") uniqueTraders30d: float,
    @spice.key("platform_volume_share_24h_pct") platformVolumeShare24hPct: string,
    @spice.key("deposit_token_volumes") depositTokenVolumes: array<DepositTokenVolume.t>,
  }
}

module Categories: {
  @genType @spice
  type t = {categories: array<CategoryVolume.t>}
}

// ── Deposit tokens (platform-wide) ─────────────────────────────────────────────
module DepositTokens: {
  @genType @spice
  type t = {
    @spice.key("deposit_tokens") depositTokens: array<DepositTokenVolume.t>,
  }
}

// ── Deposit-token volume history ───────────────────────────────────────────────
module DepositTokenVolumeHistory: {
  @genType @spice
  type token = {
    rank: float,
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    symbol?: string,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
  }

  @genType @spice
  type pointToken = {
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    symbol?: string,
    @spice.key("volume_usd") volumeUsd: string,
  }

  @genType @spice
  type point = {
    // Unix milliseconds (bucket start).
    @spice.key("bucket_start") bucketStart: float,
    // Calendar day "YYYY-MM-DD".
    @spice.key("bucket_start_date") bucketStartDate: string,
    @spice.key("total_volume_usd") totalVolumeUsd: string,
    @spice.key("cumulative_volume_usd") cumulativeVolumeUsd: string,
    @spice.key("deposit_token_volumes") depositTokenVolumes: array<pointToken>,
  }

  @genType @spice
  type t = {
    // Unix milliseconds.
    timestamp: float,
    resolution: Shared.Resolution.t,
    // Unix milliseconds (inclusive lower bound). Wire field `from`.
    @spice.key("from") fromMs: float,
    // Unix milliseconds (exclusive upper bound). Wire field `to` (reserved word).
    @spice.key("to") toMs: float,
    @spice.key("volume_total_usd") volumeTotalUsd: string,
    @spice.key("total_days") totalDays: float,
    @spice.key("deposit_tokens") depositTokens: array<token>,
    points: array<point>,
  }
}

// ── Open-interest history ──────────────────────────────────────────────────────
module OpenInterestHistory: {
  @genType @spice
  type depositAsset = {
    rank: float,
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    symbol?: string,
    @spice.key("latest_open_interest_usd") latestOpenInterestUsd: string,
    @spice.key("max_open_interest_usd") maxOpenInterestUsd: string,
  }

  @genType @spice
  type pointDepositAsset = {
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    symbol?: string,
    @spice.key("open_interest_usd") openInterestUsd: string,
  }

  @genType @spice
  type point = {
    // Unix milliseconds (UTC day start).
    @spice.key("bucket_start") bucketStart: float,
    // Calendar day "YYYY-MM-DD".
    @spice.key("bucket_start_date") bucketStartDate: string,
    @spice.key("total_open_interest_usd") totalOpenInterestUsd: string,
    @spice.key("deposit_asset_open_interest")
    depositAssetOpenInterest: array<pointDepositAsset>,
  }

  @genType @spice
  type t = {
    // Unix milliseconds.
    timestamp: float,
    resolution: Shared.Resolution.t,
    // Unix milliseconds (inclusive lower bound). Wire field `from`.
    @spice.key("from") fromMs: float,
    // Unix milliseconds (exclusive upper bound). Wire field `to` (reserved word).
    @spice.key("to") toMs: float,
    @spice.key("latest_open_interest_usd") latestOpenInterestUsd: string,
    @spice.key("total_days") totalDays: float,
    @spice.key("deposit_assets") depositAssets: array<depositAsset>,
    points: array<point>,
  }
}

// ── Unique-traders history ─────────────────────────────────────────────────────
module UniqueTradersHistory: {
  @genType @spice
  type point = {
    // Unix milliseconds (UTC day start).
    @spice.key("bucket_start") bucketStart: float,
    // Calendar day "YYYY-MM-DD".
    @spice.key("bucket_start_date") bucketStartDate: string,
    @spice.key("unique_traders") uniqueTraders: float,
  }

  @genType @spice
  type t = {
    // Unix milliseconds.
    timestamp: float,
    resolution: Shared.Resolution.t,
    scope: UniqueTradersHistoryScope.t,
    @spice.key("scope_key") scopeKey: string,
    // Unix milliseconds (inclusive lower bound). Wire field `from`.
    @spice.key("from") fromMs: float,
    // Unix milliseconds (exclusive upper bound). Wire field `to` (reserved word).
    @spice.key("to") toMs: float,
    @spice.key("latest_unique_traders") latestUniqueTraders: float,
    @spice.key("total_days") totalDays: float,
    points: array<point>,
  }
}

// ── Leaderboard ────────────────────────────────────────────────────────────────
module Leaderboard: {
  @genType @spice
  type entry = {
    rank: float,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    slug?: string,
    @spice.key("market_name") marketName?: string,
    category?: string,
    @spice.key("volume_24h_usd") volume24hUsd: string,
    @spice.key("category_volume_share_24h_pct") categoryVolumeShare24hPct: string,
    @spice.key("platform_volume_share_24h_pct") platformVolumeShare24hPct: string,
  }

  @genType @spice
  type t = {
    entries: array<entry>,
    period: string,
  }
}

// ── History (time-series volume buckets) ───────────────────────────────────────
module History: {
  @genType @spice
  type point = {
    // Unix milliseconds (bucket start).
    @spice.key("bucket_start") bucketStart: float,
    @spice.key("volume_usd") volumeUsd: string,
  }

  @genType @spice
  type t = {
    scope: string,
    @spice.key("scope_key") scopeKey: string,
    resolution: Shared.Resolution.t,
    points: array<point>,
  }
}

// ── Per-wallet aggregates ──────────────────────────────────────────────────────
module User: {
  @genType @spice
  type t = {
    @spice.key("wallet_address") walletAddress: Shared.pubkeyStr,
    @spice.key("total_outcomes_traded") totalOutcomesTraded: float,
    @spice.key("total_volume_usd") totalVolumeUsd: string,
    @spice.key("total_referrals_used") totalReferralsUsed: float,
  }
}
