# WebSocket Module Reference

Real-time data streaming for Lightcone markets.

## Connection Methods

### connect

Connects to the WebSocket server.

```python
from lightcone_sdk.websocket import LightconeWebSocketClient

client = await LightconeWebSocketClient.connect("wss://ws.lightcone.xyz/ws")
```

### connect (with config)

Connects with custom configuration.

```python
client = await LightconeWebSocketClient.connect(
    url="wss://ws.lightcone.xyz/ws",
    reconnect=True,
    max_reconnect_attempts=5,
    reconnect_delay=1.0,
    max_delay=30.0,
    auth_token=None,
)
```

### connect_authenticated

Connects with Ed25519 authentication (required for user streams).

```python
from nacl.signing import SigningKey

signing_key = SigningKey(secret_key_bytes)
client = await LightconeWebSocketClient.connect_authenticated(signing_key)
```

## Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `url` | str | Required | WebSocket URL |
| `reconnect` | bool | True | Auto-reconnect on disconnect |
| `max_reconnect_attempts` | int | 5 | Maximum reconnection attempts |
| `reconnect_delay` | float | 1.0 | Initial backoff delay (seconds) |
| `max_delay` | float | 30.0 | Maximum backoff delay (seconds) |
| `auth_token` | Optional[str] | None | Pre-obtained auth token |

**Backoff Formula:** Full jitter: `delay = random(0, base_delay * 2^attempt)` capped at max_delay

## Subscription Types

### book_updates

Real-time orderbook updates with automatic delta application.

```python
await client.subscribe_book_updates([
    "market1:orderbook1",
    "market2:orderbook2",
])

# Unsubscribe
await client.unsubscribe_book_updates(["market1:orderbook1"])
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `orderbook_ids` | list[str] | List of orderbook identifiers |

### trades

Real-time trade stream.

```python
await client.subscribe_trades(["market:orderbook"])

await client.unsubscribe_trades(["market:orderbook"])
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `orderbook_ids` | list[str] | List of orderbook identifiers |

### user

User-specific events (orders, balances). Requires authentication.

```python
await client.subscribe_user("user_pubkey")

await client.unsubscribe_user("user_pubkey")
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `user` | str | User public key (base58) |

### price_history

Real-time candle updates.

```python
await client.subscribe_price_history(
    orderbook_id="orderbook_id",
    resolution="1h",
    include_ohlcv=True,
)

await client.unsubscribe_price_history("orderbook_id", "1h")
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `orderbook_id` | str | Orderbook identifier |
| `resolution` | str | "1m", "5m", "15m", "1h", "4h", "1d" |
| `include_ohlcv` | bool | Include OHLCV data |

### market

Market lifecycle events (orderbook created, settled, etc.).

```python
await client.subscribe_market("market_pubkey")

await client.unsubscribe_market("market_pubkey")
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `market_pubkey` | str | Market public key |

## WsEvent and WsEventType

Events emitted by the WebSocket client.

| Event Type | Fields | Description |
|------------|--------|-------------|
| `CONNECTED` | - | Connection established |
| `DISCONNECTED` | `reason: str` | Connection lost |
| `BOOK_UPDATE` | `orderbook_id: str, is_snapshot: bool` | Orderbook updated |
| `TRADE` | `orderbook_id: str, trade: TradeData` | Trade executed |
| `USER_UPDATE` | `event_type: str, user: str` | User event |
| `PRICE_UPDATE` | `orderbook_id: str, resolution: str` | Candle updated |
| `MARKET_EVENT` | `event_type: str, market_pubkey: str` | Market event |
| `ERROR` | `error: Exception` | Error occurred |
| `RESYNC_REQUIRED` | `orderbook_id: str` | Sequence gap detected |
| `PONG` | - | Pong received |
| `RECONNECTING` | `attempt: int` | Reconnection attempt |

### Event Loop

```python
from lightcone_sdk.websocket import WsEventType

async for event in client:
    if event.type == WsEventType.CONNECTED:
        print("Connected")

    elif event.type == WsEventType.BOOK_UPDATE:
        book = client.get_orderbook(event.orderbook_id)
        if book:
            print(f"Best bid: {book.best_bid()}")
            print(f"Spread: {book.spread()}")

    elif event.type == WsEventType.TRADE:
        print(f"{event.orderbook_id}: {event.trade.side} @ {event.trade.price} size {event.trade.size}")

    elif event.type == WsEventType.USER_UPDATE:
        state = client.get_user_state(event.user)
        if state:
            print(f"Orders: {state.order_count()}")

    elif event.type == WsEventType.PRICE_UPDATE:
        history = client.get_price_history(event.orderbook_id, event.resolution)
        if history:
            candle = history.latest_candle()
            if candle:
                print(f"Latest close: {candle.c}")

    elif event.type == WsEventType.MARKET_EVENT:
        print(f"Market {event.market_pubkey}: {event.event_type}")

    elif event.type == WsEventType.RESYNC_REQUIRED:
        print(f"Resync for {event.orderbook_id}")

    elif event.type == WsEventType.RECONNECTING:
        print(f"Reconnecting... attempt {event.attempt}")

    elif event.type == WsEventType.DISCONNECTED:
        print(f"Disconnected: {event.reason}")

    elif event.type == WsEventType.ERROR:
        print(f"Error: {event.error}")
```

### Alternative: recv()

```python
event = await client.recv()
```

## State Management

The client automatically maintains local state for subscribed streams.

### LocalOrderbook

Maintains a local copy of the orderbook with automatic delta application.

```python
book = client.get_orderbook("orderbook_id")
if book:
    # Price levels
    bids = book.get_bids()        # All bids (descending price)
    asks = book.get_asks()        # All asks (ascending price)
    top_bids = book.get_top_bids(5)  # Top 5 bids
    top_asks = book.get_top_asks(5)  # Top 5 asks

    # Best prices (returns tuple of (price, size) or None)
    best_bid = book.best_bid()
    best_ask = book.best_ask()
    mid = book.midpoint()
    spread = book.spread()

    # Depth
    total_bids = book.total_bid_depth()
    total_asks = book.total_ask_depth()
    bid_levels = book.bid_count()
    ask_levels = book.ask_count()

    # Metadata
    has_data = book.has_snapshot()
    seq = book.expected_sequence()
    ts = book.last_timestamp()
```

**LocalOrderbook Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `get_bids()` | list[PriceLevel] | All bids (descending) |
| `get_asks()` | list[PriceLevel] | All asks (ascending) |
| `get_top_bids(n)` | list[PriceLevel] | Top n bids |
| `get_top_asks(n)` | list[PriceLevel] | Top n asks |
| `best_bid()` | Optional[tuple[str, str]] | Best bid (price, size) |
| `best_ask()` | Optional[tuple[str, str]] | Best ask (price, size) |
| `midpoint()` | Optional[str] | Midpoint price |
| `spread()` | Optional[str] | Bid-ask spread |
| `bid_size_at(price)` | Optional[str] | Size at bid price |
| `ask_size_at(price)` | Optional[str] | Size at ask price |
| `total_bid_depth()` | float | Sum of all bid sizes |
| `total_ask_depth()` | float | Sum of all ask sizes |
| `bid_count()` | int | Number of bid levels |
| `ask_count()` | int | Number of ask levels |
| `has_snapshot()` | bool | Has received snapshot |
| `expected_sequence()` | int | Expected seq number |
| `last_timestamp()` | Optional[str] | Last update time |

### UserState

Maintains user's orders and balances.

```python
state = client.get_user_state("user_pubkey")
if state:
    # Orders
    all_orders = state.open_orders()
    order = state.get_order("order_hash")
    market_orders = state.orders_for_market("market_pubkey")
    book_orders = state.orders_for_orderbook("orderbook_id")

    # Balances
    balance = state.get_balance("market_pubkey", "deposit_mint")
    all_balances = state.all_balances()
    idle = state.idle_balance_for_outcome("market", "mint", 0)
    on_book = state.on_book_balance_for_outcome("market", "mint", 0)

    # Metadata
    count = state.order_count()
    has_data = state.has_snapshot()
    ts = state.last_timestamp()
```

**UserState Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `get_order(hash)` | Optional[Order] | Get order by hash |
| `open_orders()` | list[Order] | All open orders |
| `orders_for_market(pubkey)` | list[Order] | Orders for market |
| `orders_for_orderbook(id)` | list[Order] | Orders for orderbook |
| `get_balance(market, mint)` | Optional[BalanceEntry] | Get balance entry |
| `all_balances()` | list[BalanceEntry] | All balances |
| `idle_balance_for_outcome(...)` | Optional[str] | Idle balance |
| `on_book_balance_for_outcome(...)` | Optional[str] | On-book balance |
| `order_count()` | int | Number of orders |
| `has_snapshot()` | bool | Has received snapshot |
| `last_timestamp()` | Optional[str] | Last update time |

### PriceHistory

Maintains candle data for an orderbook.

```python
history = client.get_price_history("orderbook_id", "1h")
if history:
    # All candles
    candles = history.candles()  # Newest first
    recent = history.recent_candles(10)

    # Specific candles
    latest = history.latest_candle()
    oldest = history.oldest_candle()
    candle = history.get_candle(timestamp)

    # Current prices
    mid = history.current_midpoint()
    bid = history.current_best_bid()
    ask = history.current_best_ask()

    # Metadata
    count = history.candle_count()
    has_data = history.has_snapshot()
    ts = history.last_timestamp()
    server = history.server_time()
```

**PriceHistory Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `candles()` | list[Candle] | All candles (newest first) |
| `recent_candles(n)` | list[Candle] | Most recent n candles |
| `get_candle(timestamp)` | Optional[Candle] | Candle at timestamp |
| `latest_candle()` | Optional[Candle] | Most recent candle |
| `oldest_candle()` | Optional[Candle] | Oldest candle |
| `current_midpoint()` | Optional[str] | Current midpoint |
| `current_best_bid()` | Optional[str] | Current best bid |
| `current_best_ask()` | Optional[str] | Current best ask |
| `candle_count()` | int | Number of candles |
| `has_snapshot()` | bool | Has received snapshot |
| `last_timestamp()` | Optional[int] | Last candle timestamp |
| `server_time()` | Optional[int] | Server time |

## Data Types

### BookUpdateData

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | str | Orderbook identifier |
| `timestamp` | str | Update timestamp |
| `seq` | int | Sequence number |
| `bids` | list[PriceLevel] | Bid updates |
| `asks` | list[PriceLevel] | Ask updates |
| `is_snapshot` | bool | Is full snapshot |
| `resync` | bool | Resync required |
| `message` | Optional[str] | Status message |

### PriceLevel

| Field | Type | Description |
|-------|------|-------------|
| `side` | str | "bid" or "ask" |
| `price` | str | Price (decimal string) |
| `size` | str | Size (0 = remove level) |

### TradeData

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | str | Orderbook identifier |
| `price` | str | Trade price |
| `size` | str | Trade size |
| `side` | str | "buy" or "sell" |
| `timestamp` | str | Trade timestamp |
| `trade_id` | str | Unique trade ID |

### Order

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
| `created_at` | int | Creation timestamp |
| `expiration` | int | Expiration timestamp |

### BalanceEntry

| Field | Type | Description |
|-------|------|-------------|
| `market_pubkey` | str | Market pubkey |
| `deposit_mint` | str | Deposit mint |
| `outcomes` | list[OutcomeBalance] | Outcome balances |

### OutcomeBalance

| Field | Type | Description |
|-------|------|-------------|
| `outcome_index` | int | Outcome index |
| `mint` | str | Conditional mint |
| `idle` | str | Available balance |
| `on_book` | str | Locked in orders |

### Candle

| Field | Type | Description |
|-------|------|-------------|
| `t` | int | Timestamp (ms) |
| `o` | Optional[str] | Open |
| `h` | Optional[str] | High |
| `l` | Optional[str] | Low |
| `c` | Optional[str] | Close |
| `v` | Optional[str] | Volume |
| `m` | Optional[str] | Midpoint |
| `bb` | Optional[str] | Best bid |
| `ba` | Optional[str] | Best ask |

### MarketEventData

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | str | "orderbook_created", "settled", "opened", "paused" |
| `market_pubkey` | str | Market public key |
| `orderbook_id` | Optional[str] | Related orderbook |
| `timestamp` | str | Event timestamp |

## Authentication

Required for `user` subscriptions.

### Sign-in Flow

```python
from nacl.signing import SigningKey
from lightcone_sdk.websocket import authenticate, authenticate_with_secret_key

# Load signing key
signing_key = SigningKey(secret_key_bytes)

# Authenticate and get credentials
credentials = await authenticate(signing_key)
print(f"Auth token: {credentials.auth_token}")
print(f"User pubkey: {credentials.user_pubkey}")

# Connect with token
client = await LightconeWebSocketClient.connect(
    "wss://ws.lightcone.xyz/ws",
    auth_token=credentials.auth_token,
)

# Or use connect_authenticated for convenience
client = await LightconeWebSocketClient.connect_authenticated(signing_key)
```

### Authenticate with Secret Key

```python
from lightcone_sdk.websocket import authenticate_with_secret_key

credentials = await authenticate_with_secret_key(secret_key_bytes)  # 32 bytes
```

### Manual Authentication

```python
from lightcone_sdk.websocket import generate_signin_message, sign_message

# Generate sign-in message
message = generate_signin_message()

# Sign message
signature = sign_message(message, signing_key)
```

## Status & Control

### Connection Status

```python
# Connection state
connected = client.is_connected

# URL
url = client._url
```

### Control Methods

```python
# Send ping (keepalive)
await client.ping()

# Disconnect
await client.disconnect()
```

## Error Handling

```python
from lightcone_sdk.websocket import (
    WebSocketError,
    ConnectionFailedError,
    ConnectionClosedError,
    RateLimitedError,
    MessageParseError,
    SequenceGapError,
    ResyncRequiredError,
    SubscriptionFailedError,
    PingTimeoutError,
    ProtocolError,
    ServerError,
    NotConnectedError,
    AlreadyConnectedError,
    SendFailedError,
    ChannelClosedError,
    InvalidUrlError,
    OperationTimeoutError,
    AuthenticationFailedError,
    AuthRequiredError,
)

try:
    await client.subscribe_user("pubkey")
except ConnectionFailedError as e:
    print(f"Connection failed: {e.message}")
except NotConnectedError:
    print("Not connected")
except RateLimitedError:
    print("Rate limited, back off")
except AuthRequiredError:
    print("Authentication required for user stream")
except AuthenticationFailedError as e:
    print(f"Auth failed: {e.message}")
except WebSocketError as e:
    print(f"WebSocket error: {e}")
```

### WebSocketError Variants

| Error | Description |
|-------|-------------|
| `ConnectionFailedError(message)` | Failed to establish connection |
| `ConnectionClosedError(code, reason)` | Connection closed by server |
| `RateLimitedError` | Rate limit exceeded (code 1008) |
| `MessageParseError(message)` | Failed to parse message |
| `SequenceGapError(expected, received)` | Sequence number gap detected |
| `ResyncRequiredError(orderbook_id)` | Full resync needed |
| `SubscriptionFailedError(message)` | Subscription rejected |
| `PingTimeoutError` | No pong received |
| `ProtocolError(message)` | Protocol violation |
| `ServerError(code, message)` | Server-side error |
| `NotConnectedError` | Operation requires connection |
| `AlreadyConnectedError` | Already connected |
| `SendFailedError(message)` | Failed to send message |
| `ChannelClosedError` | Internal channel closed |
| `InvalidUrlError(url)` | Invalid WebSocket URL |
| `OperationTimeoutError` | Operation timed out |
| `AuthenticationFailedError(message)` | Authentication failed |
| `AuthRequiredError` | Authentication required |
