# Trades

Trade execution history and real-time trade feeds.

[← Overview](../../../README.md#trades)

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
| `trade_id` | `String` | `"<execution_id>:<leg_index>:<projection_generation>"`, shared by REST and WS (WS derives it via `WsTrade::trade_id()`) |
| `cursor_id` | `Option<i64>` | Numeric REST row id used for pagination (`None` on WS trades) |
| `timestamp` | `DateTime<Utc>` | Execution timestamp |
| `price` | `Decimal` | Quote units per base unit: REST reports the maker price truncated to price decimals; WS derives `quote_amount / base_amount` |
| `size` | `Decimal` | Size in base-token units |
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
    cursor: Option<i64>,
) -> Result<TradesPage, SdkError>
```

Fetch trades for an orderbook with cursor-based pagination.

**Parameters:**
- `orderbook_id` -- which orderbook to query
- `limit` -- maximum number of trades to return
- `cursor` -- numeric cursor from `next_cursor` of a previous response (fetch older trades)

### `get_by_market`

```rust
async fn get_by_market(
    &self,
    market_pubkey: &str,
    limit: Option<u32>,
    cursor: Option<i64>,
) -> Result<TradesPage, SdkError>
```

Fetch trades across all orderbooks in a market, interleaved by time. Each `Trade` in the result carries its own `orderbook_id` so callers can distinguish which outcome it belongs to.

**Parameters:**
- `market_pubkey` -- market public key to query
- `limit` -- maximum number of trades to return
- `cursor` -- numeric cursor from `next_cursor` of a previous response (fetch older trades)

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
| `push(trade)` | Insert newest-first by execution time, deduplicating by `trade_id` (evicts oldest if at capacity) |
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
            let trade = Trade::try_from(ws_trade).unwrap();
            history.push(trade.clone());
            println!("{}: {} @ {} ({})", trade.trade_id, trade.size, trade.price, trade.side);
        }
    }
}
```

## Wire Types

Raw types in `lightcone::domain::trade::wire` include `TradeResponse` (signed `taker_fee_estimate`/`maker_fee_estimate`, `i64` `id` cursor), `TradesResponse`, `MarketTradesResponse`, and `WsTrade`. `WsTrade` is the committed fill fact on the `trades` channel (and the `user` channel `fill` event): `fill_id` (relay dedup id, not the REST `trade_id`), `execution_id`, `base_amount`, `quote_amount`, maker/taker order hashes and wallets, `taker_side`, signed fee estimates in `fee_mint` units, fee bps, `executed_at`, plus the flattened `CommitInfo`. It carries no price, side, trade id, or sequence: use `price()` and `trade_id()`. `Trade::try_from` converts a `WsTrade` or `TradeResponse` and fails with `SdkError::Validation` rather than defaulting a missing price, an unparseable amount, or an invalid timestamp.

---

[← Overview](../../../README.md#trades)
