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
| `tick_size` | `i64` | Deprecated market metadata; never use for order admission |
| `total_bids` | `i32` | Number of resting bid orders |
| `total_asks` | `i32` | Number of resting ask orders |
| `last_trade_price` | `Option<Decimal>` | Most recent trade price |
| `last_trade_time` | `Option<DateTime<Utc>>` | Most recent trade timestamp |
| `active` | `bool` | Whether the orderbook is accepting orders |

**Associated functions:**

- `OrderBookPair::impact_pct(deposit_price, conditional_price)` -- price impact as a percentage relative to a deposit asset price
- `OrderBookPair::impact(deposit_asset_price, conditional_price)` -- full impact calculation with direction, percentage, and dollar difference

Both short-circuit to zero/empty when `deposit_price` is zero; `impact_pct` also returns `(0.0, "")` when `conditional_price` is zero. Callers typically source `conditional_price` from `pair.last_trade_price` (with `Decimal::ZERO` as a fallback).

### `OutcomeImpact`

Result of an impact calculation.

| Field | Type | Description |
|-------|------|-------------|
| `direction` | `ImpactDirection` | Exact `Negative`, `Zero`, or `Positive` direction |
| `pct` | `f64` | Absolute percentage change |
| `dollar` | `Decimal` | Absolute dollar difference |

Use `OutcomeImpact::sign()` when a display sign is needed. It returns `"-"`, `""`, or `"+"` from the direction without storing a second representation that can disagree.

### `TickerData`

Best bid/ask/mid for an orderbook (from the `ticker` WS channel).

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | `OrderBookId` | Which orderbook |
| `best_bid` | `Option<Decimal>` | Highest bid price |
| `best_ask` | `Option<Decimal>` | Lowest ask price |
| `mid_price` | `Option<Decimal>` | Engine-authoritative midpoint, including one-sided/last-trade fallback |

## Client Methods

Access via `client.orderbooks()`.

### `get`

```rust
async fn get(
    &self,
    orderbook_id: &str,
    depth: Option<u32>,
    aggregation: BookAggregation,
) -> Result<OrderbookDepthResponse, SdkError>
```

Fetch the current orderbook depth (bids and asks at each price level).

**Parameters:**
- `orderbook_id` -- the orderbook to query
- `depth` -- maximum number of price levels per side. Capped server-side at 20 (omitted, `0`, or `>20` all serve 20).
- `aggregation` -- Hyperliquid-style grouping (`BookAggregation::FULL` for the raw book). Invalid combinations are rejected client-side (`SdkError::Validation`) before any request; the server would 400 with `INVALID_ORDERBOOK_QUERY`. Unknown query params are rejected server-side — only `depth`, `nSigFigs`, and `mantissa` are sent.

### `BookAggregation`

Shared aggregation value type (re-exported in the prelude): `n_sig_figs` 2–5, `mantissa` 1/2/5 only with `n_sig_figs = 5`, `(5, None)` normalized to `(5, 1)`. Key helpers: `validate(n, m)`, `normalized()`, `from_frame(n, m)` (untagged ⇒ full precision), `is_full()`, `key_suffix()`. Bids bucket by flooring, asks by ceiling, sizes summed per bucket.

### `decimals`

```rust
async fn decimals(&self, orderbook_id: &str) -> Result<DecimalsResponse, SdkError>
```

Get the complete immutable `OrderbookRules` required for exact order admission.
Both raw quantum fields are parsed from JSON strings as arbitrary-precision
integers. Results are cached per client/orderbook; failed requests are not cached.
The display quantum strings are never used for arithmetic.

### Cache invalidation

```rust
async fn invalidate_decimals(&self, orderbook_id: &str)
async fn clear_decimals_cache(&self)
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

One connection may hold multiple aggregation views of the same orderbook — key your `OrderbookState` instances by `(orderbook_id, aggregation)` using `OrderBook::aggregation()` on each incoming frame.

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `apply` | `fn apply(&mut self, book: &OrderBook) -> ApplyResult` | Discard duplicate/older revisions; otherwise replace the full snapshot and preserve truncation flags. |
| `begin_generation` | `fn begin_generation(&mut self)` | Reset the sequence gate on reconnect/resubscribe while retaining visible levels. |
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
| `Applied` | The snapshot replaced the book. Every non-resync data frame is a snapshot by contract; the `is_snapshot` flag is not consulted. |
| `DiscardedStale` | The frame revision was equal to or lower than the last accepted revision in this generation. |
| `SubscriptionMismatch` | The frame belonged to another orderbook or aggregation view. |
| `RefreshRequired(RefreshReason::ServerResync)` | The backend requested a resync: unsubscribe and re-subscribe with the same parameters (including aggregation) to receive a fresh snapshot. The book is left untouched. |

**Sequence protocol:** Every accepted frame is a full top-20 replacement.
The initial snapshot carries its real engine revision. Within one subscription
generation, discard `seq <= last_seq` and accept any forward jump. Call
`begin_generation()` on reconnect, unsubscribe/resubscribe, aggregation change,
book recreation, or resync. Revision gaps are normal and never require resync.

Depth responses and WebSocket state expose `bids_truncated` and
`asks_truncated`. A true flag means the returned levels are useful but that side
must not be presented as exhaustive liquidity. REST depth also exposes required
`revision` and `captured_at_ms` freshness metadata.

REST depth is a coherent projection and may briefly lag an authoritative book
mutation. Compare `revision` and `captured_at_ms` when freshness matters; do not
poll for the next consecutive integer because projection revisions may jump.

## Examples

### Fetch orderbook depth

```rust
use lightcone::prelude::*;

async fn show_depth(client: &LightconeClient, orderbook_id: &str) -> Result<(), SdkError> {
    let depth = client
        .orderbooks()
        .get(orderbook_id, Some(10), BookAggregation::FULL)
        .await?;
    println!("Bids: {:?}", depth.bids);
    println!("Asks: {:?}", depth.asks);

    // Grouped to 5 significant figures with a mantissa-2 sub-step:
    let aggregation = BookAggregation::validate(Some(5), Some(2))
        .map_err(|message| SdkError::Validation(message.to_string()))?;
    let grouped = client.orderbooks().get(orderbook_id, None, aggregation).await?;
    println!("Grouped bids: {:?}", grouped.bids);
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
        n_sig_figs: None,
        mantissa: None,
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

Raw backend response types are available in `lightcone::domain::orderbook::wire`, including `OrderbookDepthResponse` (with required display `decimals`), `DecimalsResponse`, `OrderBook` (with optional `n_sig_figs`/`mantissa` aggregation tags and an `aggregation()` helper), `WsBookLevel`, and `WsTickerData`.

---

[← Overview](../../../README.md#orderbooks)
