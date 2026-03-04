# Trades

Trade execution history and real-time trade feeds.

[Back to SDK root](../../README.md)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [State Container: TradeHistory](#state-container-tradehistory)
- [Examples](#examples)
- [Wire Types](#wire-types)

## Types

### `Trade`

A single trade execution record.

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | `OrderBookId` | Which orderbook the trade occurred on |
| `trade_id` | `String` | Unique trade identifier |
| `timestamp` | `DateTime<Utc>` | Execution timestamp |
| `price` | `Decimal` | Trade price |
| `size` | `Decimal` | Trade size |
| `side` | `Side` | Taker side (`Bid` or `Ask`) |

### `TradesPage`

A paginated page of trades.

| Field | Type | Description |
|-------|------|-------------|
| `trades` | `Vec<Trade>` | Trade records for this page |
| `next_cursor` | `Option<i64>` | Cursor for the next page (`None` if last page) |
| `has_more` | `bool` | Whether more trades exist |

## Client Methods

Access via `client.trades()`.

### `get`

```rust
async fn get(
    &self,
    orderbook_id: &str,
    limit: Option<u32>,
    before: Option<i64>,
) -> Result<TradesPage, SdkError>
```

Fetch trades for an orderbook with cursor-based pagination.

**Parameters:**
- `orderbook_id` -- which orderbook to query
- `limit` -- maximum number of trades to return
- `before` -- cursor from `next_cursor` of a previous response (fetch older trades)

## State Container: TradeHistory

`TradeHistory` is an app-owned rolling buffer for maintaining a live trade feed from WebSocket updates.

```rust
use lightcone::prelude::*;

let mut history = TradeHistory::new(OrderBookId::from("7BgBvyjr_EPjFWdd5"), 100);
```

### Methods

| Method | Description |
|--------|-------------|
| `new(orderbook_id, max_size)` | Create a buffer with the given capacity |
| `push(trade)` | Append a trade (evicts oldest if at capacity) |
| `replace(trades)` | Replace all trades (e.g., from an initial REST fetch) |
| `trades()` | Get all trades as a `VecDeque<Trade>` |
| `latest()` | Get the most recent trade |
| `len()` / `is_empty()` | Size helpers |
| `clear()` | Remove all trades |

## Examples

### Paginate through trade history

```rust
use lightcone::prelude::*;

async fn recent_trades(
    client: &LightconeClient,
    orderbook_id: &str,
) -> Result<Vec<Trade>, SdkError> {
    let mut all_trades = Vec::new();
    let mut cursor = None;

    loop {
        let page = client.trades().get(orderbook_id, Some(100), cursor).await?;
        all_trades.extend(page.trades);

        if !page.has_more {
            break;
        }
        cursor = page.next_cursor;
    }

    Ok(all_trades)
}
```

### Maintain a live trade feed via WebSocket

```rust
use lightcone::prelude::*;
use futures_util::StreamExt;

async fn live_trades(client: &LightconeClient, orderbook_id: OrderBookId) {
    let mut ws = client.ws_native();
    ws.connect().await.unwrap();
    ws.subscribe(SubscribeParams::Trades {
        orderbook_ids: vec![orderbook_id.clone()],
    }).unwrap();

    let mut history = TradeHistory::new(orderbook_id, 100);
    let mut stream = ws.events();

    while let Some(event) = stream.next().await {
        if let WsEvent::Message(Kind::Trade(ws_trade)) = event {
            let trade: Trade = ws_trade.into();
            history.push(trade.clone());
            println!("{}: {} @ {} ({})", trade.trade_id, trade.size, trade.price, trade.side);
        }
    }
}
```

## Wire Types

Raw types in `lightcone::domain::trade::wire` include `TradeResponse`, `WsTrade`, and `TradesResponse`.
