# Orderbooks

Orderbook depth, decimals, live state management, and ticker data.

[← Overview](../../../README.md#orderbooks)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [State Container: OrderbookState](#state-container-orderbookstate)
- [Examples](#examples)
- [Wire Types](#wire-types)

## Types

### `OrderBookPair`

A tradable pair of conditional tokens within a market.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i32` | Internal pair ID |
| `market_pubkey` | `PubkeyStr` | Parent market's public key |
| `orderbook_id` | `OrderBookId` | Unique identifier (e.g., `"7BgBvyjr_EPjFWdd5"`) |
| `base` | `ConditionalToken` | Token being bought (bids) or sold (asks) |
| `quote` | `ConditionalToken` | Token being given (bids) or received (asks) |
| `outcome_index` | `i16` | Which outcome this pair represents |
| `tick_size` | `i64` | Minimum price increment |
| `total_bids` | `i32` | Number of resting bid orders |
| `total_asks` | `i32` | Number of resting ask orders |
| `last_trade_price` | `Option<Decimal>` | Most recent trade price |
| `last_trade_time` | `Option<DateTime<Utc>>` | Most recent trade timestamp |
| `active` | `bool` | Whether the orderbook is accepting orders |

**Associated functions:**

- `OrderBookPair::impact_pct(deposit_price, conditional_price)` -- price impact as a percentage relative to a deposit asset price
- `OrderBookPair::impact(deposit_asset_price, conditional_price)` -- full impact calculation with sign, percentage, and dollar difference

Both short-circuit to zero/empty when `deposit_price` is zero; `impact_pct` also returns `(0.0, "")` when `conditional_price` is zero. Callers typically source `conditional_price` from `pair.last_trade_price` (with `Decimal::ZERO` as a fallback).

### `OutcomeImpact`

Result of an impact calculation.

| Field | Type | Description |
|-------|------|-------------|
| `sign` | `String` | `"+"` or `"-"` |
| `is_positive` | `bool` | Whether impact is positive |
| `pct` | `f64` | Absolute percentage change |
| `dollar` | `Decimal` | Absolute dollar difference |

### `TickerData`

Best bid/ask/mid for an orderbook (from the `ticker` WS channel).

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | `OrderBookId` | Which orderbook |
| `best_bid` | `Option<Decimal>` | Highest bid price |
| `best_ask` | `Option<Decimal>` | Lowest ask price |
| `mid_price` | `Option<Decimal>` | Midpoint of bid and ask |

## Client Methods

Access via `client.orderbooks()`.

### `get`

```rust
async fn get(
    &self,
    orderbook_id: &str,
    depth: Option<u32>,
) -> Result<OrderbookDepthResponse, SdkError>
```

Fetch the current orderbook depth (bids and asks at each price level).

**Parameters:**
- `orderbook_id` -- the orderbook to query
- `depth` -- maximum number of price levels per side (default: all)

### `decimals`

```rust
async fn decimals(&self, orderbook_id: &str) -> Result<DecimalsResponse, SdkError>
```

Get the price and size decimals for an orderbook. Results are cached internally since decimals are effectively immutable.

The returned `DecimalsResponse` implements `OrderbookDecimals`, which is required by the order envelope's `apply_scaling()` method.

### `clear_cache`

```rust
async fn clear_cache(&self)
```

Clear the internal decimals cache. Rarely needed.

### On-Chain Cleanup Builders

```rust
fn close_orderbook_alt_ix(&self, params: &CloseOrderbookAltParams) -> Instruction
fn close_orderbook_alt_tx(&self, params: CloseOrderbookAltParams) -> Result<Transaction, SdkError>

fn close_orderbook_ix(&self, params: &CloseOrderbookParams) -> Instruction
fn close_orderbook_tx(&self, params: CloseOrderbookParams) -> Result<Transaction, SdkError>
```

Build cleanup instructions for resolved orderbooks. `CloseOrderbookAlt` deactivates or closes the orderbook lookup table; `CloseOrderbook` closes the orderbook PDA after the lookup table is closed.

## State Container: OrderbookState

`OrderbookState` is an app-owned state container for maintaining a live orderbook from WebSocket updates.

```rust
use lightcone::prelude::*;

let mut book = OrderbookState::new(OrderBookId::from("7BgBvyjr_EPjFWdd5"));
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `apply` | `fn apply(&mut self, book: &OrderBook) -> ApplyResult` | Apply a WS book message. Snapshots replace all state; deltas merge. Zero-size levels are removed. Returns the outcome of the apply. |
| `bids` | `fn bids(&self) -> &BTreeMap<Decimal, Decimal>` | All bids, sorted by price descending |
| `asks` | `fn asks(&self) -> &BTreeMap<Decimal, Decimal>` | All asks, sorted by price ascending |
| `best_bid` | `fn best_bid(&self) -> Option<Decimal>` | Highest bid price |
| `best_ask` | `fn best_ask(&self) -> Option<Decimal>` | Lowest ask price |
| `mid_price` | `fn mid_price(&self) -> Option<Decimal>` | Average of best bid and best ask |
| `spread` | `fn spread(&self) -> Option<Decimal>` | Best ask minus best bid |
| `is_empty` | `fn is_empty(&self) -> bool` | Whether the book has any levels |
| `clear` | `fn clear(&mut self)` | Reset to empty state |

### ApplyResult

Returned by `apply()` to indicate what happened:

| Variant | Description |
|---------|-------------|
| `Applied` | The snapshot or delta was successfully merged into the book. |
| `Ignored(IgnoreReason)` | The update was dropped and the consumer should take no recovery action. Reasons include an invalid zero-sequence delta, a stale/duplicate delta, or a later delta while the book is already awaiting a snapshot. |
| `RefreshRequired(RefreshReason)` | The book cannot be trusted until a fresh snapshot is applied. Reasons include a missing initial snapshot, a skipped sequence, or a backend resync request. |

**Sequence protocol:** The backend sends snapshots with `seq = 0` and starts delta sequences at `1`. Deltas are applied only when `seq == current + 1`. Out-of-order or duplicate deltas are ignored, and gaps trigger a re-snapshot. After `RefreshRequired`, later deltas are ignored with `IgnoreReason::AlreadyAwaitingSnapshot` until a snapshot is applied.

## Examples

### Fetch orderbook depth

```rust
use lightcone::prelude::*;

async fn show_depth(client: &LightconeClient, orderbook_id: &str) -> Result<(), SdkError> {
    let depth = client.orderbooks().get(orderbook_id, Some(10)).await?;
    println!("Bids: {:?}", depth.bids);
    println!("Asks: {:?}", depth.asks);
    Ok(())
}
```

### Maintain a live orderbook via WebSocket

```rust
use lightcone::prelude::*;
use futures_util::StreamExt;

async fn run_book_feed(client: &LightconeClient, orderbook_id: OrderBookId) {
    let mut ws = client.ws_native();
    ws.connect().await.unwrap();
    ws.subscribe(SubscribeParams::Books {
        orderbook_ids: vec![orderbook_id.clone()],
    }).unwrap();

    let mut snapshot = OrderbookState::new(orderbook_id);
    let mut stream = ws.events();

    while let Some(event) = stream.next().await {
        if let WsEvent::Message(Kind::BookUpdate(book)) = event {
            match snapshot.apply(&book) {
                ApplyResult::Applied => println!(
                    "Best bid: {:?} | Best ask: {:?} | Spread: {:?}",
                    snapshot.best_bid(),
                    snapshot.best_ask(),
                    snapshot.spread()
                ),
                ApplyResult::Ignored(reason) => {
                    eprintln!("Ignored book update: {reason:?}");
                }
                ApplyResult::RefreshRequired(reason) => {
                    eprintln!("Refresh required: {reason:?}");
                    // re-subscribe or request a fresh snapshot
                }
            }
        }
    }
}
```

## Wire Types

Raw backend response types are available in `lightcone::domain::orderbook::wire`, including `OrderbookDepthResponse`, `DecimalsResponse`, `BookOrder`, `OrderBook`, and `WsTickerData`.

---

[← Overview](../../../README.md#orderbooks)
