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
) -> Result<serde_json::Value, SdkError>
```

Fetch OHLCV price history for an orderbook.

**Parameters:**
- `orderbook_id` -- which orderbook to query
- `resolution` -- candle resolution
- `from` -- optional start time as Unix timestamp in milliseconds
- `to` -- optional end time as Unix timestamp in milliseconds

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
        .get(orderbook_id, Resolution::Hour1, None, None)
        .await?;

    println!("Price history: {}", serde_json::to_string_pretty(&data)?);
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

Raw types in `lightcone::domain::price_history::wire` include `PriceHistory` (WS snapshot/update), `PriceCandle`, and related types.

---

[← Overview](../../../README.md#price-history)
