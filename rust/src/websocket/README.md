# WebSocket Module Reference

Real-time data streaming for Lightcone prediction markets.

## Connection Methods

### connect_default

Connects to the default WebSocket endpoint.

```rust
use lightcone_sdk::websocket::*;

let mut client = LightconeWebSocketClient::connect_default().await?;
// Connects to wss://ws.lightcone.xyz/ws
```

### connect

Connects to a custom WebSocket URL.

```rust
let mut client = LightconeWebSocketClient::connect("wss://custom.url/ws").await?;
```

### connect_with_config

Connects with custom configuration.

```rust
let config = WebSocketConfig {
    reconnect_attempts: 5,
    base_delay_ms: 500,
    max_delay_ms: 15000,
    ping_interval_secs: 30,
    auto_reconnect: true,
    auto_resubscribe: true,
    auth_token: None,
};
let mut client = LightconeWebSocketClient::connect_with_config("wss://...", config).await?;
```

### connect_authenticated

Connects with Ed25519 authentication (required for user streams).

```rust
use ed25519_dalek::SigningKey;

let signing_key = SigningKey::from_bytes(&secret_key_bytes);
let mut client = LightconeWebSocketClient::connect_authenticated(&signing_key).await?;
```

### connect_authenticated_with_config

Connects with authentication and custom configuration.

```rust
let mut client = LightconeWebSocketClient::connect_authenticated_with_config(
    &signing_key,
    config
).await?;
```

### connect_with_auth

Connects with a pre-obtained auth token.

```rust
let mut client = LightconeWebSocketClient::connect_with_auth(
    "wss://ws.lightcone.xyz/ws",
    auth_token
).await?;
```

## WebSocketConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `reconnect_attempts` | u32 | 10 | Maximum reconnection attempts |
| `base_delay_ms` | u64 | 1000 | Initial backoff delay (ms) |
| `max_delay_ms` | u64 | 30000 | Maximum backoff delay (ms) |
| `ping_interval_secs` | u64 | 30 | Keepalive ping interval (seconds) |
| `auto_reconnect` | bool | true | Auto-reconnect on disconnect |
| `auto_resubscribe` | bool | true | Re-subscribe after reconnect |
| `auth_token` | Option\<String\> | None | Pre-obtained auth token |

**Backoff Formula:** `delay = min(base_delay * 2^(attempt-1), max_delay)`

## Subscription Types

### book_updates

Real-time orderbook updates with automatic delta application.

```rust
client.subscribe_book_updates(vec![
    "market1:orderbook1".to_string(),
    "market2:orderbook2".to_string(),
]).await?;

// Unsubscribe
client.unsubscribe_book_updates(vec!["market1:orderbook1".to_string()]).await?;
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `orderbook_ids` | Vec\<String\> | List of orderbook identifiers |

### trades

Real-time trade stream.

```rust
client.subscribe_trades(vec!["market:orderbook".to_string()]).await?;

client.unsubscribe_trades(vec!["market:orderbook".to_string()]).await?;
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `orderbook_ids` | Vec\<String\> | List of orderbook identifiers |

### user

User-specific events (orders, balances). Requires authentication.

```rust
client.subscribe_user("user_pubkey".to_string()).await?;

client.unsubscribe_user("user_pubkey".to_string()).await?;
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `user` | String | User public key (base58) |

### price_history

Real-time candle updates.

```rust
client.subscribe_price_history(
    "orderbook_id".to_string(),
    "1h".to_string(),  // Resolution
    true,              // include_ohlcv
).await?;

client.unsubscribe_price_history("orderbook_id".to_string(), "1h".to_string()).await?;
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `orderbook_id` | String | Orderbook identifier |
| `resolution` | String | "1m", "5m", "15m", "1h", "4h", "1d" |
| `include_ohlcv` | bool | Include OHLCV data |

### market

Market lifecycle events (orderbook created, settled, etc.).

```rust
client.subscribe_market("market_pubkey".to_string()).await?;

client.unsubscribe_market("market_pubkey".to_string()).await?;
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `market_pubkey` | String | Market public key |

## WsEvent Variants

Events emitted by the WebSocket stream.

| Variant | Fields | Description |
|---------|--------|-------------|
| `Connected` | - | Connection established |
| `Disconnected` | `reason: String` | Connection lost |
| `BookUpdate` | `orderbook_id: String, is_snapshot: bool` | Orderbook updated |
| `Trade` | `orderbook_id: String, trade: TradeData` | Trade executed |
| `UserUpdate` | `event_type: String, user: String` | User event |
| `PriceUpdate` | `orderbook_id: String, resolution: String` | Candle updated |
| `MarketEvent` | `event_type: String, market_pubkey: String` | Market event |
| `Error` | `error: WebSocketError` | Error occurred |
| `ResyncRequired` | `orderbook_id: String` | Sequence gap detected |
| `Pong` | - | Pong received |
| `Reconnecting` | `attempt: u32` | Reconnection attempt |

### Event Loop

```rust
use futures_util::StreamExt;

while let Some(event) = client.next().await {
    match event {
        WsEvent::Connected => {
            println!("Connected");
        }
        WsEvent::BookUpdate { orderbook_id, is_snapshot } => {
            if let Some(book) = client.get_orderbook(&orderbook_id) {
                println!("Best bid: {:?}", book.best_bid());
                println!("Spread: {:?}", book.spread());
            }
        }
        WsEvent::Trade { orderbook_id, trade } => {
            println!("{}: {} @ {} size {}",
                orderbook_id, trade.side, trade.price, trade.size);
        }
        WsEvent::UserUpdate { event_type, user } => {
            if let Some(state) = client.get_user_state(&user) {
                println!("Orders: {}", state.orders.len());
            }
        }
        WsEvent::PriceUpdate { orderbook_id, resolution } => {
            if let Some(history) = client.get_price_history(&orderbook_id, &resolution) {
                if let Some(candle) = history.last_candle() {
                    println!("Latest close: {:?}", candle.c);
                }
            }
        }
        WsEvent::MarketEvent { event_type, market_pubkey } => {
            println!("Market {}: {}", market_pubkey, event_type);
        }
        WsEvent::ResyncRequired { orderbook_id } => {
            // Client auto-resubscribes by default
            println!("Resync for {}", orderbook_id);
        }
        WsEvent::Reconnecting { attempt } => {
            println!("Reconnecting... attempt {}", attempt);
        }
        WsEvent::Disconnected { reason } => {
            println!("Disconnected: {}", reason);
        }
        WsEvent::Error { error } => {
            eprintln!("Error: {:?}", error);
        }
        WsEvent::Pong => {}
    }
}
```

## State Management

The client automatically maintains local state for subscribed streams.

### LocalOrderbook

Maintains a local copy of the orderbook with automatic delta application.

```rust
if let Some(book) = client.get_orderbook("orderbook_id") {
    // Price levels
    let bids = book.get_bids();       // All bids (descending price)
    let asks = book.get_asks();       // All asks (ascending price)
    let top_bids = book.get_top_bids(5);  // Top 5 bids
    let top_asks = book.get_top_asks(5);  // Top 5 asks

    // Best prices
    let best_bid = book.best_bid();   // Option<String>
    let best_ask = book.best_ask();   // Option<String>
    let mid = book.mid_price();       // Option<String>
    let spread = book.spread();       // Option<String>

    // Metadata
    let id = &book.orderbook_id;
    let has_data = book.has_snapshot;
}

// Async version (waits for lock)
let book = client.get_orderbook_async("orderbook_id").await;
```

**LocalOrderbook Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | String | Orderbook identifier |
| `bids` | BTreeMap\<String, String\> | Price → Size (descending) |
| `asks` | BTreeMap\<String, String\> | Price → Size (ascending) |
| `expected_seq` | u64 | Expected sequence number |
| `has_snapshot` | bool | Has received initial snapshot |
| `last_timestamp` | Option\<String\> | Last update timestamp |

**LocalOrderbook Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(orderbook_id)` | Self | Create new instance |
| `apply_snapshot(update)` | () | Apply full snapshot |
| `apply_delta(update)` | Result\<(), WebSocketError\> | Apply incremental update |
| `apply_update(update)` | Result\<(), WebSocketError\> | Apply any update type |
| `get_bids()` | Vec\<PriceLevel\> | All bids |
| `get_asks()` | Vec\<PriceLevel\> | All asks |
| `get_top_bids(n)` | Vec\<PriceLevel\> | Top n bids |
| `get_top_asks(n)` | Vec\<PriceLevel\> | Top n asks |
| `best_bid()` | Option\<String\> | Best bid price |
| `best_ask()` | Option\<String\> | Best ask price |
| `mid_price()` | Option\<String\> | Midpoint price |
| `spread()` | Option\<String\> | Bid-ask spread |

### UserState

Maintains user's orders and balances.

```rust
if let Some(state) = client.get_user_state("user_pubkey") {
    // Orders
    for (hash, order) in &state.orders {
        println!("{}: {} remaining", hash, order.remaining);
    }
    let order = state.get_order("order_hash");

    // Balances
    let balance = state.get_balance("market_pubkey", "deposit_mint");
    let total = state.total_balance("market_pubkey", "deposit_mint", 0);
}

// Async version
let state = client.get_user_state_async("user_pubkey").await;
```

**UserState Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `user` | String | User public key |
| `orders` | HashMap\<String, Order\> | Order hash → Order |
| `balances` | HashMap\<String, BalanceEntry\> | Key → Balance |
| `has_snapshot` | bool | Has received initial snapshot |
| `last_timestamp` | Option\<String\> | Last update timestamp |

**UserState Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(user)` | Self | Create new instance |
| `apply_snapshot(data)` | () | Apply full snapshot |
| `apply_order_update(data)` | () | Apply order update |
| `apply_balance_update(data)` | () | Apply balance update |
| `apply_event(data)` | () | Apply any event type |
| `get_order(hash)` | Option\<&Order\> | Get order by hash |
| `get_balance(market, mint)` | Option\<&BalanceEntry\> | Get balance entry |
| `total_balance(market, mint, outcome)` | String | Total balance for outcome |

### PriceHistory

Maintains candle data for an orderbook.

```rust
if let Some(history) = client.get_price_history("orderbook_id", "1h") {
    // All candles
    for candle in history.candles() {
        println!("t={} c={:?}", candle.t, candle.c);
    }

    // Latest candle
    if let Some(candle) = history.last_candle() {
        println!("Latest: {:?}", candle);
    }

    // Specific timestamp
    let candle = history.get_candle(timestamp);
}

// Async version
let history = client.get_price_history_async("orderbook_id", "1h").await;
```

**PriceHistory Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | String | Orderbook identifier |
| `resolution` | String | Candle resolution |
| `include_ohlcv` | bool | OHLCV enabled |
| `candles` | Vec\<Candle\> | Candle data |
| `candle_index` | HashMap\<i64, usize\> | Timestamp → Index |
| `last_timestamp` | Option\<i64\> | Last candle timestamp |
| `server_time` | Option\<i64\> | Server time reference |
| `has_snapshot` | bool | Has received snapshot |

**PriceHistory Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(orderbook_id, resolution, include_ohlcv)` | Self | Create new instance |
| `apply_snapshot(data)` | () | Apply full snapshot |
| `apply_update(data)` | () | Apply candle update |
| `apply_heartbeat(data)` | () | Apply heartbeat |
| `apply_event(data)` | () | Apply any event type |
| `candles()` | &[Candle] | All candles |
| `last_candle()` | Option\<&Candle\> | Most recent candle |
| `get_candle(timestamp)` | Option\<&Candle\> | Candle at timestamp |

## Data Types

### BookUpdateData

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | String | Orderbook identifier |
| `timestamp` | String | Update timestamp |
| `seq` | u64 | Sequence number |
| `bids` | Vec\<PriceLevel\> | Bid updates |
| `asks` | Vec\<PriceLevel\> | Ask updates |
| `is_snapshot` | bool | Is full snapshot |
| `resync` | bool | Resync required |
| `message` | Option\<String\> | Status message |

### PriceLevel

| Field | Type | Description |
|-------|------|-------------|
| `price` | String | Price (decimal string) |
| `size` | String | Size (0 = remove level) |

### TradeData

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | String | Orderbook identifier |
| `price` | String | Trade price |
| `size` | String | Trade size |
| `side` | String | "buy" or "sell" |
| `timestamp` | String | Trade timestamp |
| `trade_id` | String | Unique trade ID |

### UserEventData

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | String | "snapshot", "order_update", "balance_update" |
| `orders` | Vec\<Order\> | Orders (snapshot) |
| `balances` | HashMap\<String, BalanceEntry\> | Balances (snapshot) |
| `order` | Option\<OrderUpdate\> | Single order update |
| `balance` | Option\<Balance\> | Single balance update |
| `market_pubkey` | Option\<String\> | Market context |
| `orderbook_id` | Option\<String\> | Orderbook context |
| `deposit_mint` | Option\<String\> | Deposit mint context |
| `timestamp` | Option\<String\> | Event timestamp |

### Order

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | String | Order hash |
| `side` | String | "bid" or "ask" |
| `price` | String | Order price |
| `size` | String | Original size |
| `remaining` | String | Remaining size |
| `filled` | String | Filled size |
| `status` | String | Order status |
| `created_at` | String | Creation timestamp |

### BalanceEntry

| Field | Type | Description |
|-------|------|-------------|
| `outcome_index` | u32 | Outcome index |
| `conditional_token` | String | Conditional mint |
| `balance` | String | Total balance |
| `balance_idle` | String | Available |
| `balance_on_book` | String | Locked in orders |

### PriceHistoryData

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | String | "snapshot", "update", "heartbeat" |
| `orderbook_id` | Option\<String\> | Orderbook identifier |
| `resolution` | Option\<String\> | Resolution |
| `include_ohlcv` | Option\<bool\> | OHLCV enabled |
| `prices` | Vec\<Candle\> | Candle data |
| `last_timestamp` | Option\<i64\> | Last timestamp |
| `server_time` | Option\<i64\> | Server time |
| `last_processed` | Option\<i64\> | Last processed |

### Candle

| Field | Type | Description |
|-------|------|-------------|
| `t` | i64 | Timestamp (ms) |
| `o` | Option\<String\> | Open |
| `h` | Option\<String\> | High |
| `l` | Option\<String\> | Low |
| `c` | Option\<String\> | Close |
| `v` | Option\<String\> | Volume |
| `m` | Option\<String\> | Midpoint |
| `bb` | Option\<String\> | Best bid |
| `ba` | Option\<String\> | Best ask |

### MarketEventData

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | String | "orderbook_created", "settled", "opened", "paused" |
| `market_pubkey` | String | Market public key |
| `orderbook_id` | Option\<String\> | Related orderbook |
| `timestamp` | String | Event timestamp |

## Authentication

Required for `user` subscriptions.

### Sign-in Flow

```rust
use ed25519_dalek::SigningKey;

// Load signing key
let signing_key = SigningKey::from_bytes(&secret_key_bytes);

// Connect with automatic authentication
let mut client = LightconeWebSocketClient::connect_authenticated(&signing_key).await?;

// Check authentication status
if client.is_authenticated() {
    let pubkey = client.user_pubkey();
    println!("Authenticated as: {:?}", pubkey);
}

// Access credentials
let creds = client.auth_credentials();
```

### Manual Authentication

```rust
use lightcone_sdk::websocket::{generate_signin_message, authenticate};

// Generate sign-in message
let message = generate_signin_message();

// Sign message
let signature = signing_key.sign(message.as_bytes());

// Get auth token from API
let auth_token = authenticate(&signing_key).await?;

// Connect with token
let client = LightconeWebSocketClient::connect_with_auth(url, auth_token).await?;
```

## Status & Control

### Connection Status

```rust
// Connection state
let state = client.connection_state();  // ConnectionState enum
let connected = client.is_connected();  // bool
let authenticated = client.is_authenticated();  // bool

// Configuration
let url = client.url();
let config = client.config();
let pubkey = client.user_pubkey();  // Option<&str>
```

### ConnectionState

```rust
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Disconnecting,
}
```

### Control Methods

```rust
// Send ping (keepalive)
client.ping().await?;

// Disconnect
client.disconnect().await?;
```

## Error Handling

```rust
use lightcone_sdk::websocket::WebSocketError;

match result {
    Err(WebSocketError::ConnectionFailed(msg)) => {
        println!("Connection failed: {}", msg);
    }
    Err(WebSocketError::ConnectionClosed { code, reason }) => {
        println!("Closed with code {}: {}", code, reason);
    }
    Err(WebSocketError::RateLimited) => {
        println!("Rate limited, back off");
    }
    Err(WebSocketError::MessageParseError(msg)) => {
        println!("Parse error: {}", msg);
    }
    Err(WebSocketError::SequenceGap { expected, received }) => {
        println!("Sequence gap: expected {}, got {}", expected, received);
    }
    Err(WebSocketError::ResyncRequired { orderbook_id }) => {
        println!("Resync needed for {}", orderbook_id);
    }
    Err(WebSocketError::SubscriptionFailed(msg)) => {
        println!("Subscription failed: {}", msg);
    }
    Err(WebSocketError::PingTimeout) => {
        println!("Ping timeout");
    }
    Err(WebSocketError::Protocol(msg)) => {
        println!("Protocol error: {}", msg);
    }
    Err(WebSocketError::ServerError { code, message }) => {
        println!("Server error {}: {}", code, message);
    }
    Err(WebSocketError::NotConnected) => {
        println!("Not connected");
    }
    Err(WebSocketError::AlreadyConnected) => {
        println!("Already connected");
    }
    Err(WebSocketError::SendFailed(msg)) => {
        println!("Send failed: {}", msg);
    }
    Err(WebSocketError::ChannelClosed) => {
        println!("Channel closed");
    }
    Err(WebSocketError::InvalidUrl(msg)) => {
        println!("Invalid URL: {}", msg);
    }
    Err(WebSocketError::Timeout) => {
        println!("Operation timeout");
    }
    Err(WebSocketError::Io(msg)) => {
        println!("IO error: {}", msg);
    }
    Err(WebSocketError::AuthenticationFailed(msg)) => {
        println!("Auth failed: {}", msg);
    }
    Err(WebSocketError::AuthRequired) => {
        println!("Authentication required");
    }
    Err(WebSocketError::HttpError(msg)) => {
        println!("HTTP error: {}", msg);
    }
    Ok(value) => {
        // Success
    }
}
```

### WebSocketError Variants

| Variant | Description |
|---------|-------------|
| `ConnectionFailed(String)` | Failed to establish connection |
| `ConnectionClosed { code, reason }` | Connection closed by server |
| `RateLimited` | Rate limit exceeded |
| `MessageParseError(String)` | Failed to parse message |
| `SequenceGap { expected, received }` | Sequence number gap detected |
| `ResyncRequired { orderbook_id }` | Full resync needed |
| `SubscriptionFailed(String)` | Subscription rejected |
| `PingTimeout` | No pong received |
| `Protocol(String)` | Protocol violation |
| `ServerError { code, message }` | Server-side error |
| `NotConnected` | Operation requires connection |
| `AlreadyConnected` | Already connected |
| `SendFailed(String)` | Failed to send message |
| `ChannelClosed` | Internal channel closed |
| `InvalidUrl(String)` | Invalid WebSocket URL |
| `Timeout` | Operation timed out |
| `Io(String)` | IO error |
| `AuthenticationFailed(String)` | Authentication failed |
| `AuthRequired` | Authentication required |
| `HttpError(String)` | HTTP request error |

### WsResult

```rust
pub type WsResult<T> = Result<T, WebSocketError>;
```
