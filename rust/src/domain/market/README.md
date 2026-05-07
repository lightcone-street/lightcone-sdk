# Markets

Fetch, search, and inspect prediction markets.

[← Overview](../../../README.md#markets)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [Examples](#examples)
- [Wire Types](#wire-types)

## Types

### `Market`

A fully validated market with all nested domain types.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Internal market ID |
| `pubkey` | `PubkeyStr` | On-chain market public key |
| `name` | `String` | Market title (e.g., "Who wins the 2024 election?") |
| `slug` | `String` | URL-friendly identifier |
| `status` | `Status` | Lifecycle status |
| `volume` | `Decimal` | Total traded volume |
| `description` | `String` | Detailed market description |
| `definition` | `String` | Resolution criteria |
| `category` | `Option<String>` | Market category |
| `tags` | `Vec<String>` | Searchable tags |
| `deposit_assets` | `Vec<DepositAsset>` | Accepted collateral tokens |
| `conditional_tokens` | `Vec<ConditionalToken>` | One per outcome |
| `outcomes` | `Vec<Outcome>` | Outcome definitions with metadata |
| `orderbook_pairs` | `Vec<OrderBookPair>` | Tradable pairs for this market |
| `orderbook_ids` | `Vec<OrderBookId>` | Convenience list of orderbook IDs |
| `token_metadata` | `HashMap<PubkeyStr, TokenMetadata>` | Metadata keyed by token mint |
| `banner_image_url_low` | `String` | Market banner image (low quality) |
| `banner_image_url_medium` | `String` | Market banner image (medium quality) |
| `banner_image_url_high` | `String` | Market banner image (high quality) |
| `icon_url_low` | `String` | Market thumbnail (low quality) |
| `icon_url_medium` | `String` | Market thumbnail (medium quality) |
| `icon_url_high` | `String` | Market thumbnail (high quality) |
| `featured_rank` | `Option<i16>` | Featured position (if featured) |
| `created_at` | `DateTime<Utc>` | Creation timestamp |
| `activated_at` | `Option<DateTime<Utc>>` | When market became active |
| `settled_at` | `Option<DateTime<Utc>>` | When market was resolved |
| `resolution` | `Option<MarketResolutionResponse>` | Canonical payout-vector settlement data |

Resolution semantics:

- `resolution == None` means the market has not settled yet.
- `MarketResolutionKind::SingleWinner` preserves winner-takes-all behavior, but payouts are still canonical.
- `MarketResolutionKind::Scalar` has no single winner; use each payout entry.
- `single_winning_outcome() == None` does not necessarily mean unresolved. It can also mean a scalar/split resolution. Use `is_resolved()` for settlement state.

Convenience helpers:

```rust
market.is_resolved();
market.single_winning_outcome();
market.has_single_winning_outcome();
```

### `Status`

Market lifecycle status.

| Variant | Description |
|---------|-------------|
| `Pending` | Market created, not yet accepting orders |
| `Active` | Open for trading |
| `Resolved` | Winning outcome determined, tokens redeemable |
| `Cancelled` | Market cancelled |

### `Outcome`

An individual outcome within a market.

| Field | Type | Description |
|-------|------|-------------|
| `index` | `i16` | Outcome index (0-based) |
| `name` | `String` | Outcome name (e.g., "Yes", "No") |
| `icon_url_low` | `String` | Outcome-specific icon (low quality) |
| `icon_url_medium` | `String` | Outcome-specific icon (medium quality) |
| `icon_url_high` | `String` | Outcome-specific icon (high quality) |

### `ConditionalToken`

An SPL token representing a bet on one outcome.

| Field | Type | Description |
|-------|------|-------------|
| `mint` | `PubkeyStr` | Token mint address |
| `outcome_index` | `i16` | Which outcome this token represents |
| `outcome` | `String` | Outcome name (e.g., "Yes") |
| `short_symbol` | `String` | Short symbol (e.g., "YES") |
| `icon_url_low` | `String` | Token icon (low quality) |
| `icon_url_medium` | `String` | Token icon (medium quality) |
| `icon_url_high` | `String` | Token icon (high quality) |

### `DepositAsset`

Collateral token accepted by the market.

| Field | Type | Description |
|-------|------|-------------|
| `mint` | `PubkeyStr` | Token mint address (e.g., USDC) |
| `display_name` | `String` | Human-readable name |
| `symbol` | `String` | Token symbol (e.g., "USDC") |
| `decimals` | `u8` | Token decimals |
| `icon_url_low` | `String` | Token icon (low quality) |
| `icon_url_medium` | `String` | Token icon (medium quality) |
| `icon_url_high` | `String` | Token icon (high quality) |

### `DepositAssetPair`

A base/quote pairing of two `DepositAsset`s. Populated on
`Market::deposit_asset_pairs` during wire→domain conversion — one entry per
unique base/quote combination across the market's orderbook pairs.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Stable identifier `"{base_pubkey}-{quote_pubkey}"` |
| `base` | `DepositAsset` | Base deposit asset |
| `quote` | `DepositAsset` | Quote deposit asset |

Use this when building UIs that let users pick which collateral pair to trade,
independently of outcome selection.

## Client Methods

Access via `client.markets()`.

### `get`

```rust
async fn get(
    &self,
    cursor: Option<i64>,
    limit: Option<u32>,
) -> Result<MarketsResult, SdkError>
```

Fetch markets with cursor-based pagination. Returns only `Active` and `Resolved` markets. Invalid markets (failing validation) are skipped and reported in `MarketsResult::validation_errors`.

**Parameters:**
- `cursor` -- pagination cursor from a previous response's `next_cursor`
- `limit` -- maximum number of markets to return

### `get_by_slug`

```rust
async fn get_by_slug(&self, slug: &str) -> Result<Market, SdkError>
```

Fetch a single market by its URL slug.

### `get_by_pubkey`

```rust
async fn get_by_pubkey(&self, pubkey: &str) -> Result<Market, SdkError>
```

Fetch a single market by its on-chain public key.

### `search`

```rust
async fn search(
    &self,
    query: &str,
    limit: Option<u32>,
) -> Result<Vec<MarketSearchResult>, SdkError>
```

Full-text search across market names, descriptions, and tags.

### `featured`

```rust
async fn featured(&self) -> Result<Vec<MarketSearchResult>, SdkError>
```

Get the current featured markets. Returns only `Active` and `Resolved` markets.

### `deposit_assets`

```rust
async fn deposit_assets(
    &self,
    market_pubkey: &PubkeyStr,
) -> Result<DepositMintsResponse, SdkError>
```

Fetch the deposit assets registered for a specific market, including each asset's conditional mints. Returns the raw wire response (`DepositMintsResponse` / `DepositAssetResponse` / `ConditionalTokenResponse`) rather than a rich type so consumers can access the full payload without re-fetching the parent market.

### `global_deposit_assets`

```rust
async fn global_deposit_assets(&self) -> Result<GlobalDepositAssetsResult, SdkError>
```

Fetch the active global deposit asset whitelist (platform-scoped, not market-bound). Assets that fail validation are skipped with their errors in `validation_errors`.

### Market Helpers

#### `derive_condition_id`

```rust
fn derive_condition_id(
    &self,
    oracle: &Pubkey,
    question_id: &[u8; 32],
    num_outcomes: u8,
) -> [u8; 32]
```

Derive the condition ID for a market from its oracle, question ID, and outcome count.

#### `get_conditional_mints`

```rust
fn get_conditional_mints(
    &self,
    market: &Pubkey,
    deposit_mint: &Pubkey,
    num_outcomes: u8,
) -> Vec<Pubkey>
```

Get all conditional mint pubkeys for a market.

## Examples

### Paginate through all markets

```rust
use lightcone::prelude::*;

async fn list_all_markets(client: &LightconeClient) -> Result<Vec<Market>, SdkError> {
    let mut all_markets = Vec::new();
    let mut cursor = None;

    loop {
        let result = client.markets().get(cursor, Some(50)).await?;
        all_markets.extend(result.markets);

        match result.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    Ok(all_markets)
}
```

### Find a specific market and inspect its orderbooks

```rust
use lightcone::prelude::*;

async fn inspect_market(client: &LightconeClient) -> Result<(), SdkError> {
    let market = client.markets().get(None, Some(1)).await?.markets.into_iter().next().unwrap();

    println!("Market: {} ({})", market.name, market.status.as_str());
    println!("Volume: {}", market.volume);

    for outcome in &market.outcomes {
        println!("  Outcome {}: {}", outcome.index, outcome.name);
    }

    for ob in &market.orderbook_pairs {
        println!(
            "  Orderbook: {} ({} vs {})",
            ob.orderbook_id,
            ob.base.symbol(),
            ob.quote.symbol()
        );
        if let Some(price) = ob.last_trade_price {
            println!("    Last trade: {}", price);
        }
    }

    println!("Deposit asset pairs:");
    for pair in &market.deposit_asset_pairs {
        println!("  - {} (base={}, quote={})", pair.id, pair.base.symbol, pair.quote.symbol);
    }

    Ok(())
}
```

## Wire Types

Raw backend response types are available in `lightcone::domain::market::wire` for consumers who need direct access to the REST response format. Domain types are converted from wire types via `TryFrom` with validation.

---

[← Overview](../../../README.md#markets)
