// Metrics client — platform / market / orderbook / category / deposit-token
// volume metrics, plus deposit-token volume history, open-interest history,
// unique-trader history, the market leaderboard, time-series history, and
// per-wallet aggregates. Each call decodes straight into its `Metrics__Raw`
// wire type (no domain conversion).

let optionalQuery = (query, key, value) =>
  value->Option.forEach(value => query->Array.push((key, value)))

// Platform-wide metrics: total volume, trader counts, active market/orderbook
// counts, and per-deposit-token breakdowns.
let platform = async (client: Client.t): result<Metrics__Raw.Platform.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/metrics/platform",
    ~decode=Metrics__Raw.Platform.t_decode,
  )

// Metrics for all active markets.
let markets = async (client: Client.t): result<Metrics__Raw.Markets.t, SdkError.t> =>
  await Http.get(client.http, ~path="/api/metrics/markets", ~decode=Metrics__Raw.Markets.t_decode)

// Detailed metrics for one market — per-outcome, per-orderbook, and
// per-deposit-token breakdowns.
let market = async (
  client: Client.t,
  ~marketPubkey: string,
): result<Metrics__Raw.MarketDetail.t, SdkError.t> => {
  let path = `/api/metrics/markets/${encodeURIComponent(marketPubkey)}`
  await Http.get(client.http, ~path, ~decode=Metrics__Raw.MarketDetail.t_decode)
}

// Batch BBO + midpoint per active orderbook (same shape as the WS `Ticker`
// stream). Optionally filter to orderbooks whose base token is backed by
// `depositAsset` (trimmed; ignored when empty).
let orderbookTickers = async (
  client: Client.t,
  ~depositAsset: option<string>=?,
): result<Metrics__Raw.OrderbookTickersResponse.t, SdkError.t> => {
  let query: array<(string, string)> = []
  switch depositAsset->Option.map(value => String.trim(value)) {
  | Some(mint) if mint != "" => query->Array.push(("deposit_asset", mint))
  | _ => ()
  }
  await Http.get(
    client.http,
    ~path="/api/metrics/orderbooks/tickers",
    ~query,
    ~decode=Metrics__Raw.OrderbookTickersResponse.t_decode,
  )
}

// Metrics for one orderbook, broken down by base / quote / USD volume.
let orderbook = async (
  client: Client.t,
  ~orderbookId: string,
): result<Metrics__Raw.OrderbookVolume.t, SdkError.t> => {
  let path = `/api/metrics/orderbooks/${encodeURIComponent(orderbookId)}`
  await Http.get(client.http, ~path, ~decode=Metrics__Raw.OrderbookVolume.t_decode)
}

// Metrics for every market category (e.g. Politics, Sports).
let categories = async (client: Client.t): result<Metrics__Raw.Categories.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/metrics/categories",
    ~decode=Metrics__Raw.Categories.t_decode,
  )

// Metrics for a single category (URL-encoded).
let category = async (
  client: Client.t,
  ~category: string,
): result<Metrics__Raw.CategoryVolume.t, SdkError.t> => {
  let path = `/api/metrics/categories/${encodeURIComponent(category)}`
  await Http.get(client.http, ~path, ~decode=Metrics__Raw.CategoryVolume.t_decode)
}

// Per-deposit-token volumes across the entire platform.
let depositTokens = async (client: Client.t): result<Metrics__Raw.DepositTokens.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/metrics/deposit-tokens",
    ~decode=Metrics__Raw.DepositTokens.t_decode,
  )

// Daily platform volume history broken down by deposit token. `from`/`to` are
// Unix ms (inclusive / exclusive); `limit` defaults to the backend max (5000).
let depositTokensVolumeHistory = async (
  client: Client.t,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~limit: option<int>=?,
): result<Metrics__Raw.DepositTokenVolumeHistory.t, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "from", fromMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "to", toMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path="/api/metrics/deposit-tokens/volume-history",
    ~query,
    ~decode=Metrics__Raw.DepositTokenVolumeHistory.t_decode,
  )
}

// Daily platform open-interest snapshots by deposit asset. Open interest is a
// live snapshot (not cumulative) — do not sum across days.
let openInterestHistory = async (
  client: Client.t,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~limit: option<int>=?,
): result<Metrics__Raw.OpenInterestHistory.t, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "from", fromMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "to", toMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path="/api/metrics/open-interest/history",
    ~query,
    ~decode=Metrics__Raw.OpenInterestHistory.t_decode,
  )
}

// Daily unique trader counts for the platform or a scoped entity. With no
// `scope`, the backend returns platform-wide history; for other scopes provide
// both `scope` and `scopeKey`.
let uniqueTradersHistory = async (
  client: Client.t,
  ~scope: option<Metrics__Raw.UniqueTradersHistoryScope.t>=?,
  ~scopeKey: option<string>=?,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~limit: option<int>=?,
): result<Metrics__Raw.UniqueTradersHistory.t, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(
    query,
    "scope",
    scope->Option.map(value => Metrics__Raw.UniqueTradersHistoryScope.toString(value)),
  )
  optionalQuery(query, "scope_key", scopeKey)
  optionalQuery(query, "from", fromMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "to", toMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path="/api/metrics/unique-traders/history",
    ~query,
    ~decode=Metrics__Raw.UniqueTradersHistory.t_decode,
  )
}

// Market leaderboard (top markets by 24h volume). `limit` defaults to the
// backend setting (currently 20) when omitted.
let leaderboard = async (
  client: Client.t,
  ~limit: option<int>=?,
): result<Metrics__Raw.Leaderboard.t, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path="/api/metrics/leaderboard/markets",
    ~query,
    ~decode=Metrics__Raw.Leaderboard.t_decode,
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
): result<Metrics__Raw.History.t, SdkError.t> => {
  let path = `/api/metrics/history/${encodeURIComponent(scope)}/${encodeURIComponent(scopeKey)}`
  let query = [("resolution", Shared.Resolution.toString(resolution))]
  optionalQuery(query, "from", fromMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "to", toMs->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(client.http, ~path, ~query, ~decode=Metrics__Raw.History.t_decode)
}

// Per-wallet trading + referral aggregates for the authenticated user (the
// wallet is resolved server-side from the auth cookie). Pass `~cookieHeader` to
// forward a raw `Cookie` header for this single call (SSR / server-function use)
// instead of the SDK's process-wide token.
let user = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<Metrics__Raw.User.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/metrics/user",
    ~cookieHeader?,
    ~decode=Metrics__Raw.User.t_decode,
  )

// Public path-based variant of `user`: takes the wallet via the URL and needs
// no auth (`GET /api/metrics/user/{wallet_address}`).
let userByWallet = async (
  client: Client.t,
  ~walletAddress: string,
): result<Metrics__Raw.User.t, SdkError.t> => {
  let path = `/api/metrics/user/${encodeURIComponent(walletAddress)}`
  await Http.get(client.http, ~path, ~decode=Metrics__Raw.User.t_decode)
}
