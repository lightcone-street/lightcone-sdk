# Markets

Fetch, search, and inspect prediction markets.

[Back to SDK root](../../README.md)

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
| `banner_image_url` | `String` | Market banner image |
| `icon_url` | `String` | Market thumbnail |
| `featured_rank` | `Option<i16>` | Featured position (if featured) |
| `created_at` | `DateTime<Utc>` | Creation timestamp |
| `activated_at` | `Option<DateTime<Utc>>` | When market became active |
| `settled_at` | `Option<DateTime<Utc>>` | When market was resolved |
| `winning_outcome` | `Option<i16>` | Winning outcome index (after resolution) |

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
| `icon_url` | `Option<String>` | Outcome-specific icon |

### `ConditionalToken`

An SPL token representing a bet on one outcome.

| Field | Type | Description |
|-------|------|-------------|
| `mint` | `PubkeyStr` | Token mint address |
| `outcome_index` | `i16` | Which outcome this token represents |
| `display_name` | `String` | Human-readable name |
| `short_name` | `String` | Abbreviated name |
| `icon_url` | `String` | Token icon |

### `DepositAsset`

Collateral token accepted by the market.

| Field | Type | Description |
|-------|------|-------------|
| `mint` | `PubkeyStr` | Token mint address (e.g., USDC) |
| `display_name` | `String` | Human-readable name |
| `symbol` | `String` | Token symbol (e.g., "USDC") |
| `decimals` | `u8` | Token decimals |
| `icon_url` | `String` | Token icon |

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
    let market = client.markets().get_by_slug("btc-above-100k").await?;

    println!("Market: {} ({})", market.name, market.status.as_str());
    println!("Volume: {}", market.volume);

    for outcome in &market.outcomes {
        println!("  Outcome {}: {}", outcome.index, outcome.name);
    }

    for ob in &market.orderbook_pairs {
        println!(
            "  Orderbook: {} ({} vs {})",
            ob.orderbook_id,
            ob.base.short_name,
            ob.quote.short_name
        );
        if let Some(price) = ob.last_trade_price {
            println!("    Last trade: {}", price);
        }
    }

    Ok(())
}
```

## Wire Types

Raw backend response types are available in `lightcone::domain::market::wire` for consumers who need direct access to the REST response format. Domain types are converted from wire types via `TryFrom` with validation.
