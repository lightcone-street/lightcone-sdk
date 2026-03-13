# Price History

OHLCV candle data and real-time price chart updates.

[← Overview](../../../README.md#price-history)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [State Container: PriceHistoryState](#state-container-pricehistorystate)
- [Examples](#examples)
- [Wire Types](#wire-types)

## Types

### `LineData`

A single data point on a price chart.

| Field | Type | Description |
|-------|------|-------------|
| `time` | `i64` | Unix timestamp in milliseconds |
| `value` | `String` | Midpoint value as a decimal string |

### `Resolution`

Candle resolution for price history queries.

| Variant | Serializes as | Duration |
|---------|---------------|----------|
| `Minute1` | `"1m"` | 60 seconds |
| `Minute5` | `"5m"` | 5 minutes |
| `Minute15` | `"15m"` | 15 minutes |
| `Hour1` | `"1h"` | 1 hour |
| `Hour4` | `"4h"` | 4 hours |
| `Day1` | `"1d"` | 1 day |

`Resolution::seconds()` returns the duration of one candle in seconds.

### `OrderbookPriceHistoryQuery`

Query options for `GET /api/price-history?orderbook_id=...`.

| Field | Type | Description |
|-------|------|-------------|
| `resolution` | `Resolution` | Candle resolution |
| `from` | `Option<u64>` | Optional start time in Unix milliseconds |
| `to` | `Option<u64>` | Optional end time in Unix milliseconds |
| `cursor` | `Option<u64>` | Optional pagination cursor (`next_cursor`) |
| `limit` | `Option<usize>` | Optional page size, max 1000 |
| `include_ohlcv` | `bool` | Whether to request full OHLCV candles |

### `DepositPriceHistoryQuery`

Query options for `GET /api/price-history?deposit_asset=...`.

| Field | Type | Description |
|-------|------|-------------|
| `resolution` | `Resolution` | Candle resolution |
| `from` | `Option<u64>` | Optional start time in Unix milliseconds |
| `to` | `Option<u64>` | Optional end time in Unix milliseconds |
| `cursor` | `Option<u64>` | Optional pagination cursor (`next_cursor`) |
| `limit` | `Option<usize>` | Optional page size, max 1000 |

## Client Methods

Access via `client.price_history()`.

### `get`

```rust
async fn get(
    &self,
    orderbook_id: &str,
    resolution: Resolution,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<OrderbookPriceHistoryResponse, SdkError>
```

Fetch orderbook price history for an orderbook with the simple parameter set.

**Parameters:**
- `orderbook_id` -- which orderbook to query
- `resolution` -- candle resolution
- `from` -- optional start time as Unix timestamp in milliseconds
- `to` -- optional end time as Unix timestamp in milliseconds

### `get_with_query`

```rust
async fn get_with_query(
    &self,
    orderbook_id: &str,
    query: OrderbookPriceHistoryQuery,
) -> Result<OrderbookPriceHistoryResponse, SdkError>
```

Fetch orderbook price history with support for `cursor`, `limit`, and `include_ohlcv`.

### `get_deposit_asset`

```rust
async fn get_deposit_asset(
    &self,
    deposit_asset: &str,
    query: DepositPriceHistoryQuery,
) -> Result<DepositPriceHistoryResponse, SdkError>
```

Fetch deposit-token price history from the same endpoint using `deposit_asset`.

## State Container: PriceHistoryState

`PriceHistoryState` manages live price chart data from WebSocket updates, keyed by orderbook + resolution.

```rust
use lightcone::prelude::*;

let mut state = PriceHistoryState::new();
```

### Methods

| Method | Description |
|--------|-------------|
| `new()` | Create empty state |
| `apply_snapshot(orderbook_id, resolution, prices)` | Replace all data for an orderbook+resolution |
| `apply_update(orderbook_id, resolution, point)` | Append or update the latest candle |
| `get(orderbook_id, resolution)` | Get price data for an orderbook+resolution |
| `clear()` | Remove all data |

## Examples

### Fetch hourly candles

```rust
use lightcone::prelude::*;

async fn get_price_chart(
    client: &LightconeClient,
    orderbook_id: &str,
) -> Result<(), SdkError> {
    let data = client.price_history()
        .get_with_query(
            orderbook_id,
            OrderbookPriceHistoryQuery {
                resolution: Resolution::Hour1,
                include_ohlcv: true,
                limit: Some(10),
                ..OrderbookPriceHistoryQuery::default()
            },
        )
        .await?;

    println!("Price history: {}", serde_json::to_string_pretty(&data)?);
    Ok(())
}
```

### Fetch deposit-token candles

```rust
use lightcone::domain::price_history::DepositPriceHistoryQuery;
use lightcone::prelude::*;

async fn get_deposit_price_chart(
    client: &LightconeClient,
    deposit_asset: &str,
) -> Result<(), SdkError> {
    let data = client.price_history()
        .get_deposit_asset(
            deposit_asset,
            DepositPriceHistoryQuery {
                resolution: Resolution::Hour1,
                limit: Some(10),
                ..DepositPriceHistoryQuery::default()
            },
        )
        .await?;

    println!("Deposit price history: {}", serde_json::to_string_pretty(&data)?);
    Ok(())
}
```

### Stream live price updates via WebSocket

```rust
use lightcone::prelude::*;
use futures_util::StreamExt;

async fn live_chart(client: &LightconeClient, orderbook_id: OrderBookId) {
    let mut ws = client.ws_native();
    ws.connect().await.unwrap();
    ws.subscribe(SubscribeParams::PriceHistory {
        orderbook_id: orderbook_id.clone(),
        resolution: Resolution::Minute1,
        include_ohlcv: false,
    }).unwrap();

    let mut state = PriceHistoryState::new();
    let mut stream = ws.events();

    while let Some(event) = stream.next().await {
        if let WsEvent::Message(Kind::PriceHistory(ph)) = event {
            // ph contains snapshot or update data from the wire type
            println!("Price history update received");
        }
    }
}
```

## Wire Types

Raw types in `lightcone::domain::price_history::wire` include:
- `PriceHistory`, `PriceCandle`, and related WS types
- `OrderbookPriceHistoryResponse`
- `DepositPriceHistoryResponse`
- `OrderbookPriceCandle`
- `DepositPriceCandle`

---

[← Overview](../../../README.md#price-history)
