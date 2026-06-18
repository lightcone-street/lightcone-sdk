# Metrics

Platform, market, orderbook, category, and deposit-token volume metrics, plus deposit-token volume history, open-interest history, unique-trader history, the market leaderboard, and time-series history.

[← Overview](../../../README.md)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [Examples](#examples)

## Types

All monetary / percentage fields are `rust_decimal::Decimal` (deserialized from the backend's string representation via the `serde-str` feature). Pubkeys are `PubkeyStr`; orderbook IDs are `OrderBookId`. Fields map 1:1 to the backend's `dto::metrics` types.

### `PlatformMetrics` — response of `metrics().platform()`

| Field | Type | Description |
|-------|------|-------------|
| `volume_{24h,7d,30d,total}_usd` | `Decimal` | Total traded USD volume per window |
| `taker_bid_volume_{24h,7d,30d,total}_usd` | `Decimal` | Taker-bid-side USD volume |
| `taker_ask_volume_{24h,7d,30d,total}_usd` | `Decimal` | Taker-ask-side USD volume |
| `taker_bid_ask_imbalance_{24h,7d,30d,total}_pct` | `Decimal` | Bid/ask imbalance (%) |
| `open_interest_usd` | `Decimal` | Current open interest in USD |
| `fees_{24h,7d,30d}_usd` | `Decimal` | Fee totals per window. The backend currently returns `"0"` for each fee window |
| `unique_traders_{24h,7d,30d}` | `i32` | Unique trader counts |
| `active_markets` / `active_orderbooks` | `i64` | Currently active entities |
| `deposit_token_volumes` | `Vec<DepositTokenVolumeMetrics>` | Per-deposit-token breakdown |
| `updated_at` | `Option<DateTime<Utc>>` | When the snapshot was computed |

### `MarketVolumeMetrics` — entry in the `metrics().markets()` list

Summary per market: `market_pubkey: PubkeyStr`, denormalized `slug`/`market_name`/`category`, same four-window volume/imbalance/unique-trader fields as `PlatformMetrics`, plus `category_volume_share_24h_pct` and `platform_volume_share_24h_pct`.

### `MarketDetailMetrics` — response of `metrics().market(pubkey, ..)`

Same fields as `MarketVolumeMetrics`, plus three vector breakdowns:

| Field | Type | Description |
|-------|------|-------------|
| `outcome_volumes` | `Vec<OutcomeVolumeMetrics>` | Per-outcome |
| `orderbook_volumes` | `Vec<MarketOrderbookVolumeMetrics>` | Per-orderbook (USD + base + quote) |
| `deposit_token_volumes` | `Vec<DepositTokenVolumeMetrics>` | Per deposit token |

### `OrderbookVolumeMetrics` — response of `metrics().orderbook(id, ..)`

Per-orderbook totals with volume expressed in USD, base token, and quote token across all four windows. Includes `market_pubkey`, `orderbook_id`, outcome binding, and `market_volume_share_24h_pct`.

### `CategoryVolumeMetrics`, `DepositTokenVolumeMetrics`

Single-dimension summaries with the same four-window shape. See [`wire.rs`](./wire.rs) for exact fields.

### `DepositTokenVolumeHistory`

Daily platform volume history broken down by deposit token. The response has `resolution: Resolution::Day1`, inclusive `from`, exclusive `to`, `volume_total_usd`, `total_days`, a `deposit_tokens` legend sorted by total range volume, and `points`.

Each `DepositTokenVolumeHistoryPoint` has `bucket_start: DateTime<Utc>` (deserialized from Unix epoch ms), `bucket_start_date: NaiveDate`, `total_volume_usd`, `cumulative_volume_usd`, and `deposit_token_volumes` for the stacked-bar breakdown. All USD fields are `Decimal`.

### `OpenInterestHistory`

Daily platform open-interest snapshots broken down by deposit asset. The response has `resolution: Resolution::Day1`, inclusive `from`, exclusive `to`, `latest_open_interest_usd`, `total_days`, a `deposit_assets` legend sorted by latest open interest, and `points`.

Open interest is a snapshot metric, not cumulative. Do not sum values across days. Explicit zero values are preserved as `Decimal::ZERO` when an asset's open interest drops to zero.

### `UniqueTradersHistory`

Daily unique trader counts for the platform or a scoped entity. The response has `resolution: Resolution::Day1`, `scope`, `scope_key`, inclusive `from`, exclusive `to`, `latest_unique_traders`, `total_days`, and `points`.

Each `UniqueTradersHistoryPoint` has `bucket_start: DateTime<Utc>` (deserialized from Unix epoch ms for the UTC day start), `bucket_start_date: NaiveDate`, and `unique_traders`. Missing days are returned with `unique_traders: 0`.

### `CategoriesMetrics`, `DepositTokensMetrics`, `MarketsMetrics`, `Leaderboard`

Plural envelopes holding a `Vec<_>` of their single-dimension counterparts (plus `total` / `period` metadata where relevant).

### `LeaderboardEntry`

`rank: i32`, `market_pubkey: PubkeyStr`, denormalized metadata, `volume_24h_usd: Decimal`, and two share-% decimals.

### `MetricsHistory` / `HistoryPoint`

Time-series of volume buckets for a given scope + key. Each `HistoryPoint` has `bucket_start: DateTime<Utc>` (deserialized from Unix epoch ms) and `volume_usd: Decimal`.

### `UserMetrics` — response of `metrics().user()`, `metrics().user_with_cookies()`, and `metrics().user_by_wallet()`

| Field | Type | Description |
|-------|------|-------------|
| `wallet_address` | `PubkeyStr` | The wallet the metrics describe |
| `total_outcomes_traded` | `i64` | Distinct orderbooks the wallet has traded as taker or maker |
| `total_volume_usd` | `Decimal` | Sum of `usd_value` across all the wallet's trades |
| `total_referrals_used` | `i64` | Redemptions of referral codes owned by this wallet's user |

## Client Methods

Access via `client.metrics()`.

### `platform`

```rust
async fn platform(&self) -> Result<PlatformMetrics, SdkError>
```

Fetch platform-wide metrics.

### `markets`

```rust
async fn markets(&self, query: &MarketsMetricsQuery) -> Result<MarketsMetrics, SdkError>
```

List metrics for all active markets. `MarketsMetricsQuery::default()` returns everything.

### `market`

```rust
async fn market(
    &self,
    market_pubkey: &PubkeyStr,
    query: &MarketMetricsQuery,
) -> Result<MarketDetailMetrics, SdkError>
```

Detailed metrics for one market, including outcome, orderbook, and deposit-token breakdowns.

### `orderbook`

```rust
async fn orderbook(
    &self,
    orderbook_id: &OrderBookId,
    query: &OrderbookMetricsQuery,
) -> Result<OrderbookVolumeMetrics, SdkError>
```

Detailed metrics for one orderbook.

### `categories`

```rust
async fn categories(&self) -> Result<CategoriesMetrics, SdkError>
```

List metrics per category.

### `category`

```rust
async fn category(
    &self,
    category: &str,
    query: &CategoryMetricsQuery,
) -> Result<CategoryVolumeMetrics, SdkError>
```

Metrics for a single category. The `category` argument is URL-encoded.

### `deposit_tokens`

```rust
async fn deposit_tokens(&self) -> Result<DepositTokensMetrics, SdkError>
```

Per-deposit-token platform-wide volumes.

### `deposit_tokens_volume_history`

```rust
async fn deposit_tokens_volume_history(
    &self,
    query: &DepositTokenVolumeHistoryQuery,
) -> Result<DepositTokenVolumeHistory, SdkError>
```

Daily platform volume history by deposit token from `GET /api/metrics/deposit-tokens/volume-history`. `DepositTokenVolumeHistoryQuery::default()` lets the backend choose its range. Optional query params are `from` (inclusive epoch ms), `to` (exclusive epoch ms), and `limit` (backend default/max is `5000`).

### `open_interest_history`

```rust
async fn open_interest_history(
    &self,
    query: &OpenInterestHistoryQuery,
) -> Result<OpenInterestHistory, SdkError>
```

Daily platform open-interest snapshots by deposit asset from `GET /api/metrics/open-interest/history`. `OpenInterestHistoryQuery::default()` lets the backend choose its range. Optional query params are `from` (inclusive epoch ms), `to` (exclusive epoch ms), and `limit` (backend default is `5000`).

### `unique_traders_history`

```rust
async fn unique_traders_history(
    &self,
    query: &UniqueTradersHistoryQuery,
) -> Result<UniqueTradersHistory, SdkError>
```

Daily unique trader counts from `GET /api/metrics/unique-traders/history`. `UniqueTradersHistoryQuery::default()` returns platform-wide history. Optional query params are `scope`, `scope_key`, `from` (inclusive epoch ms), `to` (exclusive epoch ms), and `limit` (backend default is `5000`). For non-platform scopes, set both `scope` and `scope_key`.

### `leaderboard`

```rust
async fn leaderboard(&self, limit: Option<u32>) -> Result<Leaderboard, SdkError>
```

Top markets by 24h volume. `limit` defaults to the backend's setting (currently 20) when `None`.

### `history`

```rust
async fn history(
    &self,
    scope: &str,
    scope_key: &str,
    query: &MetricsHistoryQuery,
) -> Result<MetricsHistory, SdkError>
```

Time-series of volume buckets. `scope` is one of `"orderbook" | "market" | "category" | "deposit_token" | "platform"`. `MetricsHistoryQuery::default()` uses `Resolution::Hour1` with no time bounds.

### `user`

```rust
async fn user(&self) -> Result<UserMetrics, SdkError>
```

Per-wallet trading + referral aggregates for the **authenticated** user. Hits `GET /api/metrics/user`; the wallet is resolved server-side from the `auth_token` cookie.

### `user_with_cookies`

```rust
async fn user_with_cookies(&self, auth_token: &str) -> Result<UserMetrics, SdkError>
```

SSR / server-function variant of [`user`]. Hits the same authenticated endpoint with the supplied `auth_token` injected as `Cookie: lightcone-token=…` for that single call. Does not touch the SDK's process-wide token store; safe under concurrent SSR. See [the Authentication section](../../../README.md#authentication) for the broader `_with_cookies` story.

### `user_by_wallet`

```rust
async fn user_by_wallet(&self, wallet_address: &str) -> Result<UserMetrics, SdkError>
```

Public path-based variant. Hits `GET /api/metrics/user/{wallet_address}` and requires no auth — useful for leaderboards / "view another trader's profile" flows.

## Examples

```rust
use lightcone::prelude::*;

let client = LightconeClient::builder().build()?;

// Platform totals
let platform = client.metrics().platform().await?;
println!("24h volume: ${}", platform.volume_24h_usd);

// Market leaderboard
let board = client.metrics().leaderboard(Some(10)).await?;
for entry in &board.entries {
    println!(
        "#{} {} — ${}",
        entry.rank,
        entry.market_name.as_deref().unwrap_or("?"),
        entry.volume_24h_usd
    );
}

// Time-series history
let history = client
    .metrics()
    .history("platform", "platform", &MetricsHistoryQuery::default())
    .await?;
println!("buckets: {}", history.points.len());
```

See [`examples/metrics_all.rs`](../../../examples/metrics_all.rs) for a program that exercises every metrics endpoint.

---

[← Overview](../../../README.md)
