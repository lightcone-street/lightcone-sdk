// Metrics domain — platform / market / orderbook / category / deposit-token
// volume metrics, plus deposit-token volume history, open-interest history,
// unique-trader history, the market leaderboard, time-series history, and
// per-wallet aggregates. Mirrors the Rust `domain/metrics`.
//
// Unlike `Trade`, the Rust metrics domain has NO `convert.rs`: the wire structs
// in `wire.rs` ARE the values the client returns (the backend's `dto::metrics`
// shapes deserialize and flow straight through). So each type here is declared
// ONCE with BOTH `@spice` (decode the exact JSON) and `@genType` (export the
// clean TS shape) — the same dual-attribute style `Shared.res` uses for its
// newtypes — and the client functions decode directly into it (no `…OfResponse`).
//
// Representation: decimal-valued fields (volumes, USD amounts, percentages;
// Rust `Decimal`, wire JSON string) → `string`. Counts / ids → `float`.
// Timestamps carrying `#[serde(with = "chrono::serde::ts_milliseconds")]`
// (`timestamp`, `from`, `to`, `bucket_start`) → `float` (Unix **ms**). The
// un-annotated `updated_at` / `computed_at` (chrono's default RFC3339) → `string`;
// `bucket_start_date` (`NaiveDate`, "YYYY-MM-DD") → `string`. The Rust `from` / `to`
// bounds are spelled `fromMs` / `toMs` here (matching `PriceHistory.res`) — `to` is
// a reserved ReScript identifier — with `@spice.key` preserving the wire names.

// ── Enums ──────────────────────────────────────────────────────────────────────
// Scope vocabulary for `unique-traders/history` (serde `rename_all = "lowercase"`).
module UniqueTradersHistoryScope = {
  @spice
  type t =
    | @as("platform") @spice.as("platform") Platform
    | @as("market") @spice.as("market") Market
    | @as("orderbook") @spice.as("orderbook") Orderbook
    | @as("category") @spice.as("category") Category
    | @as("outcome") @spice.as("outcome") Outcome

  let toString = (scope: t) =>
    switch scope {
    | Platform => "platform"
    | Market => "market"
    | Orderbook => "orderbook"
    | Category => "category"
    | Outcome => "outcome"
    }
}

// ── Deposit-token volume (shared sub-shape) ────────────────────────────────────
// Entry in `deposit-tokens`, also nested in platform / market-detail / category.
@spice
type depositTokenVolumeMetrics = {
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

// ── Orderbook tickers (batch BBO + midpoint over REST) ─────────────────────────
@spice
type orderbookTickerEntry = {
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
  // RFC3339 string (no `ts_milliseconds` annotation on the Rust field).
  @spice.key("computed_at") computedAt?: string,
}

@spice
type orderbookTickersResponse = {tickers: array<orderbookTickerEntry>}

// ── Platform ───────────────────────────────────────────────────────────────────
@spice
type platformMetrics = {
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
  @spice.key("deposit_token_volumes") depositTokenVolumes: array<depositTokenVolumeMetrics>,
  // RFC3339 string (no `ts_milliseconds` annotation on the Rust field).
  @spice.key("updated_at") updatedAt?: string,
}

// ── Market (listing) ───────────────────────────────────────────────────────────
@spice
type marketVolumeMetrics = {
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

@spice
type marketsMetrics = {
  markets: array<marketVolumeMetrics>,
  total: float,
}

// ── Market detail (per-outcome / per-orderbook breakdowns) ─────────────────────
@spice
type outcomeVolumeMetrics = {
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

@spice
type marketOrderbookVolumeMetrics = {
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

@spice
type marketDetailMetrics = {
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
  @spice.key("outcome_volumes") outcomeVolumes: array<outcomeVolumeMetrics>,
  @spice.key("orderbook_volumes") orderbookVolumes: array<marketOrderbookVolumeMetrics>,
  @spice.key("deposit_token_volumes") depositTokenVolumes: array<depositTokenVolumeMetrics>,
}

// ── Orderbook (detail) ─────────────────────────────────────────────────────────
@spice
type orderbookVolumeMetrics = {
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

// ── Category ───────────────────────────────────────────────────────────────────
@spice
type categoryVolumeMetrics = {
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
  @spice.key("deposit_token_volumes") depositTokenVolumes: array<depositTokenVolumeMetrics>,
}

@spice
type categoriesMetrics = {categories: array<categoryVolumeMetrics>}

// ── Deposit tokens (platform-wide) ─────────────────────────────────────────────
@spice
type depositTokensMetrics = {
  @spice.key("deposit_tokens") depositTokens: array<depositTokenVolumeMetrics>,
}

// ── Deposit-token volume history ───────────────────────────────────────────────
@spice
type depositTokenVolumeHistoryToken = {
  rank: float,
  @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
  symbol?: string,
  @spice.key("volume_total_usd") volumeTotalUsd: string,
}

@spice
type depositTokenVolumeHistoryPointToken = {
  @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
  symbol?: string,
  @spice.key("volume_usd") volumeUsd: string,
}

@spice
type depositTokenVolumeHistoryPoint = {
  // Unix milliseconds (bucket start).
  @spice.key("bucket_start") bucketStart: float,
  // Calendar day "YYYY-MM-DD".
  @spice.key("bucket_start_date") bucketStartDate: string,
  @spice.key("total_volume_usd") totalVolumeUsd: string,
  @spice.key("cumulative_volume_usd") cumulativeVolumeUsd: string,
  @spice.key("deposit_token_volumes") depositTokenVolumes: array<depositTokenVolumeHistoryPointToken>,
}

@spice
type depositTokenVolumeHistory = {
  // Unix milliseconds.
  timestamp: float,
  resolution: Shared.Resolution.t,
  // Unix milliseconds (inclusive lower bound). Rust field `from`.
  @spice.key("from") fromMs: float,
  // Unix milliseconds (exclusive upper bound). Rust field `to` (reserved word).
  @spice.key("to") toMs: float,
  @spice.key("volume_total_usd") volumeTotalUsd: string,
  @spice.key("total_days") totalDays: float,
  @spice.key("deposit_tokens") depositTokens: array<depositTokenVolumeHistoryToken>,
  points: array<depositTokenVolumeHistoryPoint>,
}

// ── Open-interest history ──────────────────────────────────────────────────────
@spice
type openInterestHistoryDepositAsset = {
  rank: float,
  @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
  symbol?: string,
  @spice.key("latest_open_interest_usd") latestOpenInterestUsd: string,
  @spice.key("max_open_interest_usd") maxOpenInterestUsd: string,
}

@spice
type openInterestHistoryPointDepositAsset = {
  @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
  symbol?: string,
  @spice.key("open_interest_usd") openInterestUsd: string,
}

@spice
type openInterestHistoryPoint = {
  // Unix milliseconds (UTC day start).
  @spice.key("bucket_start") bucketStart: float,
  // Calendar day "YYYY-MM-DD".
  @spice.key("bucket_start_date") bucketStartDate: string,
  @spice.key("total_open_interest_usd") totalOpenInterestUsd: string,
  @spice.key("deposit_asset_open_interest")
  depositAssetOpenInterest: array<openInterestHistoryPointDepositAsset>,
}

@spice
type openInterestHistory = {
  // Unix milliseconds.
  timestamp: float,
  resolution: Shared.Resolution.t,
  // Unix milliseconds (inclusive lower bound). Rust field `from`.
  @spice.key("from") fromMs: float,
  // Unix milliseconds (exclusive upper bound). Rust field `to` (reserved word).
  @spice.key("to") toMs: float,
  @spice.key("latest_open_interest_usd") latestOpenInterestUsd: string,
  @spice.key("total_days") totalDays: float,
  @spice.key("deposit_assets") depositAssets: array<openInterestHistoryDepositAsset>,
  points: array<openInterestHistoryPoint>,
}

// ── Unique-traders history ─────────────────────────────────────────────────────
@spice
type uniqueTradersHistoryPoint = {
  // Unix milliseconds (UTC day start).
  @spice.key("bucket_start") bucketStart: float,
  // Calendar day "YYYY-MM-DD".
  @spice.key("bucket_start_date") bucketStartDate: string,
  @spice.key("unique_traders") uniqueTraders: float,
}

@spice
type uniqueTradersHistory = {
  // Unix milliseconds.
  timestamp: float,
  resolution: Shared.Resolution.t,
  scope: UniqueTradersHistoryScope.t,
  @spice.key("scope_key") scopeKey: string,
  // Unix milliseconds (inclusive lower bound). Rust field `from`.
  @spice.key("from") fromMs: float,
  // Unix milliseconds (exclusive upper bound). Rust field `to` (reserved word).
  @spice.key("to") toMs: float,
  @spice.key("latest_unique_traders") latestUniqueTraders: float,
  @spice.key("total_days") totalDays: float,
  points: array<uniqueTradersHistoryPoint>,
}

// ── Leaderboard ────────────────────────────────────────────────────────────────
@spice
type leaderboardEntry = {
  rank: float,
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  slug?: string,
  @spice.key("market_name") marketName?: string,
  category?: string,
  @spice.key("volume_24h_usd") volume24hUsd: string,
  @spice.key("category_volume_share_24h_pct") categoryVolumeShare24hPct: string,
  @spice.key("platform_volume_share_24h_pct") platformVolumeShare24hPct: string,
}

@spice
type leaderboard = {
  entries: array<leaderboardEntry>,
  period: string,
}

// ── History (time-series volume buckets) ───────────────────────────────────────
@spice
type historyPoint = {
  // Unix milliseconds (bucket start).
  @spice.key("bucket_start") bucketStart: float,
  @spice.key("volume_usd") volumeUsd: string,
}

@spice
type metricsHistory = {
  scope: string,
  @spice.key("scope_key") scopeKey: string,
  resolution: Shared.Resolution.t,
  points: array<historyPoint>,
}

// ── Per-wallet aggregates ──────────────────────────────────────────────────────
@spice
type userMetrics = {
  @spice.key("wallet_address") walletAddress: Shared.pubkeyStr,
  @spice.key("total_outcomes_traded") totalOutcomesTraded: float,
  @spice.key("total_volume_usd") totalVolumeUsd: string,
  @spice.key("total_referrals_used") totalReferralsUsed: float,
}

// ── Client functions ───────────────────────────────────────────────────────────
let optionalQuery = (query, key, value) =>
  value->Option.forEach(value => query->Array.push((key, value)))

// Platform-wide metrics: total volume, trader counts, active market/orderbook
// counts, and per-deposit-token breakdowns.
let platform = async (client: Client.t): result<platformMetrics, SdkError.t> =>
  await Http.get(client.http, ~path="/api/metrics/platform", ~decode=platformMetrics_decode)

// Metrics for all active markets. (The Rust `MarketsMetricsQuery` is an empty
// reserved-for-future struct, so this takes no query.)
let markets = async (client: Client.t): result<marketsMetrics, SdkError.t> =>
  await Http.get(client.http, ~path="/api/metrics/markets", ~decode=marketsMetrics_decode)

// Detailed metrics for one market — per-outcome, per-orderbook, and
// per-deposit-token breakdowns.
let market = async (
  client: Client.t,
  ~marketPubkey: string,
): result<marketDetailMetrics, SdkError.t> => {
  let path = `/api/metrics/markets/${encodeURIComponent(marketPubkey)}`
  await Http.get(client.http, ~path, ~decode=marketDetailMetrics_decode)
}

// Batch BBO + midpoint per active orderbook (same shape as the WS `Ticker`
// stream). Optionally filter to orderbooks whose base token is backed by
// `depositAsset` (trimmed; ignored when empty).
let orderbookTickers = async (
  client: Client.t,
  ~depositAsset: option<string>=?,
): result<orderbookTickersResponse, SdkError.t> => {
  let query: array<(string, string)> = []
  switch depositAsset->Option.map(value => String.trim(value)) {
  | Some(mint) if mint != "" => query->Array.push(("deposit_asset", mint))
  | _ => ()
  }
  await Http.get(
    client.http,
    ~path="/api/metrics/orderbooks/tickers",
    ~query,
    ~decode=orderbookTickersResponse_decode,
  )
}

// Metrics for one orderbook, broken down by base / quote / USD volume.
let orderbook = async (
  client: Client.t,
  ~orderbookId: string,
): result<orderbookVolumeMetrics, SdkError.t> => {
  let path = `/api/metrics/orderbooks/${encodeURIComponent(orderbookId)}`
  await Http.get(client.http, ~path, ~decode=orderbookVolumeMetrics_decode)
}

// Metrics for every market category (e.g. Politics, Sports).
let categories = async (client: Client.t): result<categoriesMetrics, SdkError.t> =>
  await Http.get(client.http, ~path="/api/metrics/categories", ~decode=categoriesMetrics_decode)

// Metrics for a single category (URL-encoded).
let category = async (
  client: Client.t,
  ~category: string,
): result<categoryVolumeMetrics, SdkError.t> => {
  let path = `/api/metrics/categories/${encodeURIComponent(category)}`
  await Http.get(client.http, ~path, ~decode=categoryVolumeMetrics_decode)
}

// Per-deposit-token volumes across the entire platform.
let depositTokens = async (client: Client.t): result<depositTokensMetrics, SdkError.t> =>
  await Http.get(client.http, ~path="/api/metrics/deposit-tokens", ~decode=depositTokensMetrics_decode)

// Daily platform volume history broken down by deposit token. `from`/`to` are
// Unix ms (inclusive / exclusive); `limit` defaults to the backend max (5000).
let depositTokensVolumeHistory = async (
  client: Client.t,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~limit: option<int>=?,
): result<depositTokenVolumeHistory, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "from", fromMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "to", toMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path="/api/metrics/deposit-tokens/volume-history",
    ~query,
    ~decode=depositTokenVolumeHistory_decode,
  )
}

// Daily platform open-interest snapshots by deposit asset. Open interest is a
// live snapshot (not cumulative) — do not sum across days.
let openInterestHistory = async (
  client: Client.t,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~limit: option<int>=?,
): result<openInterestHistory, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "from", fromMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "to", toMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path="/api/metrics/open-interest/history",
    ~query,
    ~decode=openInterestHistory_decode,
  )
}

// Daily unique trader counts for the platform or a scoped entity. With no
// `scope`, the backend returns platform-wide history; for other scopes provide
// both `scope` and `scopeKey`.
let uniqueTradersHistory = async (
  client: Client.t,
  ~scope: option<UniqueTradersHistoryScope.t>=?,
  ~scopeKey: option<string>=?,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~limit: option<int>=?,
): result<uniqueTradersHistory, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "scope", scope->Option.map(value => UniqueTradersHistoryScope.toString(value)))
  optionalQuery(query, "scope_key", scopeKey)
  optionalQuery(query, "from", fromMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "to", toMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path="/api/metrics/unique-traders/history",
    ~query,
    ~decode=uniqueTradersHistory_decode,
  )
}

// Market leaderboard (top markets by 24h volume). `limit` defaults to the
// backend setting (currently 20) when omitted.
let leaderboard = async (
  client: Client.t,
  ~limit: option<int>=?,
): result<leaderboard, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path="/api/metrics/leaderboard/markets",
    ~query,
    ~decode=leaderboard_decode,
  )
}

// Time-series of volume buckets for the given scope + key. `scope` is one of
// "orderbook" | "market" | "category" | "deposit_token" | "platform"; `scopeKey`
// is the corresponding id. `resolution` defaults to 1h with no time bounds.
let history = async (
  client: Client.t,
  ~scope: string,
  ~scopeKey: string,
  ~resolution: Shared.Resolution.t=Shared.Resolution.Hour1,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~limit: option<int>=?,
): result<metricsHistory, SdkError.t> => {
  let path = `/api/metrics/history/${encodeURIComponent(scope)}/${encodeURIComponent(scopeKey)}`
  let query = [("resolution", Shared.Resolution.toString(resolution))]
  optionalQuery(query, "from", fromMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "to", toMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(client.http, ~path, ~query, ~decode=metricsHistory_decode)
}

// Per-wallet trading + referral aggregates for the authenticated user (the
// wallet is resolved server-side from the auth cookie). Pass `~cookieHeader` to
// forward a raw `Cookie` header for this single call (SSR / server-function use)
// instead of the SDK's process-wide token — this covers the Rust
// `user_with_cookies` variant.
let user = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<userMetrics, SdkError.t> =>
  await Http.get(client.http, ~path="/api/metrics/user", ~cookieHeader?, ~decode=userMetrics_decode)

// Public path-based variant of `user`: takes the wallet via the URL and needs
// no auth (`GET /api/metrics/user/{wallet_address}`).
let userByWallet = async (
  client: Client.t,
  ~walletAddress: string,
): result<userMetrics, SdkError.t> => {
  let path = `/api/metrics/user/${encodeURIComponent(walletAddress)}`
  await Http.get(client.http, ~path, ~decode=userMetrics_decode)
}
