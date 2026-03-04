# WebSocket

Real-time data feeds for orderbooks, trades, user events, price history, ticker, and market lifecycle.

[Back to SDK root](../../README.md)

## Table of Contents

- [Channels](#channels)
- [Outbound Messages](#outbound-messages)
- [Inbound Messages](#inbound-messages)
- [WsEvent](#wsevent)
- [WsConfig](#wsconfig)
- [Native Client](#native-client)
- [WASM Client](#wasm-client)
- [Examples](#examples)

## Channels

| Channel | Subscribe with | Events | Description |
|---------|---------------|--------|-------------|
| Books | `SubscribeParams::Books { orderbook_ids }` | `Kind::BookUpdate` | Orderbook snapshots + deltas |
| Trades | `SubscribeParams::Trades { orderbook_ids }` | `Kind::Trade` | Individual trade executions |
| User | `SubscribeParams::User { wallet_address }` | `Kind::User` | Order updates, balance changes, snapshots |
| Price History | `SubscribeParams::PriceHistory { orderbook_id, resolution }` | `Kind::PriceHistory` | OHLCV snapshots + updates |
| Ticker | `SubscribeParams::Ticker { orderbook_ids }` | `Kind::Ticker` | Best bid/ask/mid prices |
| Market | `SubscribeParams::Market { market_pubkey }` | `Kind::Market` | Settlement, activation, pausing |

## Outbound Messages

`MessageOut` is the enum for client-to-server messages. Convenience constructors are provided:

| Constructor | Description |
|-------------|-------------|
| `MessageOut::subscribe_books(ids)` | Subscribe to orderbook updates |
| `MessageOut::unsubscribe_books(ids)` | Unsubscribe from orderbooks |
| `MessageOut::subscribe_trades(ids)` | Subscribe to trade events |
| `MessageOut::unsubscribe_trades(ids)` | Unsubscribe from trades |
| `MessageOut::subscribe_user(wallet)` | Subscribe to user events (requires auth) |
| `MessageOut::unsubscribe_user(wallet)` | Unsubscribe from user events |
| `MessageOut::subscribe_price_history(id, resolution)` | Subscribe to price candles |
| `MessageOut::unsubscribe_price_history(id, resolution)` | Unsubscribe from candles |
| `MessageOut::subscribe_ticker(ids)` | Subscribe to ticker updates |
| `MessageOut::unsubscribe_ticker(ids)` | Unsubscribe from ticker |
| `MessageOut::subscribe_market(pubkey)` | Subscribe to market lifecycle |
| `MessageOut::unsubscribe_market(pubkey)` | Unsubscribe from market |
| `MessageOut::ping()` | Application-level ping |

You can also construct subscribe/unsubscribe messages from `SubscribeParams` and `UnsubscribeParams` directly:

```rust
let msg = MessageOut::Subscribe(SubscribeParams::Books {
    orderbook_ids: vec![OrderBookId::from("7BgBvyjr_EPjFWdd5")],
});
```

## Inbound Messages

### `MessageIn`

Raw inbound message containing a `Kind` and `version`.

### `Kind`

Discriminated union of all inbound message types:

| Variant | Payload | Channel |
|---------|---------|---------|
| `Kind::BookUpdate(OrderBook)` | Orderbook snapshot or delta | `book_update` |
| `Kind::Trade(WsTrade)` | Single trade execution | `trades` |
| `Kind::User(UserUpdate)` | User snapshot, order update, or balance update | `user` |
| `Kind::PriceHistory(PriceHistory)` | Price candle snapshot or update | `price_history` |
| `Kind::Ticker(WsTickerData)` | Best bid/ask/mid | `ticker` |
| `Kind::Market(MarketEvent)` | Market lifecycle events | `market` |
| `Kind::Auth(AuthUpdate)` | Authentication status | `auth` |
| `Kind::Pong(Pong)` | Pong response | -- |
| `Kind::Error(WsError)` | Server-side error | -- |

### `UserUpdate`

The `User` channel delivers three event types:

| Variant | Description |
|---------|-------------|
| `UserUpdate::Snapshot(UserSnapshot)` | Full snapshot of orders, balances, and global deposits |
| `UserUpdate::Order(OrderEvent)` | Limit or trigger order update |
| `UserUpdate::BalanceUpdate(UserBalanceUpdate)` | Token balance change |

`OrderEvent` is further discriminated:

| Variant | Description |
|---------|-------------|
| `OrderEvent::Limit(OrderUpdate)` | Limit order placement, update, or cancellation |
| `OrderEvent::Trigger(TriggerOrderUpdate)` | Trigger order status change |

## WsEvent

High-level events emitted by the WS client to the consumer:

| Variant | Description |
|---------|-------------|
| `WsEvent::Message(Kind)` | A parsed message from the server |
| `WsEvent::Connected` | Connection established |
| `WsEvent::Disconnected { code, reason }` | Connection lost (may trigger reconnect) |
| `WsEvent::Error(String)` | Deserialization or protocol error |
| `WsEvent::MaxReconnectReached` | All automatic reconnect attempts exhausted |

## WsConfig

Configuration for the WS client. Defaults are sensible for most use cases.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | `String` | `"wss://tws.lightcone.xyz/ws"` | WebSocket server URL |
| `reconnect` | `bool` | `true` | Auto-reconnect on disconnect |
| `max_reconnect_attempts` | `u32` | `10` | Maximum reconnect attempts |
| `base_reconnect_delay_ms` | `u32` | `1000` | Base delay for exponential backoff |
| `ping_interval_ms` | `u32` | `30000` | Application-level ping interval |
| `pong_timeout_ms` | `u32` | `1000` | Pong response timeout |

The WS config is accessible from the client:

```rust
let config = client.ws_config().clone();
```

## Native Client

Requires the `ws-native` feature (included in `native` bundle). Uses `tokio-tungstenite` with a background task for connection management.

```rust
let mut ws = client.ws_native();
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `connect` | `async fn connect(&mut self) -> Result<(), WsError>` | Establish connection (spawns background task) |
| `disconnect` | `async fn disconnect(&mut self) -> Result<(), WsError>` | Graceful disconnect |
| `send` | `fn send(&self, msg: MessageOut) -> Result<(), WsError>` | Send a message to the server |
| `subscribe` | `fn subscribe(&self, params: SubscribeParams) -> Result<(), WsError>` | Subscribe to a channel |
| `unsubscribe` | `fn unsubscribe(&self, params: UnsubscribeParams) -> Result<(), WsError>` | Unsubscribe from a channel |
| `is_connected` | `fn is_connected(&self) -> bool` | Connection status |
| `ready_state` | `fn ready_state(&self) -> ReadyState` | Detailed connection state |
| `restart_connection` | `async fn restart_connection(&mut self)` | Force a fresh connection |
| `clear_authed_subscriptions` | `fn clear_authed_subscriptions(&self)` | Remove User channel from tracking |
| `events` | `fn events(&self) -> impl Stream<Item = WsEvent>` | Stream of events from the connection |

### Features

- **Automatic reconnection** with exponential backoff + jitter
- **Subscription tracking** -- subscriptions are automatically re-sent on reconnect
- **Message queue** -- messages sent while disconnected are flushed on reconnect
- **Ping/pong health check** -- detects stale connections
- **Auth token injection** -- passes the session cookie in the WS upgrade request

## WASM Client

Requires the `ws-wasm` feature (included in `wasm` bundle). Uses `web-sys::WebSocket` with static methods (one global connection).

```rust
use lightcone::ws::wasm::WsClient;

WsClient::connect(config, |event| {
    match event {
        WsEvent::Message(kind) => { /* handle */ }
        WsEvent::Connected => { /* ready */ }
        _ => {}
    }
});
```

### Methods

All methods are static (the WASM client manages a single global connection):

| Method | Signature | Description |
|--------|-----------|-------------|
| `connect` | `fn connect(config: WsConfig, on_event: impl Fn(WsEvent))` | Connect with event callback |
| `send` | `fn send(message: MessageOut)` | Send a message |
| `subscribe` | `fn subscribe(params: SubscribeParams)` | Subscribe to a channel |
| `unsubscribe` | `fn unsubscribe(params: UnsubscribeParams)` | Unsubscribe from a channel |
| `is_connected` | `fn is_connected() -> bool` | Connection status |
| `ready_state` | `fn ready_state() -> ReadyState` | Detailed state |
| `restart_connection` | `fn restart_connection()` | Force reconnect |
| `cleanup` | `fn cleanup()` | Tear down connection and clean up |

## Examples

### Market maker: subscribe to book + user events

```rust
use lightcone::prelude::*;
use futures_util::StreamExt;

async fn run_market_maker(
    client: &LightconeClient,
    orderbook_id: OrderBookId,
    wallet: PubkeyStr,
) {
    let mut ws = client.ws_native();
    ws.connect().await.unwrap();

    // Subscribe to book updates and user events
    ws.subscribe(SubscribeParams::Books {
        orderbook_ids: vec![orderbook_id.clone()],
    }).unwrap();
    ws.subscribe(SubscribeParams::User {
        wallet_address: wallet,
    }).unwrap();

    let mut book = OrderbookSnapshot::new(orderbook_id);
    let mut open_orders = UserOpenOrders::new();
    let mut stream = ws.events();

    while let Some(event) = stream.next().await {
        match event {
            WsEvent::Connected => {
                println!("WebSocket connected");
            }
            WsEvent::Message(Kind::BookUpdate(update)) => {
                book.apply(&update);
                if let (Some(bid), Some(ask)) = (book.best_bid(), book.best_ask()) {
                    println!("Book: {} / {}", bid, ask);
                }
            }
            WsEvent::Message(Kind::User(UserUpdate::Snapshot(snapshot))) => {
                println!("User snapshot: {} orders", snapshot.orders.len());
            }
            WsEvent::Message(Kind::User(UserUpdate::Order(OrderEvent::Limit(update)))) => {
                open_orders.upsert(&update);
                println!("Order {}: {:?}", update.order.order_hash, update.order.status);
            }
            WsEvent::Disconnected { code, reason } => {
                println!("Disconnected: {:?} {}", code, reason);
            }
            WsEvent::MaxReconnectReached => {
                println!("Max reconnect attempts reached, exiting");
                break;
            }
            _ => {}
        }
    }
}
```

### Subscribe to multiple channels

```rust
use lightcone::prelude::*;

fn setup_subscriptions(ws: &lightcone::ws::native::WsClient, orderbook_ids: Vec<OrderBookId>) {
    // Book depth
    ws.subscribe(SubscribeParams::Books {
        orderbook_ids: orderbook_ids.clone(),
    }).unwrap();

    // Live trades
    ws.subscribe(SubscribeParams::Trades {
        orderbook_ids: orderbook_ids.clone(),
    }).unwrap();

    // Ticker (best bid/ask)
    ws.subscribe(SubscribeParams::Ticker {
        orderbook_ids,
    }).unwrap();
}
```
