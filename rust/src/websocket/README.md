# WebSocket Module Reference

Real-time data streaming for Lightcone markets.

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
| `pong_timeout_secs` | u64 | 60 | Timeout for pong response (seconds) |
| `auto_reconnect` | bool | true | Auto-reconnect on disconnect |
| `auto_resubscribe` | bool | true | Re-subscribe after reconnect |
| `auth_token` | Option\<String\> | None | Pre-obtained auth token |
| `event_channel_capacity` | usize | 1000 | Capacity of the event channel |
| `command_channel_capacity` | usize | 100 | Capacity of the command channel |

**Backoff Formula:** `delay = min(base_delay * 2^(attempt-1), max_delay)` with full jitter

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
            if let Some(book) = client.get_orderbook(&orderbook_id).await {
                println!("Best bid: {:?}", book.best_bid());
                println!("Spread: {:?}", book.spread());
            }
        }
        WsEvent::Trade { orderbook_id, trade } => {
            println!("{}: {:?} @ {} size {}",
                orderbook_id, trade.side, trade.price, trade.size);
        }
        WsEvent::UserUpdate { event_type, user } => {
            if let Some(state) = client.get_user_state(&user).await {
                println!("Orders: {}", state.orders.len());
            }
        }
        WsEvent::PriceUpdate { orderbook_id, resolution } => {
            if let Some(history) = client.get_price_history(&orderbook_id, &resolution).await {
                if let Some(candle) = history.latest_candle() {
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
if let Some(book) = client.get_orderbook("orderbook_id").await {
    // Price levels
    let bids = book.get_bids();       // All bids (descending price)
    let asks = book.get_asks();       // All asks (ascending price)
    let top_bids = book.get_top_bids(5);  // Top 5 bids
    let top_asks = book.get_top_asks(5);  // Top 5 asks

    // Best prices - returns (price, size) tuple
    let best_bid = book.best_bid();   // Option<(String, String)>
    let best_ask = book.best_ask();   // Option<(String, String)>
    let mid = book.midpoint();        // Option<String>
    let spread = book.spread();       // Option<String>

    // Size at specific price
    let bid_size = book.bid_size_at("0.500000");  // Option<String>
    let ask_size = book.ask_size_at("0.510000");  // Option<String>

    // Depth totals (returns Decimal for precision)
    let total_bids = book.total_bid_depth();
    let total_asks = book.total_ask_depth();

    // Level counts
    let bid_levels = book.bid_count();
    let ask_levels = book.ask_count();

    // Metadata
    let id = &book.orderbook_id;
    let has_data = book.has_snapshot();
    let seq = book.expected_sequence();
    let ts = book.last_timestamp();
}
```

**LocalOrderbook Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | String | Orderbook identifier |
| `bids` | BTreeMap | Price → Size (descending by numeric value) |
| `asks` | BTreeMap | Price → Size (ascending by numeric value) |

**LocalOrderbook Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(orderbook_id)` | Self | Create new instance |
| `apply_snapshot(update)` | () | Apply full snapshot |
| `apply_delta(update)` | Result\<(), WebSocketError\> | Apply incremental update |
| `apply_update(update)` | Result\<(), WebSocketError\> | Apply any update type |
| `get_bids()` | Vec\<PriceLevel\> | All bids (descending price) |
| `get_asks()` | Vec\<PriceLevel\> | All asks (ascending price) |
| `get_top_bids(n)` | Vec\<PriceLevel\> | Top n bids |
| `get_top_asks(n)` | Vec\<PriceLevel\> | Top n asks |
| `best_bid()` | Option\<(String, String)\> | Best bid (price, size) |
| `best_ask()` | Option\<(String, String)\> | Best ask (price, size) |
| `midpoint()` | Option\<String\> | Midpoint price |
| `spread()` | Option\<String\> | Bid-ask spread |
| `bid_size_at(price)` | Option\<String\> | Size at bid price |
| `ask_size_at(price)` | Option\<String\> | Size at ask price |
| `total_bid_depth()` | Decimal | Sum of all bid sizes |
| `total_ask_depth()` | Decimal | Sum of all ask sizes |
| `bid_count()` | usize | Number of bid levels |
| `ask_count()` | usize | Number of ask levels |
| `has_snapshot()` | bool | Has received initial snapshot |
| `expected_sequence()` | u64 | Expected next sequence number |
| `last_timestamp()` | Option\<&str\> | Last update timestamp |
| `clear()` | () | Clear state for resync |

### UserState

Maintains user's orders and balances.

```rust
if let Some(state) = client.get_user_state("user_pubkey").await {
    // Orders
    for (hash, order) in &state.orders {
        println!("{}: {} remaining", hash, order.remaining);
    }
    let order = state.get_order("order_hash");
    let open = state.open_orders();
    let market_orders = state.orders_for_market("market_pubkey");
    let ob_orders = state.orders_for_orderbook("orderbook_id");
    let count = state.order_count();

    // Balances
    let balance = state.get_balance("market_pubkey", "deposit_mint");
    let all = state.all_balances();

    // Outcome-specific balances
    let idle = state.idle_balance_for_outcome("market", "mint", 0);
    let on_book = state.on_book_balance_for_outcome("market", "mint", 0);

    // State info
    let has_snap = state.has_snapshot();
    let ts = state.last_timestamp();
}
```

**UserState Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `user` | String | User public key |
| `orders` | HashMap\<String, Order\> | Order hash → Order |
| `balances` | HashMap\<String, BalanceEntry\> | Key → Balance |

**UserState Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(user)` | Self | Create new instance |
| `apply_snapshot(data)` | () | Apply full snapshot |
| `apply_order_update(data)` | () | Apply order update |
| `apply_balance_update(data)` | () | Apply balance update |
| `apply_event(data)` | () | Apply any event type |
| `get_order(hash)` | Option\<&Order\> | Get order by hash |
| `open_orders()` | Vec\<&Order\> | All open orders |
| `orders_for_market(market)` | Vec\<&Order\> | Orders for specific market |
| `orders_for_orderbook(ob_id)` | Vec\<&Order\> | Orders for specific orderbook |
| `order_count()` | usize | Number of open orders |
| `get_balance(market, mint)` | Option\<&BalanceEntry\> | Get balance entry |
| `all_balances()` | Vec\<&BalanceEntry\> | All balance entries |
| `idle_balance_for_outcome(market, mint, idx)` | Option\<String\> | Idle balance for outcome |
| `on_book_balance_for_outcome(market, mint, idx)` | Option\<String\> | On-book balance for outcome |
| `has_snapshot()` | bool | Has received initial snapshot |
| `last_timestamp()` | Option\<&str\> | Last update timestamp |
| `clear()` | () | Clear state for resync |

### PriceHistory

Maintains candle data for an orderbook.

```rust
if let Some(history) = client.get_price_history("orderbook_id", "1h").await {
    // All candles (newest first)
    for candle in history.candles() {
        println!("t={} c={:?}", candle.t, candle.c);
    }

    // Most recent candle
    if let Some(candle) = history.latest_candle() {
        println!("Latest: {:?}", candle);
    }

    // Oldest candle
    if let Some(candle) = history.oldest_candle() {
        println!("Oldest: {:?}", candle);
    }

    // N most recent candles
    let recent = history.recent_candles(10);

    // Candle at specific timestamp
    let candle = history.get_candle(timestamp);

    // Current prices from most recent candle
    let mid = history.current_midpoint();
    let bid = history.current_best_bid();
    let ask = history.current_best_ask();

    // State info
    let count = history.candle_count();
    let res = history.resolution_enum();
    let server_t = history.server_time();
    let has_snap = history.has_snapshot();
}
```

**PriceHistory Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | String | Orderbook identifier |
| `resolution` | String | Candle resolution |
| `include_ohlcv` | bool | OHLCV enabled |

**PriceHistory Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(orderbook_id, resolution, include_ohlcv)` | Self | Create new instance |
| `apply_snapshot(data)` | () | Apply full snapshot |
| `apply_update(data)` | () | Apply candle update |
| `apply_heartbeat(data)` | () | Apply heartbeat |
| `apply_event(data)` | () | Apply any event type |
| `candles()` | &[Candle] | All candles (newest first) |
| `recent_candles(n)` | &[Candle] | N most recent candles |
| `get_candle(timestamp)` | Option\<&Candle\> | Candle at timestamp |
| `latest_candle()` | Option\<&Candle\> | Most recent candle |
| `oldest_candle()` | Option\<&Candle\> | Oldest candle |
| `current_midpoint()` | Option\<String\> | Midpoint from latest candle |
| `current_best_bid()` | Option\<String\> | Best bid from latest candle |
| `current_best_ask()` | Option\<String\> | Best ask from latest candle |
| `candle_count()` | usize | Number of candles |
| `has_snapshot()` | bool | Has received snapshot |
| `last_timestamp()` | Option\<i64\> | Last candle timestamp |
| `server_time()` | Option\<i64\> | Server time reference |
| `resolution_enum()` | Option\<Resolution\> | Resolution as enum |
| `clear()` | () | Clear state for resync |

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
use lightcone_sdk::websocket::{generate_signin_message, generate_signin_message_with_timestamp, authenticate};

// Generate sign-in message with current timestamp
let message = generate_signin_message()?;

// Or with a specific timestamp (milliseconds)
let message = generate_signin_message_with_timestamp(1704067200000);

// Sign message
let signature = signing_key.sign(message.as_bytes());

// Get auth token from API
let auth_token = authenticate(&signing_key).await?;

// Connect with token
let client = LightconeWebSocketClient::connect_with_auth(url, auth_token.auth_token).await?;
```

**Sign-in Message Format:**
```text
Sign in to Lightcone

Timestamp: {unix_ms}
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
| `InvalidAuthToken(String)` | Invalid or empty auth token |

### WsResult

```rust
pub type WsResult<T> = Result<T, WebSocketError>;
```
