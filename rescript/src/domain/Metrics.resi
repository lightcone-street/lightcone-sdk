// Metrics domain — platform / market / orderbook / category / deposit-token volume
// metrics, plus volume / open-interest / unique-trader history series, the market
// leaderboard, generic time-series history, and per-wallet aggregates. Public
// surface: the gentype-exported metric record types, the `UniqueTradersHistoryScope`
// enum, and the client query functions. The spice JSON codecs (and the
// query-string helpers) are internal.

// ── Enums ──────────────────────────────────────────────────────────────────────
module UniqueTradersHistoryScope: {
  @genType
  type t =
    | @as("platform") Platform
    | @as("market") Market
    | @as("orderbook") Orderbook
    | @as("category") Category
    | @as("outcome") Outcome
}

// ── Deposit-token volume (shared sub-shape) ────────────────────────────────────
// Entry in `deposit-tokens`, also nested in platform / market-detail / category.
@genType
type depositTokenVolumeMetrics = {
  depositAsset: Shared.pubkeyStr,
  symbol?: string,
  volume24hUsd: string,
  volume7dUsd: string,
  volume30dUsd: string,
  volumeTotalUsd: string,
  takerBidVolume24hUsd: string,
  takerBidVolume7dUsd: string,
  takerBidVolume30dUsd: string,
  takerBidVolumeTotalUsd: string,
  takerAskVolume24hUsd: string,
  takerAskVolume7dUsd: string,
  takerAskVolume30dUsd: string,
  takerAskVolumeTotalUsd: string,
  takerBidAskImbalance24hPct: string,
  takerBidAskImbalance7dPct: string,
  takerBidAskImbalance30dPct: string,
  takerBidAskImbalanceTotalPct: string,
  volumeShare24hPct: string,
}

// ── Orderbook tickers (batch BBO + midpoint over REST) ─────────────────────────
@genType
type orderbookTickerEntry = {
  orderbookId: Shared.orderBookId,
  marketPubkey: Shared.pubkeyStr,
  outcomeIndex?: float,
  outcomeName?: string,
  outcomeNameLong?: string,
  baseDepositAsset: Shared.pubkeyStr,
  quoteDepositAsset: Shared.pubkeyStr,
  bestBid?: string,
  bestAsk?: string,
  midpoint?: string,
  // RFC3339 string (no `ts_milliseconds` annotation on the Rust field).
  computedAt?: string,
}

@genType
type orderbookTickersResponse = {tickers: array<orderbookTickerEntry>}

// ── Platform ───────────────────────────────────────────────────────────────────
@genType
type platformMetrics = {
  volume24hUsd: string,
  volume7dUsd: string,
  volume30dUsd: string,
  volumeTotalUsd: string,
  takerBidVolume24hUsd: string,
  takerBidVolume7dUsd: string,
  takerBidVolume30dUsd: string,
  takerBidVolumeTotalUsd: string,
  takerAskVolume24hUsd: string,
  takerAskVolume7dUsd: string,
  takerAskVolume30dUsd: string,
  takerAskVolumeTotalUsd: string,
  takerBidAskImbalance24hPct: string,
  takerBidAskImbalance7dPct: string,
  takerBidAskImbalance30dPct: string,
  takerBidAskImbalanceTotalPct: string,
  openInterestUsd: string,
  fees24hUsd: string,
  fees7dUsd: string,
  fees30dUsd: string,
  uniqueTraders24h: float,
  uniqueTraders7d: float,
  uniqueTraders30d: float,
  activeMarkets: float,
  activeOrderbooks: float,
  depositTokenVolumes: array<depositTokenVolumeMetrics>,
  // RFC3339 string (no `ts_milliseconds` annotation on the Rust field).
  updatedAt?: string,
}

// ── Market (listing) ───────────────────────────────────────────────────────────
@genType
type marketVolumeMetrics = {
  marketPubkey: Shared.pubkeyStr,
  slug?: string,
  marketName?: string,
  category?: string,
  volume24hUsd: string,
  volume7dUsd: string,
  volume30dUsd: string,
  volumeTotalUsd: string,
  takerBidVolume24hUsd: string,
  takerBidVolume7dUsd: string,
  takerBidVolume30dUsd: string,
  takerBidVolumeTotalUsd: string,
  takerAskVolume24hUsd: string,
  takerAskVolume7dUsd: string,
  takerAskVolume30dUsd: string,
  takerAskVolumeTotalUsd: string,
  takerBidAskImbalance24hPct: string,
  takerBidAskImbalance7dPct: string,
  takerBidAskImbalance30dPct: string,
  takerBidAskImbalanceTotalPct: string,
  uniqueTraders24h: float,
  uniqueTraders7d: float,
  uniqueTraders30d: float,
  categoryVolumeShare24hPct: string,
  platformVolumeShare24hPct: string,
}

@genType
type marketsMetrics = {
  markets: array<marketVolumeMetrics>,
  total: float,
}

// ── Market detail (per-outcome / per-orderbook breakdowns) ─────────────────────
@genType
type outcomeVolumeMetrics = {
  outcomeIndex?: float,
  outcomeName?: string,
  outcomeNameLong?: string,
  volume24hUsd: string,
  volume7dUsd: string,
  volume30dUsd: string,
  volumeTotalUsd: string,
  takerBidVolume24hUsd: string,
  takerBidVolume7dUsd: string,
  takerBidVolume30dUsd: string,
  takerBidVolumeTotalUsd: string,
  takerAskVolume24hUsd: string,
  takerAskVolume7dUsd: string,
  takerAskVolume30dUsd: string,
  takerAskVolumeTotalUsd: string,
  takerBidAskImbalance24hPct: string,
  takerBidAskImbalance7dPct: string,
  takerBidAskImbalance30dPct: string,
  takerBidAskImbalanceTotalPct: string,
  uniqueTraders24h: float,
  uniqueTraders7d: float,
  uniqueTraders30d: float,
  volumeShare24hPct: string,
}

@genType
type marketOrderbookVolumeMetrics = {
  orderbookId: Shared.orderBookId,
  outcomeIndex?: float,
  outcomeName?: string,
  outcomeNameLong?: string,
  baseDepositAsset: Shared.pubkeyStr,
  baseDepositSymbol?: string,
  quoteDepositAsset: Shared.pubkeyStr,
  quoteDepositSymbol?: string,
  volume24hUsd: string,
  volume7dUsd: string,
  volume30dUsd: string,
  volumeTotalUsd: string,
  volume24hBase: string,
  volume7dBase: string,
  volume30dBase: string,
  volumeTotalBase: string,
  volume24hQuote: string,
  volume7dQuote: string,
  volume30dQuote: string,
  volumeTotalQuote: string,
  takerBidVolume24hUsd: string,
  takerBidVolume7dUsd: string,
  takerBidVolume30dUsd: string,
  takerBidVolumeTotalUsd: string,
  takerBidVolume24hBase: string,
  takerBidVolume7dBase: string,
  takerBidVolume30dBase: string,
  takerBidVolumeTotalBase: string,
  takerBidVolume24hQuote: string,
  takerBidVolume7dQuote: string,
  takerBidVolume30dQuote: string,
  takerBidVolumeTotalQuote: string,
  takerAskVolume24hUsd: string,
  takerAskVolume7dUsd: string,
  takerAskVolume30dUsd: string,
  takerAskVolumeTotalUsd: string,
  takerAskVolume24hBase: string,
  takerAskVolume7dBase: string,
  takerAskVolume30dBase: string,
  takerAskVolumeTotalBase: string,
  takerAskVolume24hQuote: string,
  takerAskVolume7dQuote: string,
  takerAskVolume30dQuote: string,
  takerAskVolumeTotalQuote: string,
  takerBidAskImbalance24hPct: string,
  takerBidAskImbalance7dPct: string,
  takerBidAskImbalance30dPct: string,
  takerBidAskImbalanceTotalPct: string,
  volumeShare24hPct: string,
}

@genType
type marketDetailMetrics = {
  marketPubkey: Shared.pubkeyStr,
  slug?: string,
  marketName?: string,
  category?: string,
  volume24hUsd: string,
  volume7dUsd: string,
  volume30dUsd: string,
  volumeTotalUsd: string,
  takerBidVolume24hUsd: string,
  takerBidVolume7dUsd: string,
  takerBidVolume30dUsd: string,
  takerBidVolumeTotalUsd: string,
  takerAskVolume24hUsd: string,
  takerAskVolume7dUsd: string,
  takerAskVolume30dUsd: string,
  takerAskVolumeTotalUsd: string,
  takerBidAskImbalance24hPct: string,
  takerBidAskImbalance7dPct: string,
  takerBidAskImbalance30dPct: string,
  takerBidAskImbalanceTotalPct: string,
  uniqueTraders24h: float,
  uniqueTraders7d: float,
  uniqueTraders30d: float,
  categoryVolumeShare24hPct: string,
  platformVolumeShare24hPct: string,
  outcomeVolumes: array<outcomeVolumeMetrics>,
  orderbookVolumes: array<marketOrderbookVolumeMetrics>,
  depositTokenVolumes: array<depositTokenVolumeMetrics>,
}

// ── Orderbook (detail) ─────────────────────────────────────────────────────────
@genType
type orderbookVolumeMetrics = {
  orderbookId: Shared.orderBookId,
  marketPubkey: Shared.pubkeyStr,
  outcomeIndex?: float,
  outcomeName?: string,
  outcomeNameLong?: string,
  baseDepositAsset: Shared.pubkeyStr,
  baseDepositSymbol?: string,
  quoteDepositAsset: Shared.pubkeyStr,
  quoteDepositSymbol?: string,
  volume24hUsd: string,
  volume7dUsd: string,
  volume30dUsd: string,
  volumeTotalUsd: string,
  volume24hBase: string,
  volume7dBase: string,
  volume30dBase: string,
  volumeTotalBase: string,
  volume24hQuote: string,
  volume7dQuote: string,
  volume30dQuote: string,
  volumeTotalQuote: string,
  takerBidVolume24hUsd: string,
  takerBidVolume7dUsd: string,
  takerBidVolume30dUsd: string,
  takerBidVolumeTotalUsd: string,
  takerBidVolume24hBase: string,
  takerBidVolume7dBase: string,
  takerBidVolume30dBase: string,
  takerBidVolumeTotalBase: string,
  takerBidVolume24hQuote: string,
  takerBidVolume7dQuote: string,
  takerBidVolume30dQuote: string,
  takerBidVolumeTotalQuote: string,
  takerAskVolume24hUsd: string,
  takerAskVolume7dUsd: string,
  takerAskVolume30dUsd: string,
  takerAskVolumeTotalUsd: string,
  takerAskVolume24hBase: string,
  takerAskVolume7dBase: string,
  takerAskVolume30dBase: string,
  takerAskVolumeTotalBase: string,
  takerAskVolume24hQuote: string,
  takerAskVolume7dQuote: string,
  takerAskVolume30dQuote: string,
  takerAskVolumeTotalQuote: string,
  takerBidAskImbalance24hPct: string,
  takerBidAskImbalance7dPct: string,
  takerBidAskImbalance30dPct: string,
  takerBidAskImbalanceTotalPct: string,
  uniqueTraders24h: float,
  uniqueTraders7d: float,
  uniqueTraders30d: float,
  marketVolumeShare24hPct: string,
}

// ── Category ───────────────────────────────────────────────────────────────────
@genType
type categoryVolumeMetrics = {
  category: string,
  volume24hUsd: string,
  volume7dUsd: string,
  volume30dUsd: string,
  volumeTotalUsd: string,
  takerBidVolume24hUsd: string,
  takerBidVolume7dUsd: string,
  takerBidVolume30dUsd: string,
  takerBidVolumeTotalUsd: string,
  takerAskVolume24hUsd: string,
  takerAskVolume7dUsd: string,
  takerAskVolume30dUsd: string,
  takerAskVolumeTotalUsd: string,
  takerBidAskImbalance24hPct: string,
  takerBidAskImbalance7dPct: string,
  takerBidAskImbalance30dPct: string,
  takerBidAskImbalanceTotalPct: string,
  uniqueTraders24h: float,
  uniqueTraders7d: float,
  uniqueTraders30d: float,
  platformVolumeShare24hPct: string,
  depositTokenVolumes: array<depositTokenVolumeMetrics>,
}

@genType
type categoriesMetrics = {categories: array<categoryVolumeMetrics>}

// ── Deposit tokens (platform-wide) ─────────────────────────────────────────────
@genType
type depositTokensMetrics = {
  depositTokens: array<depositTokenVolumeMetrics>,
}

// ── Deposit-token volume history ───────────────────────────────────────────────
@genType
type depositTokenVolumeHistoryToken = {
  rank: float,
  depositAsset: Shared.pubkeyStr,
  symbol?: string,
  volumeTotalUsd: string,
}

@genType
type depositTokenVolumeHistoryPointToken = {
  depositAsset: Shared.pubkeyStr,
  symbol?: string,
  volumeUsd: string,
}

@genType
type depositTokenVolumeHistoryPoint = {
  // Unix milliseconds (bucket start).
  bucketStart: float,
  // Calendar day "YYYY-MM-DD".
  bucketStartDate: string,
  totalVolumeUsd: string,
  cumulativeVolumeUsd: string,
  depositTokenVolumes: array<depositTokenVolumeHistoryPointToken>,
}

@genType
type depositTokenVolumeHistory = {
  // Unix milliseconds.
  timestamp: float,
  resolution: Shared.Resolution.t,
  // Unix milliseconds (inclusive lower bound). Rust field `from`.
  fromMs: float,
  // Unix milliseconds (exclusive upper bound). Rust field `to` (reserved word).
  toMs: float,
  volumeTotalUsd: string,
  totalDays: float,
  depositTokens: array<depositTokenVolumeHistoryToken>,
  points: array<depositTokenVolumeHistoryPoint>,
}

// ── Open-interest history ──────────────────────────────────────────────────────
@genType
type openInterestHistoryDepositAsset = {
  rank: float,
  depositAsset: Shared.pubkeyStr,
  symbol?: string,
  latestOpenInterestUsd: string,
  maxOpenInterestUsd: string,
}

@genType
type openInterestHistoryPointDepositAsset = {
  depositAsset: Shared.pubkeyStr,
  symbol?: string,
  openInterestUsd: string,
}

@genType
type openInterestHistoryPoint = {
  // Unix milliseconds (UTC day start).
  bucketStart: float,
  // Calendar day "YYYY-MM-DD".
  bucketStartDate: string,
  totalOpenInterestUsd: string,
  depositAssetOpenInterest: array<openInterestHistoryPointDepositAsset>,
}

@genType
type openInterestHistory = {
  // Unix milliseconds.
  timestamp: float,
  resolution: Shared.Resolution.t,
  // Unix milliseconds (inclusive lower bound). Rust field `from`.
  fromMs: float,
  // Unix milliseconds (exclusive upper bound). Rust field `to` (reserved word).
  toMs: float,
  latestOpenInterestUsd: string,
  totalDays: float,
  depositAssets: array<openInterestHistoryDepositAsset>,
  points: array<openInterestHistoryPoint>,
}

// ── Unique-traders history ─────────────────────────────────────────────────────
@genType
type uniqueTradersHistoryPoint = {
  // Unix milliseconds (UTC day start).
  bucketStart: float,
  // Calendar day "YYYY-MM-DD".
  bucketStartDate: string,
  uniqueTraders: float,
}

@genType
type uniqueTradersHistory = {
  // Unix milliseconds.
  timestamp: float,
  resolution: Shared.Resolution.t,
  scope: UniqueTradersHistoryScope.t,
  scopeKey: string,
  // Unix milliseconds (inclusive lower bound). Rust field `from`.
  fromMs: float,
  // Unix milliseconds (exclusive upper bound). Rust field `to` (reserved word).
  toMs: float,
  latestUniqueTraders: float,
  totalDays: float,
  points: array<uniqueTradersHistoryPoint>,
}

// ── Leaderboard ────────────────────────────────────────────────────────────────
@genType
type leaderboardEntry = {
  rank: float,
  marketPubkey: Shared.pubkeyStr,
  slug?: string,
  marketName?: string,
  category?: string,
  volume24hUsd: string,
  categoryVolumeShare24hPct: string,
  platformVolumeShare24hPct: string,
}

@genType
type leaderboard = {
  entries: array<leaderboardEntry>,
  period: string,
}

// ── History (time-series volume buckets) ───────────────────────────────────────
@genType
type historyPoint = {
  // Unix milliseconds (bucket start).
  bucketStart: float,
  volumeUsd: string,
}

@genType
type metricsHistory = {
  scope: string,
  scopeKey: string,
  resolution: Shared.Resolution.t,
  points: array<historyPoint>,
}

// ── Per-wallet aggregates ──────────────────────────────────────────────────────
@genType
type userMetrics = {
  walletAddress: Shared.pubkeyStr,
  totalOutcomesTraded: float,
  totalVolumeUsd: string,
  totalReferralsUsed: float,
}

// ── Client functions ───────────────────────────────────────────────────────────

// Platform-wide totals, trader counts, active market/orderbook counts, and
// per-deposit-token breakdowns.
let platform: Client.t => promise<result<platformMetrics, SdkError.t>>

// Metrics for all active markets.
let markets: Client.t => promise<result<marketsMetrics, SdkError.t>>

// Detailed metrics for one market (per-outcome / per-orderbook / per-token).
let market: (Client.t, ~marketPubkey: string) => promise<result<marketDetailMetrics, SdkError.t>>

// Batch BBO + midpoint per active orderbook, optionally filtered by base deposit asset.
let orderbookTickers: (
  Client.t,
  ~depositAsset: string=?,
) => promise<result<orderbookTickersResponse, SdkError.t>>

// Metrics for one orderbook, broken down by base / quote / USD volume.
let orderbook: (Client.t, ~orderbookId: string) => promise<result<orderbookVolumeMetrics, SdkError.t>>

// Metrics for every market category.
let categories: Client.t => promise<result<categoriesMetrics, SdkError.t>>

// Metrics for a single category (URL-encoded).
let category: (Client.t, ~category: string) => promise<result<categoryVolumeMetrics, SdkError.t>>

// Per-deposit-token volumes across the entire platform.
let depositTokens: Client.t => promise<result<depositTokensMetrics, SdkError.t>>

// Daily platform volume history broken down by deposit token.
let depositTokensVolumeHistory: (
  Client.t,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~limit: int=?,
) => promise<result<depositTokenVolumeHistory, SdkError.t>>

// Daily platform open-interest snapshots by deposit asset.
let openInterestHistory: (
  Client.t,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~limit: int=?,
) => promise<result<openInterestHistory, SdkError.t>>

// Daily unique-trader counts for the platform or a scoped entity.
let uniqueTradersHistory: (
  Client.t,
  ~scope: UniqueTradersHistoryScope.t=?,
  ~scopeKey: string=?,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~limit: int=?,
) => promise<result<uniqueTradersHistory, SdkError.t>>

// Market leaderboard (top markets by 24h volume).
let leaderboard: (Client.t, ~limit: int=?) => promise<result<leaderboard, SdkError.t>>

// Time-series of volume buckets for the given scope + key.
let history: (
  Client.t,
  ~scope: string,
  ~scopeKey: string,
  ~resolution: Shared.Resolution.t=?,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~limit: int=?,
) => promise<result<metricsHistory, SdkError.t>>

// Per-wallet trading + referral aggregates for the authenticated user.
let user: (Client.t, ~cookieHeader: string=?) => promise<result<userMetrics, SdkError.t>>

// Public path-based variant of `user` (no auth; wallet via URL).
let userByWallet: (Client.t, ~walletAddress: string) => promise<result<userMetrics, SdkError.t>>
