// Metrics client — platform / market / orderbook / category / deposit-token
// volume metrics, plus volume / open-interest / unique-trader history series,
// the market leaderboard, generic time-series history, and per-wallet
// aggregates. The query-string helper is internal.

// Platform-wide totals, trader counts, active market/orderbook counts, and
// per-deposit-token breakdowns.
let platform: Client.t => promise<result<Metrics__Raw.Platform.t, SdkError.t>>

// Metrics for all active markets.
let markets: Client.t => promise<result<Metrics__Raw.Markets.t, SdkError.t>>

// Detailed metrics for one market (per-outcome / per-orderbook / per-token).
let market: (
  Client.t,
  ~marketPubkey: string,
) => promise<result<Metrics__Raw.MarketDetail.t, SdkError.t>>

// Batch BBO + midpoint per active orderbook, optionally filtered by base deposit asset.
let orderbookTickers: (
  Client.t,
  ~depositAsset: string=?,
) => promise<result<Metrics__Raw.OrderbookTickersResponse.t, SdkError.t>>

// Metrics for one orderbook, broken down by base / quote / USD volume.
let orderbook: (
  Client.t,
  ~orderbookId: string,
) => promise<result<Metrics__Raw.OrderbookVolume.t, SdkError.t>>

// Metrics for every market category.
let categories: Client.t => promise<result<Metrics__Raw.Categories.t, SdkError.t>>

// Metrics for a single category (URL-encoded).
let category: (
  Client.t,
  ~category: string,
) => promise<result<Metrics__Raw.CategoryVolume.t, SdkError.t>>

// Per-deposit-token volumes across the entire platform.
let depositTokens: Client.t => promise<result<Metrics__Raw.DepositTokens.t, SdkError.t>>

// Daily platform volume history broken down by deposit token.
let depositTokensVolumeHistory: (
  Client.t,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~limit: int=?,
) => promise<result<Metrics__Raw.DepositTokenVolumeHistory.t, SdkError.t>>

// Daily platform open-interest snapshots by deposit asset.
let openInterestHistory: (
  Client.t,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~limit: int=?,
) => promise<result<Metrics__Raw.OpenInterestHistory.t, SdkError.t>>

// Daily unique-trader counts for the platform or a scoped entity.
let uniqueTradersHistory: (
  Client.t,
  ~scope: Metrics__Raw.UniqueTradersHistoryScope.t=?,
  ~scopeKey: string=?,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~limit: int=?,
) => promise<result<Metrics__Raw.UniqueTradersHistory.t, SdkError.t>>

// Market leaderboard (top markets by 24h volume).
let leaderboard: (Client.t, ~limit: int=?) => promise<result<Metrics__Raw.Leaderboard.t, SdkError.t>>

// Time-series of volume buckets for the given scope + key.
let history: (
  Client.t,
  ~scope: string,
  ~scopeKey: string,
  ~resolution: Shared.Resolution.t=?,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~limit: int=?,
) => promise<result<Metrics__Raw.History.t, SdkError.t>>

// Per-wallet trading + referral aggregates for the authenticated user.
let user: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<Metrics__Raw.User.t, SdkError.t>>

// Public path-based variant of `user` (no auth; wallet via URL).
let userByWallet: (
  Client.t,
  ~walletAddress: string,
) => promise<result<Metrics__Raw.User.t, SdkError.t>>
