# API Module Reference

REST API client for Lightcone market data and order management.

## Client Configuration

### LightconeApiClientBuilder

```rust
use lightcone_sdk::api::LightconeApiClient;
use std::time::Duration;

let client = LightconeApiClient::builder("https://api.lightcone.xyz")
    .timeout(Duration::from_secs(60))
    .timeout_secs(60)  // Alternative
    .header("X-Custom-Header", "value")
    .build()?;
```

| Option | Default | Description |
|--------|---------|-------------|
| `base_url` | Required | API base URL |
| `timeout` | 30 seconds | Request timeout |
| `default_headers` | Content-Type, Accept: application/json | Custom headers |

### Direct Creation

```rust
// Simple creation with defaults
let client = LightconeApiClient::new("https://api.lightcone.xyz");

// Access configuration
let url = client.base_url();
```

## Endpoints

### Markets

#### get_markets

Returns all available markets.

```rust
let response = client.get_markets().await?;
// response.markets: Vec<Market>
// response.total: u32
```

#### get_market

Returns detailed market information by pubkey.

```rust
let response = client.get_market("market_pubkey").await?;
// response.market: Market
```

#### get_market_by_slug

Returns market by URL-friendly slug.

```rust
let response = client.get_market_by_slug("btc-100k-2024").await?;
// response.market: Market
```

#### get_deposit_assets

Returns deposit assets configured for a market.

```rust
let response = client.get_deposit_assets("market_pubkey").await?;
// response.deposit_assets: Vec<DepositAsset>
```

### Orderbooks

#### get_orderbook

Returns orderbook depth for a specific orderbook.

```rust
// Full depth
let response = client.get_orderbook("orderbook_id", None).await?;

// Limited depth
let response = client.get_orderbook("orderbook_id", Some(10)).await?;
```

### Orders

#### submit_order

Submits a signed order to the matching engine.

```rust
let response = client.submit_order(SubmitOrderRequest {
    maker: "maker_pubkey".to_string(),
    nonce: 1,
    market_pubkey: "market_pubkey".to_string(),
    base_token: "base_token_mint".to_string(),
    quote_token: "quote_token_mint".to_string(),
    side: 0,  // 0=BID, 1=ASK
    maker_amount: 1000000,
    taker_amount: 500000,
    expiration: 0,  // 0 = no expiration
    signature: "hex_signature_128_chars".to_string(),
    orderbook_id: "orderbook_id".to_string(),
}).await?;
```

#### cancel_order

Cancels a specific order by hash.

```rust
let response = client.cancel_order("order_hash", "maker_pubkey").await?;
```

#### cancel_all_orders

Cancels all orders for a user, optionally filtered by market.

```rust
// Cancel all orders
let response = client.cancel_all_orders("user_pubkey", None).await?;

// Cancel orders in specific market
let response = client.cancel_all_orders("user_pubkey", Some("market_pubkey")).await?;
```

### Positions

#### get_user_positions

Returns all positions for a user across all markets.

```rust
let response = client.get_user_positions("user_pubkey").await?;
// response.positions: Vec<Position>
```

#### get_user_market_positions

Returns user positions in a specific market.

```rust
let response = client.get_user_market_positions("user_pubkey", "market_pubkey").await?;
// response.positions: Vec<Position>
```

#### get_user_orders

Returns user's open orders.

```rust
let response = client.get_user_orders("user_pubkey").await?;
// response.orders: Vec<UserOrder>
```

### Trades

#### get_trades

Returns trade history with optional filters.

```rust
use lightcone_sdk::api::TradesParams;

let response = client.get_trades(
    TradesParams::new("orderbook_id")
        .with_user("user_pubkey")
        .with_time_range(start_ts, end_ts)
        .with_cursor(cursor)
        .with_limit(100)
).await?;
```

**TradesParams Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `orderbook_id` | String | ✓ | Orderbook identifier |
| `user_pubkey` | Option\<String\> | | Filter by user |
| `from` | Option\<i64\> | | Start timestamp (ms) |
| `to` | Option\<i64\> | | End timestamp (ms) |
| `cursor` | Option\<i64\> | | Pagination cursor |
| `limit` | Option\<u32\> | | Results per page (1-500) |

### Price History

#### get_price_history

Returns historical price data (OHLCV candles).

```rust
use lightcone_sdk::api::PriceHistoryParams;
use lightcone_sdk::shared::Resolution;

let response = client.get_price_history(
    PriceHistoryParams::new("orderbook_id")
        .with_resolution(Resolution::OneHour)
        .with_time_range(start_ts, end_ts)
        .with_cursor(cursor)
        .with_limit(100)
        .with_ohlcv()
).await?;
```

**PriceHistoryParams Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `orderbook_id` | String | ✓ | Orderbook identifier |
| `resolution` | Option\<Resolution\> | | Candle interval (1m, 5m, 15m, 1h, 4h, 1d) |
| `from` | Option\<i64\> | | Start timestamp (ms) |
| `to` | Option\<i64\> | | End timestamp (ms) |
| `cursor` | Option\<i64\> | | Pagination cursor |
| `limit` | Option\<u32\> | | Results per page (1-1000) |
| `include_ohlcv` | Option\<bool\> | | Include OHLCV data |

### Admin

#### create_orderbook

Creates a new orderbook for a market (admin only).

```rust
use lightcone_sdk::api::CreateOrderbookRequest;

let response = client.create_orderbook(
    CreateOrderbookRequest::new("market_pubkey", "base_token", "quote_token")
        .with_tick_size(100)
).await?;
```

#### health_check

Checks API availability.

```rust
client.health_check().await?;
```

#### admin_health_check

Returns detailed admin health status.

```rust
let response = client.admin_health_check().await?;
```

## Request Types

### SubmitOrderRequest

| Field | Type | Description |
|-------|------|-------------|
| `maker` | String | Maker public key (base58) |
| `nonce` | u64 | Order nonce (must exceed user's on-chain nonce) |
| `market_pubkey` | String | Market public key |
| `base_token` | String | Base token mint |
| `quote_token` | String | Quote token mint |
| `side` | u32 | 0=BID, 1=ASK |
| `maker_amount` | u64 | Amount maker gives (raw units) |
| `taker_amount` | u64 | Amount maker receives (raw units) |
| `expiration` | i64 | Unix timestamp (0 = no expiration) |
| `signature` | String | Hex-encoded Ed25519 signature (128 chars) |
| `orderbook_id` | String | Target orderbook |

### CreateOrderbookRequest

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | String | Market public key |
| `base_token` | String | Base token mint |
| `quote_token` | String | Quote token mint |
| `tick_size` | Option\<u32\> | Minimum price increment (default: 1000) |

## Response Types

### MarketsResponse

```rust
pub struct MarketsResponse {
    pub markets: Vec<Market>,
    pub total: u64,
}
```

### Market

| Field | Type | Description |
|-------|------|-------------|
| `market_name` | String | Display name |
| `slug` | String | URL-friendly identifier |
| `description` | String | Market description |
| `definition` | String | Resolution criteria |
| `outcomes` | Vec\<Outcome\> | Possible outcomes |
| `banner_image_url` | Option\<String\> | Banner image |
| `thumbnail_url` | Option\<String\> | Thumbnail image |
| `category` | Option\<String\> | Market category |
| `tags` | Vec\<String\> | Search tags |
| `featured_rank` | i32 | Featured ordering |
| `market_pubkey` | String | On-chain pubkey |
| `market_id` | u64 | Sequential ID |
| `oracle` | String | Oracle pubkey |
| `question_id` | String | Question identifier |
| `condition_id` | String | Condition hash |
| `market_status` | ApiMarketStatus | Pending/Active/Settled |
| `winning_outcome` | u32 | Winner (if settled) |
| `has_winning_outcome` | bool | Is settled |
| `created_at` | String | ISO timestamp |
| `activated_at` | Option\<String\> | Activation time |
| `settled_at` | Option\<String\> | Settlement time |
| `deposit_assets` | Vec\<DepositAsset\> | Accepted deposits |
| `orderbooks` | Vec\<OrderbookSummary\> | Associated orderbooks |

### Outcome

| Field | Type | Description |
|-------|------|-------------|
| `index` | u32 | Outcome index (0-based) |
| `name` | String | Outcome name |
| `thumbnail_url` | Option\<String\> | Outcome image |

### DepositAsset

| Field | Type | Description |
|-------|------|-------------|
| `display_name` | String | Human-readable name |
| `token_symbol` | String | Token symbol |
| `symbol` | String | Symbol |
| `deposit_asset` | String | Deposit mint pubkey |
| `id` | i64 | Database ID |
| `market_pubkey` | String | Parent market |
| `vault` | String | Vault pubkey |
| `num_outcomes` | u32 | Number of outcomes |
| `description` | Option\<String\> | Description |
| `icon_url` | Option\<String\> | Icon URL |
| `metadata_uri` | Option\<String\> | Metadata URI |
| `decimals` | u8 | Token decimals |
| `conditional_tokens` | Vec\<ConditionalToken\> | Conditional mints |
| `created_at` | String | ISO timestamp |

### ConditionalToken

| Field | Type | Description |
|-------|------|-------------|
| `id` | i64 | Database ID |
| `outcome_index` | u32 | Outcome index |
| `token_address` | String | Token mint address |
| `name` | Option\<String\> | Token name |
| `symbol` | Option\<String\> | Token symbol |
| `uri` | Option\<String\> | Token metadata URI |
| `display_name` | Option\<String\> | Display name for UI |
| `outcome` | Option\<String\> | Outcome name |
| `deposit_symbol` | Option\<String\> | Associated deposit symbol |
| `short_name` | Option\<String\> | Short name for display |
| `description` | Option\<String\> | Token description |
| `icon_url` | Option\<String\> | Icon URL |
| `metadata_uri` | Option\<String\> | Metadata URI |
| `decimals` | u8 | Token decimals |
| `created_at` | String | ISO timestamp |

### OrderbookResponse

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | String | Market pubkey |
| `orderbook_id` | String | Orderbook identifier |
| `bids` | Vec\<PriceLevel\> | Bid levels (descending price) |
| `asks` | Vec\<PriceLevel\> | Ask levels (ascending price) |
| `best_bid` | Option\<String\> | Best bid price |
| `best_ask` | Option\<String\> | Best ask price |
| `spread` | Option\<String\> | Bid-ask spread |
| `tick_size` | String | Minimum price increment |

### PriceLevel

| Field | Type | Description |
|-------|------|-------------|
| `price` | String | Price (decimal string) |
| `size` | String | Total size at price |
| `orders` | u32 | Number of orders |

### OrderResponse

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | String | Order hash (hex) |
| `status` | OrderStatus | Order status enum |
| `remaining` | String | Remaining amount |
| `filled` | String | Filled amount |
| `fills` | Vec\<Fill\> | Immediate fills |

### OrderStatus

```rust
pub enum OrderStatus {
    Accepted,    // Order placed on book
    PartialFill, // Partially filled, remainder on book
    Filled,      // Completely filled
    Rejected,    // Order rejected
}
```

### Fill

| Field | Type | Description |
|-------|------|-------------|
| `counterparty` | String | Counterparty pubkey |
| `counterparty_order_hash` | String | Matched order hash |
| `fill_amount` | String | Fill amount |
| `price` | String | Fill price |
| `is_maker` | bool | Was maker side |

### CancelResponse

| Field | Type | Description |
|-------|------|-------------|
| `status` | String | Cancellation status |
| `order_hash` | String | Cancelled order hash |
| `remaining` | String | Remaining amount that was cancelled |

### CancelAllResponse

| Field | Type | Description |
|-------|------|-------------|
| `status` | String | Status ("success") |
| `user_pubkey` | String | User public key |
| `market_pubkey` | Option\<String\> | Market pubkey if specified |
| `cancelled_order_hashes` | Vec\<String\> | List of cancelled order hashes |
| `count` | u64 | Count of cancelled orders |
| `message` | String | Human-readable message |

### PositionsResponse

| Field | Type | Description |
|-------|------|-------------|
| `positions` | Vec\<Position\> | User positions |

### Position

| Field | Type | Description |
|-------|------|-------------|
| `id` | i64 | Database ID |
| `position_pubkey` | String | Position PDA |
| `owner` | String | Owner pubkey |
| `market_pubkey` | String | Market pubkey |
| `outcomes` | Vec\<OutcomeBalance\> | Balances per outcome |
| `created_at` | String | ISO timestamp |
| `updated_at` | String | ISO timestamp |

### OutcomeBalance

| Field | Type | Description |
|-------|------|-------------|
| `outcome_index` | u32 | Outcome index |
| `conditional_token` | String | Conditional mint |
| `balance` | String | Total balance |
| `balance_idle` | String | Available balance |
| `balance_on_book` | String | Locked in orders |

### UserOrdersResponse

| Field | Type | Description |
|-------|------|-------------|
| `user_pubkey` | String | User public key |
| `orders` | Vec\<UserOrder\> | User's open orders |
| `balances` | Vec\<UserBalance\> | User's balances |

### UserOrder

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | String | Order hash |
| `market_pubkey` | String | Market pubkey |
| `orderbook_id` | String | Orderbook ID |
| `side` | ApiOrderSide | Order side enum |
| `maker_amount` | String | Maker amount |
| `taker_amount` | String | Taker amount |
| `remaining` | String | Remaining |
| `filled` | String | Filled |
| `price` | String | Order price |
| `created_at` | String | ISO timestamp |
| `expiration` | i64 | Expiration timestamp |

### ApiOrderSide

```rust
#[repr(u32)]
pub enum ApiOrderSide {
    Bid = 0, // Buy base token with quote token
    Ask = 1, // Sell base token for quote token
}
```

### UserBalance

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | String | Market pubkey |
| `deposit_asset` | String | Deposit asset mint |
| `outcomes` | Vec\<UserOrderOutcomeBalance\> | Outcome balances |

### UserOrderOutcomeBalance

| Field | Type | Description |
|-------|------|-------------|
| `outcome_index` | u32 | Outcome index |
| `conditional_token` | String | Conditional token address |
| `idle` | String | Available balance |
| `on_book` | String | Balance locked in orders |

### TradesResponse

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | String | Orderbook ID |
| `trades` | Vec\<Trade\> | Trade records |
| `next_cursor` | Option\<i64\> | Pagination cursor |
| `has_more` | bool | More pages available |

### Trade

| Field | Type | Description |
|-------|------|-------------|
| `id` | i64 | Trade ID |
| `orderbook_id` | String | Orderbook ID |
| `taker_pubkey` | String | Taker pubkey |
| `maker_pubkey` | String | Maker pubkey |
| `side` | ApiTradeSide | Trade side enum |
| `size` | String | Trade size |
| `price` | String | Trade price |
| `taker_fee` | String | Taker fee |
| `maker_fee` | String | Maker fee |
| `executed_at` | i64 | Unix timestamp (ms) |

### ApiTradeSide

```rust
#[serde(rename_all = "UPPERCASE")]
pub enum ApiTradeSide {
    Bid, // "BID"
    Ask, // "ASK"
}
```

### PriceHistoryResponse

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | String | Orderbook ID |
| `resolution` | String | Candle resolution |
| `include_ohlcv` | bool | OHLCV included |
| `prices` | Vec\<PricePoint\> | Price data |
| `next_cursor` | Option\<i64\> | Pagination cursor |
| `has_more` | bool | More pages available |

### PricePoint

| Field | Type | Description |
|-------|------|-------------|
| `t` | i64 | Timestamp (ms) |
| `m` | String | Midpoint price |
| `o` | Option\<String\> | Open (if OHLCV) |
| `h` | Option\<String\> | High (if OHLCV) |
| `l` | Option\<String\> | Low (if OHLCV) |
| `c` | Option\<String\> | Close (if OHLCV) |
| `v` | Option\<String\> | Volume (if OHLCV) |
| `bb` | Option\<String\> | Best bid |
| `ba` | Option\<String\> | Best ask |

## Error Handling

```rust
use lightcone_sdk::api::ApiError;

match client.get_market("invalid").await {
    Ok(market) => println!("Found: {:?}", market.market.market_name),
    Err(ApiError::NotFound(resp)) => println!("Not found: {}", resp),
    Err(ApiError::BadRequest(resp)) => println!("Bad request: {}", resp),
    Err(ApiError::Unauthorized(resp)) => println!("Unauthorized: {}", resp),
    Err(ApiError::Forbidden(resp)) => println!("Forbidden: {}", resp),
    Err(ApiError::RateLimited(resp)) => println!("Rate limited: {}", resp),
    Err(ApiError::Conflict(resp)) => println!("Conflict: {}", resp),
    Err(ApiError::ServerError(resp)) => println!("Server error: {}", resp),
    Err(ApiError::Http(e)) => println!("Network error: {}", e),
    Err(ApiError::Deserialize(msg)) => println!("Parse error: {}", msg),
    Err(ApiError::InvalidParameter(msg)) => println!("Invalid param: {}", msg),
    Err(ApiError::UnexpectedStatus(status, resp)) => {
        println!("Unexpected status {}: {}", status, resp)
    }
}
```

### ApiError Variants

| Variant | HTTP Status | Description |
|---------|-------------|-------------|
| `Http(reqwest::Error)` | - | Network/connection error |
| `NotFound(ErrorResponse)` | 404 | Resource not found |
| `BadRequest(ErrorResponse)` | 400 | Invalid parameters |
| `Unauthorized(ErrorResponse)` | 401 | Authentication required |
| `Forbidden(ErrorResponse)` | 403 | Permission denied |
| `RateLimited(ErrorResponse)` | 429 | Rate limit exceeded |
| `Conflict(ErrorResponse)` | 409 | Resource conflict |
| `ServerError(ErrorResponse)` | 5xx | Server error |
| `Deserialize(String)` | - | JSON parsing error |
| `InvalidParameter(String)` | - | Client-side validation |
| `UnexpectedStatus(u16, ErrorResponse)` | Other | Unexpected HTTP status |

### ApiResult

```rust
pub type ApiResult<T> = Result<T, ApiError>;
```
