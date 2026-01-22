# API Module Reference

REST API client for Lightcone market data and order management.

## Client Configuration

### LightconeApiClient

```typescript
import { api } from "@lightcone/sdk";

// Simple creation with defaults
const client = new api.LightconeApiClient();

// With custom configuration
const client = new api.LightconeApiClient({
  baseUrl: "https://lightcone.xyz/api",
  timeout: 60000, // milliseconds
  headers: { "X-Custom-Header": "value" },
});

// Access configuration
const url = client.baseUrl;
```

| Option | Default | Description |
|--------|---------|-------------|
| `baseUrl` | `https://lightcone.xyz/api` | API base URL |
| `timeout` | 30000 ms | Request timeout |
| `headers` | Content-Type: application/json | Custom headers |
| `retry` | disabled | Retry configuration (see below) |

### Retry Configuration

Configure automatic retry with exponential backoff for transient failures.

```typescript
const client = new api.LightconeApiClient({
  retry: {
    maxRetries: 3,
    baseDelayMs: 100,
    maxDelayMs: 10000,
  },
});
```

| Option | Default | Description |
|--------|---------|-------------|
| `maxRetries` | 0 (disabled) | Maximum retry attempts |
| `baseDelayMs` | 100 | Initial delay in milliseconds |
| `maxDelayMs` | 10000 | Maximum delay cap in milliseconds |

Retryable errors:
- `ServerError` (5xx)
- `RateLimited` (429)
- `Http` (network errors)

## Endpoints

### Markets

#### getMarkets

Returns all available markets.

```typescript
const response = await client.getMarkets();
// response.markets: Market[]
// response.total: number
```

#### getMarket

Returns detailed market information by pubkey.

```typescript
const response = await client.getMarket("market_pubkey");
// response.market: Market
```

#### getMarketBySlug

Returns market by URL-friendly slug.

```typescript
const response = await client.getMarketBySlug("btc-100k-2024");
// response.market: Market
```

#### getDepositAssets

Returns deposit assets configured for a market.

```typescript
const response = await client.getDepositAssets("market_pubkey");
// response.deposit_assets: DepositAsset[]
```

### Orderbooks

#### getOrderbook

Returns orderbook depth for a specific orderbook.

```typescript
// Full depth
const response = await client.getOrderbook("orderbook_id");

// Limited depth
const response = await client.getOrderbook("orderbook_id", 10);
```

### Orders

#### submitOrder

Submits a signed order to the matching engine.

```typescript
const response = await client.submitOrder({
  maker: "maker_pubkey",
  nonce: 1,
  market_pubkey: "market_pubkey",
  base_token: "base_token_mint",
  quote_token: "quote_token_mint",
  side: 0, // 0=BID, 1=ASK
  maker_amount: "1000000",
  taker_amount: "500000",
  expiration: 0, // 0 = no expiration
  signature: "hex_signature_128_chars",
  orderbook_id: "orderbook_id",
});
```

#### cancelOrder

Cancels a specific order by hash.

```typescript
const response = await client.cancelOrder("order_hash", "maker_pubkey");
```

#### cancelAllOrders

Cancels all orders for a user, optionally filtered by market.

```typescript
// Cancel all orders
const response = await client.cancelAllOrders("user_pubkey");

// Cancel orders in specific market
const response = await client.cancelAllOrders("user_pubkey", "market_pubkey");
```

### Positions

#### getUserPositions

Returns all positions for a user across all markets.

```typescript
const response = await client.getUserPositions("user_pubkey");
// response.positions: Position[]
```

#### getUserMarketPositions

Returns user positions in a specific market.

```typescript
const response = await client.getUserMarketPositions("user_pubkey", "market_pubkey");
// response.positions: Position[]
```

#### getUserOrders

Returns user's open orders and balances.

```typescript
const response = await client.getUserOrders("user_pubkey");
// response.user_pubkey: string
// response.orders: UserOrder[]
// response.balances: UserBalance[]
```

### Trades

#### getTrades

Returns trade history with optional filters.

```typescript
import { api } from "@lightcone/sdk";

const response = await client.getTrades({
  orderbook_id: "orderbook_id",
  user_pubkey: "user_pubkey", // optional
  from: startTimestamp,       // optional
  to: endTimestamp,           // optional
  cursor: cursor,             // optional
  limit: 100,                 // optional
});
```

**TradesParams Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `orderbook_id` | string | Yes | Orderbook identifier |
| `user_pubkey` | string | No | Filter by user |
| `from` | number | No | Start timestamp (ms) |
| `to` | number | No | End timestamp (ms) |
| `cursor` | number | No | Pagination cursor |
| `limit` | number | No | Results per page (1-500) |

### Price History

#### getPriceHistory

Returns historical price data (OHLCV candles).

```typescript
import { api } from "@lightcone/sdk";
import { Resolution } from "@lightcone/sdk";

const response = await client.getPriceHistory({
  orderbook_id: "orderbook_id",
  resolution: Resolution.OneHour,
  from: startTimestamp,
  to: endTimestamp,
  cursor: cursor,
  limit: 100,
  include_ohlcv: true,
});
```

**PriceHistoryParams Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `orderbook_id` | string | Yes | Orderbook identifier |
| `resolution` | Resolution | No | Candle interval (1m, 5m, 15m, 1h, 4h, 1d) |
| `from` | number | No | Start timestamp (ms) |
| `to` | number | No | End timestamp (ms) |
| `cursor` | number | No | Pagination cursor |
| `limit` | number | No | Results per page (1-500) |
| `include_ohlcv` | boolean | No | Include OHLCV data |

### Admin

#### createOrderbook

Creates a new orderbook for a market (admin only).

```typescript
const response = await client.createOrderbook({
  market_pubkey: "market_pubkey",
  base_token: "base_token",
  quote_token: "quote_token",
  tick_size: 100,
});
```

#### healthCheck

Checks API availability.

```typescript
await client.healthCheck();
```

#### adminHealthCheck

Returns detailed admin health status.

```typescript
const response = await client.adminHealthCheck();
```

## Request Types

### SubmitOrderRequest

| Field | Type | Description |
|-------|------|-------------|
| `maker` | string | Maker public key (base58) |
| `nonce` | number | Order nonce (must exceed user's on-chain nonce) |
| `market_pubkey` | string | Market public key |
| `base_token` | string | Base token mint |
| `quote_token` | string | Quote token mint |
| `side` | number | 0=BID, 1=ASK |
| `maker_amount` | string | Amount maker gives (decimal string) |
| `taker_amount` | string | Amount maker receives (decimal string) |
| `expiration` | number | Unix timestamp (0 = no expiration) |
| `signature` | string | Hex-encoded Ed25519 signature (128 chars) |
| `orderbook_id` | string | Target orderbook |

### CreateOrderbookRequest

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | string | Market public key |
| `base_token` | string | Base token mint |
| `quote_token` | string | Quote token mint |
| `tick_size` | number | Minimum price increment (optional) |

## Response Types

### MarketsResponse

```typescript
interface MarketsResponse {
  markets: Market[];
  total: number;
}
```

### Market

| Field | Type | Description |
|-------|------|-------------|
| `market_name` | string | Display name |
| `slug` | string | URL-friendly identifier |
| `description` | string | Market description |
| `definition` | string | Resolution criteria |
| `outcomes` | Outcome[] | Possible outcomes |
| `banner_image_url` | string? | Banner image |
| `thumbnail_url` | string? | Thumbnail image |
| `category` | string? | Market category |
| `tags` | string[] | Search tags |
| `featured_rank` | number | Featured ordering |
| `market_pubkey` | string | On-chain pubkey |
| `market_id` | number | Sequential ID |
| `oracle` | string | Oracle pubkey |
| `question_id` | string | Question identifier |
| `condition_id` | string | Condition hash |
| `market_status` | ApiMarketStatus | Pending/Active/Settled |
| `winning_outcome` | number | Winner (if settled) |
| `has_winning_outcome` | boolean | Is settled |
| `created_at` | string | ISO timestamp |
| `activated_at` | string? | Activation time |
| `settled_at` | string? | Settlement time |
| `deposit_assets` | DepositAsset[] | Accepted deposits |
| `orderbooks` | OrderbookSummary[] | Associated orderbooks |

### OrderbookResponse

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | string | Market pubkey |
| `orderbook_id` | string | Orderbook identifier |
| `bids` | PriceLevel[] | Bid levels (descending price) |
| `asks` | PriceLevel[] | Ask levels (ascending price) |
| `best_bid` | string? | Best bid price |
| `best_ask` | string? | Best ask price |
| `spread` | string? | Bid-ask spread |
| `tick_size` | string | Minimum price increment |

### PriceLevel

| Field | Type | Description |
|-------|------|-------------|
| `price` | string | Price (decimal string) |
| `size` | string | Total size at price |
| `orders` | number | Number of orders |

### OrderResponse

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | string | Order hash (hex) |
| `status` | string | Order status |
| `remaining` | string | Remaining amount |
| `filled` | string | Filled amount |
| `fills` | Fill[] | Immediate fills |

### Fill

| Field | Type | Description |
|-------|------|-------------|
| `counterparty` | string | Counterparty pubkey |
| `counterparty_order_hash` | string | Matched order hash |
| `fill_amount` | string | Fill amount |
| `price` | string | Fill price |
| `is_maker` | boolean | Was maker side |

### CancelResponse

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | Cancellation status |
| `order_hash` | string | Cancelled order hash |
| `remaining` | string | Remaining amount |

### CancelAllResponse

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | Status |
| `user_pubkey` | string | User pubkey |
| `market_pubkey` | string? | Market filter (if specified) |
| `cancelled_order_hashes` | string[] | Cancelled hashes |
| `count` | number | Orders cancelled |
| `message` | string | Status message |

### PositionsResponse

| Field | Type | Description |
|-------|------|-------------|
| `owner` | string | Position owner pubkey |
| `total_markets` | number | Total markets with positions |
| `positions` | Position[] | User positions |

### Position

| Field | Type | Description |
|-------|------|-------------|
| `id` | number | Database ID |
| `position_pubkey` | string | Position PDA |
| `owner` | string | Owner pubkey |
| `market_pubkey` | string | Market pubkey |
| `outcomes` | OutcomeBalance[] | Balances per outcome |
| `created_at` | string | ISO timestamp |
| `updated_at` | string | ISO timestamp |

### MarketPositionsResponse

| Field | Type | Description |
|-------|------|-------------|
| `owner` | string | Position owner pubkey |
| `market_pubkey` | string | Market pubkey |
| `positions` | Position[] | Positions in this market |

### TradesResponse

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | string | Orderbook ID |
| `trades` | Trade[] | Trade records |
| `next_cursor` | number? | Pagination cursor |
| `has_more` | boolean | More pages available |

### Trade

| Field | Type | Description |
|-------|------|-------------|
| `id` | number | Trade ID |
| `orderbook_id` | string | Orderbook ID |
| `taker_pubkey` | string | Taker pubkey |
| `maker_pubkey` | string | Maker pubkey |
| `side` | TradeSide | "BID" or "ASK" |
| `size` | string | Trade size |
| `price` | string | Trade price |
| `taker_fee` | string | Taker fee |
| `maker_fee` | string | Maker fee |
| `executed_at` | number | Unix timestamp (ms) |

### UserOrdersResponse

| Field | Type | Description |
|-------|------|-------------|
| `user_pubkey` | string | User's public key |
| `orders` | UserOrder[] | Open orders |
| `balances` | UserBalance[] | User balances |

### PriceHistoryResponse

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | string | Orderbook ID |
| `resolution` | string | Candle resolution |
| `include_ohlcv` | boolean | OHLCV included |
| `prices` | PricePoint[] | Price data |
| `next_cursor` | number? | Pagination cursor |
| `has_more` | boolean | More pages available |

### PricePoint

| Field | Type | Description |
|-------|------|-------------|
| `t` | number | Timestamp (ms) |
| `m` | string | Midpoint price |
| `o` | string? | Open (if OHLCV) |
| `h` | string? | High (if OHLCV) |
| `l` | string? | Low (if OHLCV) |
| `c` | string? | Close (if OHLCV) |
| `v` | string? | Volume (if OHLCV) |
| `bb` | string? | Best bid |
| `ba` | string? | Best ask |

## Error Handling

```typescript
import { api } from "@lightcone/sdk";

try {
  const market = await client.getMarket("invalid");
} catch (error) {
  if (error instanceof api.ApiError) {
    switch (error.variant) {
      case "NotFound":
        console.log(`Not found: ${error.message}`);
        break;
      case "BadRequest":
        console.log(`Bad request: ${error.message}`);
        break;
      case "Unauthorized":
        console.log(`Unauthorized: ${error.message}`);
        break;
      case "Forbidden":
        console.log(`Forbidden: ${error.message}`);
        break;
      case "Conflict":
        console.log(`Conflict: ${error.message}`);
        break;
      case "RateLimited":
        console.log(`Rate limited: ${error.message}`);
        break;
      case "ServerError":
        console.log(`Server error: ${error.message}`);
        break;
      case "Http":
        console.log(`HTTP error: ${error.message}`);
        break;
      case "Deserialize":
        console.log(`Parse error: ${error.message}`);
        break;
      case "InvalidParameter":
        console.log(`Invalid param: ${error.message}`);
        break;
      case "UnexpectedStatus":
        console.log(`Unexpected status ${error.statusCode}: ${error.message}`);
        break;
    }
  }
}
```

### ApiError Variants

| Variant | HTTP Status | Description |
|---------|-------------|-------------|
| `Http` | - | Network/connection error |
| `Unauthorized` | 401 | Invalid or missing authentication |
| `NotFound` | 404 | Resource not found |
| `BadRequest` | 400 | Invalid parameters |
| `Forbidden` | 403 | Permission denied |
| `Conflict` | 409 | Resource conflict |
| `RateLimited` | 429 | Rate limit exceeded |
| `ServerError` | 5xx | Server error |
| `Deserialize` | - | JSON parsing error |
| `InvalidParameter` | - | Client-side validation |
| `UnexpectedStatus` | Other | Unexpected HTTP status |
