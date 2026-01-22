# API Module Reference

REST API client for Lightcone market data and order management.

## Client Configuration

### LightconeApiClient

```python
from lightcone_sdk.api import LightconeApiClient

# Simple creation with defaults
client = LightconeApiClient("https://api.lightcone.xyz")

# With custom configuration
client = LightconeApiClient(
    "https://api.lightcone.xyz",
    timeout=60,  # seconds
    headers={"X-Custom-Header": "value"},
)

# As async context manager (recommended)
async with LightconeApiClient("https://api.lightcone.xyz") as client:
    markets = await client.get_markets()

# Access configuration
url = client.base_url
```

| Option | Default | Description |
|--------|---------|-------------|
| `base_url` | Required | API base URL |
| `timeout` | 30 seconds | Request timeout |
| `headers` | Content-Type, Accept: application/json | Custom headers |

### Retry Configuration

The client supports automatic retries with exponential backoff for transient errors.

```python
from lightcone_sdk.api import LightconeApiClient, RetryConfig

# Create client with retries enabled
client = LightconeApiClient(
    "https://api.lightcone.xyz",
    retry_config=RetryConfig.with_retries(3),  # Up to 3 retries
)

# Customize backoff behavior
retry_config = (
    RetryConfig.with_retries(5)
    .with_base_delay_ms(100)   # Initial delay
    .with_max_delay_ms(10000)  # Cap at 10 seconds
)
```

**RetryConfig Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `max_retries` | 0 (disabled) | Maximum retry attempts |
| `base_delay_ms` | 100 | Initial backoff delay in ms |
| `max_delay_ms` | 10000 | Maximum backoff delay in ms |

**Retry Behavior:**
- Retries on: `ServerError` (5xx), `RateLimitedError` (429), `HttpError`, connection errors
- Backoff formula: `base_delay_ms * 2^attempt` with 75-100% jitter, capped at `max_delay_ms`

## Endpoints

### Markets

#### get_markets

Returns all available markets.

```python
response = await client.get_markets()
# response.markets: list[Market]
# response.total: int
```

#### get_market

Returns detailed market information by pubkey.

```python
response = await client.get_market("market_pubkey")
# response.market: Market
```

#### get_market_by_slug

Returns market by URL-friendly slug.

```python
response = await client.get_market_by_slug("btc-100k-2024")
# response.market: Market
```

#### get_deposit_assets

Returns deposit assets configured for a market.

```python
response = await client.get_deposit_assets("market_pubkey")
# response.deposit_assets: list[DepositAsset]
```

### Orderbooks

#### get_orderbook

Returns orderbook depth for a specific orderbook.

```python
# Full depth
response = await client.get_orderbook("orderbook_id")

# Limited depth
response = await client.get_orderbook("orderbook_id", depth=10)
```

### Orders

#### submit_order

Submits a signed order to the matching engine.

```python
from lightcone_sdk.api import SubmitOrderRequest

response = await client.submit_order(SubmitOrderRequest(
    maker="maker_pubkey",
    nonce=1,
    market_pubkey="market_pubkey",
    base_token="base_token_mint",
    quote_token="quote_token_mint",
    side=0,  # 0=BID, 1=ASK
    maker_amount=1000000,
    taker_amount=500000,
    expiration=0,  # 0 = no expiration
    signature="hex_signature_128_chars",
    orderbook_id="orderbook_id",
))
```

#### cancel_order

Cancels a specific order by hash.

```python
response = await client.cancel_order("order_hash", "maker_pubkey")
```

#### cancel_all_orders

Cancels all orders for a user, optionally filtered by market.

```python
# Cancel all orders
response = await client.cancel_all_orders("user_pubkey")

# Cancel orders in specific market
response = await client.cancel_all_orders("user_pubkey", market_pubkey="market_pubkey")
```

### Positions

#### get_user_positions

Returns all positions for a user across all markets.

```python
response = await client.get_user_positions("user_pubkey")
# response.positions: list[Position]
```

#### get_user_market_positions

Returns user positions in a specific market.

```python
response = await client.get_user_market_positions("user_pubkey", "market_pubkey")
# response.positions: list[Position]
```

#### get_user_orders

Returns user's open orders and balances.

```python
response = await client.get_user_orders("user_pubkey")
# response.orders: list[UserOrder]
# response.balances: list[UserBalance]
```

### Trades

#### get_trades

Returns trade history with optional filters.

```python
from lightcone_sdk.api import TradesParams

response = await client.get_trades(
    TradesParams.new("orderbook_id")
        .with_user("user_pubkey")
        .with_time_range(start_ts, end_ts)
        .with_cursor(cursor)
        .with_limit(100)
)
```

**TradesParams Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `orderbook_id` | str | Yes | Orderbook identifier |
| `user_pubkey` | Optional[str] | No | Filter by user |
| `from_timestamp` | Optional[int] | No | Start timestamp (ms) |
| `to_timestamp` | Optional[int] | No | End timestamp (ms) |
| `cursor` | Optional[int] | No | Pagination cursor |
| `limit` | Optional[int] | No | Results per page (1-500) |

**Note:** Query parameters `from_timestamp` and `to_timestamp` are translated to `from` and `to` in the wire format.

### Price History

#### get_price_history

Returns historical price data (OHLCV candles).

```python
from lightcone_sdk.api import PriceHistoryParams
from lightcone_sdk.shared import Resolution

response = await client.get_price_history(
    PriceHistoryParams.new("orderbook_id")
        .with_resolution(Resolution.ONE_HOUR)
        .with_time_range(start_ts, end_ts)
        .with_cursor(cursor)
        .with_limit(100)
        .with_ohlcv()
)
```

**PriceHistoryParams Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `orderbook_id` | str | Yes | Orderbook identifier |
| `resolution` | Optional[Resolution\|str] | No | Candle interval (1m, 5m, 15m, 1h, 4h, 1d) |
| `from_timestamp` | Optional[int] | No | Start timestamp (ms) |
| `to_timestamp` | Optional[int] | No | End timestamp (ms) |
| `cursor` | Optional[int] | No | Pagination cursor |
| `limit` | Optional[int] | No | Results per page (1-1000) |
| `include_ohlcv` | Optional[bool] | No | Include OHLCV data |

### Admin

#### create_orderbook

Creates a new orderbook for a market (admin only).

```python
from lightcone_sdk.api import CreateOrderbookRequest

response = await client.create_orderbook(
    CreateOrderbookRequest(
        market_pubkey="market_pubkey",
        base_token="base_token",
        quote_token="quote_token",
        tick_size=100,
    )
)
```

#### health_check

Checks API availability.

```python
await client.health_check()
```

#### admin_health_check

Returns detailed admin health status.

```python
response = await client.admin_health_check()
```

## Request Types

### SubmitOrderRequest

| Field | Type | Description |
|-------|------|-------------|
| `maker` | str | Maker public key (base58) |
| `nonce` | int | Order nonce (must exceed user's on-chain nonce) |
| `market_pubkey` | str | Market public key |
| `base_token` | str | Base token mint |
| `quote_token` | str | Quote token mint |
| `side` | int | 0=BID, 1=ASK |
| `maker_amount` | int | Amount maker gives (raw units) |
| `taker_amount` | int | Amount maker receives (raw units) |
| `expiration` | int | Unix timestamp (0 = no expiration) |
| `signature` | str | Hex-encoded Ed25519 signature (128 chars) |
| `orderbook_id` | str | Target orderbook |

### CreateOrderbookRequest

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | str | Market public key |
| `base_token` | str | Base token mint |
| `quote_token` | str | Quote token mint |
| `tick_size` | Optional[int] | Minimum price increment |

## Response Types

### MarketsResponse

```python
@dataclass
class MarketsResponse:
    markets: list[Market]
    total: int
```

### Market

| Field | Type | Description |
|-------|------|-------------|
| `market_name` | str | Display name |
| `slug` | str | URL-friendly identifier |
| `description` | str | Market description |
| `definition` | str | Resolution criteria |
| `outcomes` | list[Outcome] | Possible outcomes |
| `banner_image_url` | Optional[str] | Banner image |
| `thumbnail_url` | Optional[str] | Thumbnail image |
| `category` | Optional[str] | Market category |
| `tags` | list[str] | Search tags |
| `featured_rank` | int | Featured ordering |
| `market_pubkey` | str | On-chain pubkey |
| `market_id` | int | Sequential ID |
| `oracle` | str | Oracle pubkey |
| `question_id` | str | Question identifier |
| `condition_id` | str | Condition hash |
| `market_status` | ApiMarketStatus | Pending/Active/Settled |
| `winning_outcome` | int | Winner (if settled) |
| `has_winning_outcome` | bool | Is settled |
| `created_at` | str | ISO timestamp |
| `activated_at` | Optional[str] | Activation time |
| `settled_at` | Optional[str] | Settlement time |
| `deposit_assets` | list[DepositAsset] | Accepted deposits |
| `orderbooks` | list[OrderbookSummary] | Associated orderbooks |

### OrderbookResponse

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | str | Market pubkey |
| `orderbook_id` | str | Orderbook identifier |
| `bids` | list[PriceLevel] | Bid levels (descending price) |
| `asks` | list[PriceLevel] | Ask levels (ascending price) |
| `best_bid` | Optional[str] | Best bid price |
| `best_ask` | Optional[str] | Best ask price |
| `spread` | Optional[str] | Bid-ask spread |
| `tick_size` | str | Minimum price increment |

### PriceLevel

| Field | Type | Description |
|-------|------|-------------|
| `price` | str | Price (decimal string) |
| `size` | str | Total size at price |
| `orders` | int | Number of orders |

### OrderResponse

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | str | Order hash (hex) |
| `status` | str | Order status |
| `remaining` | str | Remaining amount |
| `filled` | str | Filled amount |
| `fills` | list[Fill] | Immediate fills |

### Fill

| Field | Type | Description |
|-------|------|-------------|
| `counterparty` | str | Counterparty pubkey |
| `counterparty_order_hash` | str | Matched order hash |
| `fill_amount` | str | Fill amount |
| `price` | str | Fill price |
| `is_maker` | bool | Was maker side |

### CancelResponse

| Field | Type | Description |
|-------|------|-------------|
| `status` | str | Cancellation status |
| `order_hash` | str | Cancelled order hash |
| `remaining` | str | Remaining amount |

### CancelAllResponse

| Field | Type | Description |
|-------|------|-------------|
| `status` | str | Status |
| `user_pubkey` | str | User pubkey |
| `cancelled_order_hashes` | list[str] | Cancelled hashes |
| `count` | int | Orders cancelled |
| `message` | str | Status message |

### PositionsResponse

| Field | Type | Description |
|-------|------|-------------|
| `owner` | str | Owner public key |
| `total_markets` | int | Total number of markets with positions |
| `positions` | list[Position] | User positions |

### Position

| Field | Type | Description |
|-------|------|-------------|
| `id` | int | Database ID |
| `position_pubkey` | str | Position PDA |
| `owner` | str | Owner pubkey |
| `market_pubkey` | str | Market pubkey |
| `outcomes` | list[OutcomeBalance] | Balances per outcome |
| `created_at` | str | ISO timestamp |
| `updated_at` | str | ISO timestamp |

### TradesResponse

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | str | Orderbook ID |
| `trades` | list[Trade] | Trade records |
| `next_cursor` | Optional[int] | Pagination cursor |
| `has_more` | bool | More pages available |

### Trade

| Field | Type | Description |
|-------|------|-------------|
| `id` | int | Trade ID |
| `orderbook_id` | str | Orderbook ID |
| `taker_pubkey` | str | Taker pubkey |
| `maker_pubkey` | str | Maker pubkey |
| `side` | ApiTradeSide | "BID" or "ASK" |
| `size` | str | Trade size |
| `price` | str | Trade price |
| `taker_fee` | str | Taker fee |
| `maker_fee` | str | Maker fee |
| `executed_at` | int | Unix timestamp (ms) |

### ApiTradeSide

Enum representing the trade side:

| Value | Description |
|-------|-------------|
| `BID` | Buy side (taker was buying) |
| `ASK` | Sell side (taker was selling) |

### PriceHistoryResponse

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | str | Orderbook ID |
| `resolution` | str | Candle resolution |
| `include_ohlcv` | bool | OHLCV included |
| `prices` | list[PricePoint] | Price data |
| `next_cursor` | Optional[int] | Pagination cursor |
| `has_more` | bool | More pages available |

### PricePoint

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | int | Timestamp (ms) |
| `midpoint` | str | Midpoint price |
| `open` | Optional[str] | Open (if OHLCV) |
| `high` | Optional[str] | High (if OHLCV) |
| `low` | Optional[str] | Low (if OHLCV) |
| `close` | Optional[str] | Close (if OHLCV) |
| `volume` | Optional[str] | Volume (if OHLCV) |
| `best_bid` | Optional[str] | Best bid |
| `best_ask` | Optional[str] | Best ask |

### UserOrder

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | str | Order hash |
| `market_pubkey` | str | Market pubkey |
| `orderbook_id` | str | Orderbook ID |
| `side` | int | 0=BID, 1=ASK |
| `maker_amount` | str | Maker amount |
| `taker_amount` | str | Taker amount |
| `remaining` | str | Remaining size |
| `filled` | str | Filled size |
| `price` | str | Order price |
| `created_at` | str | ISO timestamp |
| `expiration` | int | Expiration timestamp |

### UserBalance

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | str | Market pubkey |
| `deposit_asset` | str | Deposit mint |
| `outcomes` | list[UserOrderOutcomeBalance] | Outcome balances |

### UserOrderOutcomeBalance

| Field | Type | Description |
|-------|------|-------------|
| `outcome_index` | int | Outcome index |
| `conditional_token` | str | Conditional mint |
| `idle` | str | Available balance |
| `on_book` | str | Locked in orders |

## Error Handling

```python
from lightcone_sdk.api import (
    ApiError,
    HttpError,
    NotFoundError,
    BadRequestError,
    UnauthorizedError,
    ForbiddenError,
    RateLimitedError,
    ConflictError,
    ServerError,
    DeserializeError,
    InvalidParameterError,
    UnexpectedStatusError,
)

try:
    market = await client.get_market("invalid")
except NotFoundError as e:
    print(f"Not found: {e.message}")
except BadRequestError as e:
    print(f"Bad request: {e.message}")
except UnauthorizedError as e:
    print(f"Unauthorized: {e.message}")
except ForbiddenError as e:
    print(f"Forbidden: {e.message}")
except RateLimitedError as e:
    print(f"Rate limited: {e.message}")
except ConflictError as e:
    print(f"Conflict: {e.message}")
except ServerError as e:
    print(f"Server error: {e.message}")
except HttpError as e:
    print(f"HTTP error: {e.message}")
except DeserializeError as e:
    print(f"Parse error: {e.message}")
except InvalidParameterError as e:
    print(f"Invalid param: {e.message}")
except UnexpectedStatusError as e:
    print(f"Unexpected status {e.status}: {e.message}")
except ApiError as e:
    print(f"API error: {e}")
```

### ApiError Variants

| Error | HTTP Status | Description |
|-------|-------------|-------------|
| `HttpError` | - | Network/connection error |
| `NotFoundError` | 404 | Resource not found |
| `BadRequestError` | 400 | Invalid parameters |
| `UnauthorizedError` | 401 | Authentication failed |
| `ForbiddenError` | 403 | Permission denied |
| `RateLimitedError` | 429 | Rate limit exceeded |
| `ConflictError` | 409 | Resource conflict |
| `ServerError` | 5xx | Server error |
| `DeserializeError` | - | JSON parsing error |
| `InvalidParameterError` | - | Client-side validation |
| `UnexpectedStatusError` | Other | Unexpected HTTP status |
