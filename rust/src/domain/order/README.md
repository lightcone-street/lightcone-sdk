# Orders

Submit, cancel, and track limit and trigger orders.

[← Overview](../../../README.md#orders)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [Order Envelope Builder](#order-envelope-builder)
- [State Containers](#state-containers)
- [Examples](#examples)
- [Wire Types](#wire-types)

## Types

### `Order`

A validated, domain-level limit order.

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | `String` | Unique order identifier |
| `market_pubkey` | `PubkeyStr` | Parent market |
| `orderbook_id` | `OrderBookId` | Which orderbook |
| `side` | `Side` | `Bid` (buy) or `Ask` (sell) |
| `price` | `Decimal` | Order price |
| `size` | `Decimal` | Total size |
| `filled_size` | `Decimal` | Amount filled so far |
| `remaining_size` | `Decimal` | Amount remaining |
| `status` | `OrderStatus` | Current status |
| `base_mint` | `PubkeyStr` | Base token mint |
| `quote_mint` | `PubkeyStr` | Quote token mint |
| `outcome_index` | `i16` | Which outcome |
| `tx_signature` | `Option<String>` | On-chain transaction signature |
| `created_at` | `DateTime<Utc>` | Creation timestamp |

### `TriggerOrder`

A take-profit or stop-loss trigger order. Held server-side until the trigger price is hit, then submitted as a limit order.

| Field | Type | Description |
|-------|------|-------------|
| `trigger_order_id` | `String` | Trigger order ID |
| `order_hash` | `String` | Underlying order hash |
| `market_pubkey` | `PubkeyStr` | Parent market |
| `orderbook_id` | `OrderBookId` | Which orderbook |
| `trigger_price` | `Decimal` | Price threshold that fires the order |
| `trigger_type` | `TriggerType` | `TakeProfit` (`"TP"`) or `StopLoss` (`"SL"`) |
| `side` | `Side` | `Bid` or `Ask` |
| `amount_in` | `Decimal` | Amount the maker gives |
| `amount_out` | `Decimal` | Amount the maker receives |
| `time_in_force` | `TimeInForce` | Execution constraint when triggered |
| `created_at` | `DateTime<Utc>` | Creation timestamp |

### `OrderStatus`

| Variant | Description |
|---------|-------------|
| `Open` | Resting on the book |
| `Matching` | Currently being matched |
| `Filled` | Fully filled |
| `Cancelled` | Cancelled by user or system |
| `Pending` | Awaiting processing |

### `OrderType`

| Variant | Description |
|---------|-------------|
| `Limit` | Standard limit order |
| `Market` | Market order (immediate execution) |
| `Deposit` | Deposit operation |
| `Withdraw` | Withdrawal operation |

### `TimeInForce`

Execution constraint for trigger orders.

| Variant | Serializes as | Description |
|---------|---------------|-------------|
| `Gtc` | `"GTC"` | Good-til-cancelled (default) |
| `Ioc` | `"IOC"` | Immediate-or-cancel |
| `Fok` | `"FOK"` | Fill-or-kill |
| `Alo` | `"ALO"` | Add-liquidity-only (post-only) |

### `TriggerType`

| Variant | Serializes as | Description |
|---------|---------------|-------------|
| `TakeProfit` | `"TP"` | Fires when price rises above trigger |
| `StopLoss` | `"SL"` | Fires when price falls below trigger |

## Client Methods

Access via `client.orders()`.

### `submit`

```rust
async fn submit(&self, request: &impl Serialize) -> Result<SubmitOrderResponse, SdkError>
```

Submit a signed limit order. The `request` is typically a `SubmitOrderRequest` produced by an order envelope's `.sign()` or `.finalize()` method. **Not retried** -- non-idempotent.

### `cancel`

```rust
async fn cancel(&self, body: &CancelBody) -> Result<CancelSuccess, SdkError>
```

Cancel a single order by its hash. **Not retried.**

### `cancel_all`

```rust
async fn cancel_all(&self, body: &CancelAllBody) -> Result<CancelAllSuccess, SdkError>
```

Cancel all open orders, optionally scoped to a specific orderbook. **Not retried.**

### `submit_trigger`

```rust
async fn submit_trigger(&self, request: &impl Serialize) -> Result<TriggerOrderResponse, SdkError>
```

Submit a signed trigger order (take-profit or stop-loss). **Not retried.**

### `cancel_trigger`

```rust
async fn cancel_trigger(&self, body: &CancelTriggerBody) -> Result<CancelTriggerSuccess, SdkError>
```

Cancel a trigger order by its ID. **Not retried.**

### `get_user_orders`

```rust
async fn get_user_orders(
    &self,
    wallet_address: &str,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> Result<UserOrdersResponse, SdkError>
```

Fetch a user's orders (both limit and trigger) with cursor-based pagination.

### On-Chain Transaction Builders

#### `cancel_order_ix`

```rust
fn cancel_order_ix(
    &self,
    maker: &Pubkey,
    market: &Pubkey,
    order: &OrderPayload,
) -> Result<Transaction, SdkError>
```

Build a CancelOrder transaction for on-chain order cancellation.

#### `increment_nonce_ix`

```rust
fn increment_nonce_ix(&self, user: &Pubkey) -> Result<Transaction, SdkError>
```

Build an IncrementNonce transaction — invalidates all orders with a nonce lower than the new value.

### Order Helpers

#### `create_bid_order` / `create_ask_order`

```rust
fn create_bid_order(&self, params: BidOrderParams) -> OrderPayload
fn create_ask_order(&self, params: AskOrderParams) -> OrderPayload
```

Create unsigned bid or ask orders from raw parameters.

#### `create_signed_bid_order` / `create_signed_ask_order`

```rust
fn create_signed_bid_order(&self, params: BidOrderParams, keypair: &Keypair) -> OrderPayload
fn create_signed_ask_order(&self, params: AskOrderParams, keypair: &Keypair) -> OrderPayload
```

Create and sign orders in one step. Requires the `native-auth` feature.

#### `hash_order`

```rust
fn hash_order(&self, order: &OrderPayload) -> [u8; 32]
```

Compute the Keccak256 hash of an order (excludes the signature field).

#### `sign_order`

```rust
fn sign_order(&self, order: &mut OrderPayload, keypair: &Keypair)
```

Sign an order in place with the given keypair. Requires the `native-auth` feature.

## Order Envelope Builder

The SDK provides a fluent builder API for constructing and signing orders. The envelope handles field validation, price/size scaling to raw amounts, and signature generation.

### `LimitOrderEnvelope`

For standard limit orders:

```rust
use lightcone::prelude::*;

let request = LimitOrderEnvelope::new()
    .maker(keypair.pubkey())
    .market(market_pubkey)
    .base_mint(base_mint)
    .quote_mint(quote_mint)
    .bid()                          // or .ask()
    .price("0.55")                  // human-readable price
    .size("100")                    // human-readable size
    .nonce(nonce)
    .expiration(0)                  // 0 = no expiration
    // .deposit_source(DepositSource::Global) // omit for auto
    .apply_scaling(&decimals)?      // convert to raw amounts
    .sign(&keypair, orderbook_id)?; // sign and produce SubmitOrderRequest
```

### `TriggerOrderEnvelope`

For take-profit and stop-loss orders:

```rust
use lightcone::prelude::*;

let request = TriggerOrderEnvelope::new()
    .maker(keypair.pubkey())
    .market(market_pubkey)
    .base_mint(base_mint)
    .quote_mint(quote_mint)
    .bid()
    .price("0.55")
    .size("100")
    .nonce(nonce)
    .take_profit(0.65)              // or .stop_loss(0.45)
    .gtc()                          // or .ioc(), .fok(), .alo()
    .apply_scaling(&decimals)?
    .sign(&keypair, orderbook_id)?;
```

### `OrderEnvelope` trait

Both envelope types implement the `OrderEnvelope` trait with these shared methods:

| Method | Description |
|--------|-------------|
| `.new()` | Create a new envelope |
| `.maker(pubkey)` | Set the maker public key |
| `.market(pubkey)` | Set the market public key |
| `.base_mint(pubkey)` | Set the base token mint |
| `.quote_mint(pubkey)` | Set the quote token mint |
| `.bid()` / `.ask()` | Set the order side |
| `.price(str)` | Set the human-readable price |
| `.size(str)` | Set the human-readable size |
| `.nonce(u32)` | Set the order nonce |
| `.expiration(i64)` | Set expiration (0 = none) |
| `.deposit_source(ds)` | Set collateral source (`Auto`, `Global`, `Market`) |
| `.apply_scaling(&decimals)` | Convert price/size to raw amounts using orderbook decimals |
| `.sign(&keypair, orderbook_id)` | Sign with a keypair and produce `SubmitOrderRequest` |
| `.finalize(sig_bs58, orderbook_id)` | Attach an external signature (for Privy/wallet adapter) |
| `.payload()` | Get the raw `OrderPayload` (for manual signing) |

`TriggerOrderEnvelope` adds:

| Method | Description |
|--------|-------------|
| `.take_profit(price)` | Set trigger type to take-profit at the given price |
| `.stop_loss(price)` | Set trigger type to stop-loss at the given price |
| `.gtc()` / `.ioc()` / `.fok()` / `.alo()` | Set time-in-force |

### Scaling

`apply_scaling()` converts human-readable price and size strings into the raw `amount_in` / `amount_out` values the matching engine expects. You **must** call this before signing. The `DecimalsResponse` from `client.orderbooks().decimals()` provides the required precision.

## State Containers

### `UserOpenOrders`

Tracks a user's open limit orders grouped by market pubkey. Updated from WebSocket user events.

| Method | Description |
|--------|-------------|
| `new()` | Create empty tracker |
| `get(&market_pubkey)` | Get orders for a specific market |
| `upsert(&order_update)` | Insert or update an order from a WS event |
| `remove(order_hash)` | Remove a cancelled/filled order |
| `clear()` | Remove all tracked orders |

### `UserTriggerOrders`

Tracks trigger orders grouped by orderbook ID.

| Method | Description |
|--------|-------------|
| `new()` | Create empty tracker |
| `get(&orderbook_id)` | Get trigger orders for an orderbook |
| `get_by_id(trigger_order_id)` | Find a specific trigger order |
| `insert(order)` | Add a trigger order |
| `remove(trigger_order_id)` | Remove a trigger order |
| `all()` | Iterator over all trigger orders |
| `len()` / `is_empty()` | Count helpers |

## Examples

### Full order lifecycle

```rust
use lightcone::prelude::*;
use lightcone::auth::native::sign_login_message;
use solana_keypair::Keypair;
use futures_util::StreamExt;

async fn market_make(client: &LightconeClient, keypair: &Keypair) -> Result<(), SdkError> {
    // 1. Authenticate
    let nonce = client.auth().get_nonce().await?;
    let signed = sign_login_message(keypair, &nonce);
    client.auth().login_with_message(
        &signed.message, &signed.signature_bs58, &signed.pubkey_bytes, None,
    ).await?;

    // 2. Find a market and its orderbook
    let market = client.markets().get_by_slug("btc-above-100k").await?;
    let ob = &market.orderbook_pairs[0];
    let decimals = client.orderbooks().decimals(ob.orderbook_id.as_str()).await?;

    // 3. Place a bid
    let order_nonce = 1u32;
    let bid_request = LimitOrderEnvelope::new()
        .maker(keypair.pubkey())
        .market(market.pubkey.to_pubkey().unwrap())
        .base_mint(ob.base.mint.to_pubkey().unwrap())
        .quote_mint(ob.quote.mint.to_pubkey().unwrap())
        .bid()
        .price("0.50")
        .size("100")
        .nonce(order_nonce)
        .apply_scaling(&decimals)?
        .sign(keypair, ob.orderbook_id.as_str())?;

    let response = client.orders().submit(&bid_request).await?;
    println!("Bid placed: {:?}", response);

    // 4. Monitor via WebSocket
    let mut ws = client.ws_native();
    ws.connect().await.unwrap();
    ws.subscribe(SubscribeParams::User {
        wallet_address: PubkeyStr::from(keypair.pubkey()),
    }).unwrap();

    let mut open_orders = UserOpenOrders::new();
    let mut stream = ws.events();

    while let Some(event) = stream.next().await {
        match event {
            WsEvent::Message(Kind::User(UserUpdate::Order(OrderEvent::Limit(update)))) => {
                open_orders.upsert(&update);
                println!("Order update: {} -> {:?}", update.order.order_hash, update.order.status);
            }
            _ => {}
        }
    }

    // 5. Cancel all orders
    client.orders().cancel_all(&CancelAllBody {
        wallet_address: keypair.pubkey().to_string(),
        orderbook_id: None,
    }).await?;

    Ok(())
}
```

### Place a take-profit trigger order

```rust
use lightcone::prelude::*;

async fn place_take_profit(
    client: &LightconeClient,
    keypair: &solana_keypair::Keypair,
    ob: &OrderBookPair,
    decimals: &impl lightcone::shared::scaling::OrderbookDecimals,
) -> Result<(), SdkError> {
    let request = TriggerOrderEnvelope::new()
        .maker(keypair.pubkey())
        .market(ob.market_pubkey.to_pubkey().unwrap())
        .base_mint(ob.base.mint.to_pubkey().unwrap())
        .quote_mint(ob.quote.mint.to_pubkey().unwrap())
        .ask()
        .price("0.70")
        .size("50")
        .nonce(2)
        .take_profit(0.65)
        .gtc()
        .apply_scaling(decimals)?
        .sign(keypair, ob.orderbook_id.as_str())?;

    let response = client.orders().submit_trigger(&request).await?;
    println!("Trigger order placed: {:?}", response);
    Ok(())
}
```

## Wire Types

Raw types in `lightcone::domain::order::wire` include `OrderUpdate`, `UserUpdate`, `UserSnapshot`, `UserSnapshotOrder`, `TriggerOrderUpdate`, `OrderEvent`, `ConditionalBalance`, `GlobalDepositBalance`, and `AuthUpdate`. These are the WebSocket and REST wire formats before domain conversion.

---

[← Overview](../../../README.md#orders)
