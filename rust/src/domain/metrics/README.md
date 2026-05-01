# Metrics

Platform, market, orderbook, category, and deposit-token volume metrics, plus the market leaderboard and time-series history.

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

### `CategoriesMetrics`, `DepositTokensMetrics`, `MarketsMetrics`, `Leaderboard`

Plural envelopes holding a `Vec<_>` of their single-dimension counterparts (plus `total` / `period` metadata where relevant).

### `LeaderboardEntry`

`rank: i32`, `market_pubkey: PubkeyStr`, denormalized metadata, `volume_24h_usd: Decimal`, and two share-% decimals.

### `MetricsHistory` / `HistoryPoint`

Time-series of volume buckets for a given scope + key. Each `HistoryPoint` has `bucket_start: i64` (Unix epoch ms) and `volume_usd: Decimal`.

### `UserMetrics` — response of `metrics().user()`, `metrics().user_with_auth()`, and `metrics().user_by_wallet()`

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

Time-series of volume buckets. `scope` is one of `"orderbook" | "market" | "category" | "deposit_token" | "platform"`. `MetricsHistoryQuery::default()` uses `"1h"` resolution with no time bounds.

### `user`

```rust
async fn user(&self) -> Result<UserMetrics, SdkError>
```

Per-wallet trading + referral aggregates for the **authenticated** user. Hits `GET /api/metrics/user`; the wallet is resolved server-side from the `auth_token` cookie.

### `user_with_auth`

```rust
async fn user_with_auth(&self, auth_token: &str) -> Result<UserMetrics, SdkError>
```

SSR / server-function variant of [`user`]. Hits the same authenticated endpoint with the supplied `auth_token` injected as `Cookie: auth_token=…` for that single call. Does not touch the SDK's process-wide token store; safe under concurrent SSR. See [the Authentication section](../../../README.md#authentication) for the broader `_with_auth` story.

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
