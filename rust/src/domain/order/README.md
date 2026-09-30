# Orders

Submit, cancel, and track ordinary signed orders. REST and WebSocket collections expose supported limit-order entries under the [shared response contract](../../../../README.md#supported-order-responses).

[← Overview](../../../README.md#orders)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [Order Envelope Builder](#order-envelope-builder)
- [State Containers](#state-containers)
- [Examples](#examples)
- [Wire Types](#wire-types)

## Types

### `Order` trait

Interface implemented by `LimitOrder` and its `AnyOrder` table wrapper:

| Method | Returns | Description |
|--------|---------|-------------|
| `id()` | `&str` | Unique order hash |
| `order_hash()` | `&str` | Underlying order hash |
| `market_pubkey()` | `&PubkeyStr` | Parent market |
| `orderbook_id()` | `&OrderBookId` | Which orderbook |
| `side()` | `Side` | `Bid` or `Ask` |
| `created_at()` | `DateTime<Utc>` | Creation timestamp |

Also implemented on `AnyOrder` (delegates to the inner variant).

### `LimitOrder`

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
| `Split` | Create a complete conditional-token set |
| `Merge` | Combine a complete conditional-token set into its deposit asset |

### `TimeInForce`

Execution policy for signed orders. For market-style execution, set `TimeInForce::Ioc` through `LimitOrderEnvelope::time_in_force`. The SDK does not select IOC automatically.

| Variant | Serializes as | Description |
|---------|---------------|-------------|
| `Gtc` | `"GTC"` | Good-til-cancelled (default) |
| `Ioc` | `"IOC"` | Immediate-or-cancel |
| `Fok` | `"FOK"` | Fill-or-kill |
| `Alo` | `"ALO"` | Add-liquidity-only (post-only) |

### `UserOrderFill`

An order the user participated in (as maker or taker), with nested fill events.

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | `String` | Unique order identifier |
| `market_pubkey` | `PubkeyStr` | Parent market |
| `orderbook_id` | `OrderBookId` | Which orderbook |
| `side` | `Side` | User's side: `Bid` or `Ask` |
| `role` | `Role` | `Maker` or `Taker` |
| `price` | `Decimal` | Order price |
| `size` | `Decimal` | Total order size |
| `filled_size` | `Decimal` | Amount filled |
| `remaining_size` | `Decimal` | Amount remaining |
| `base_mint` | `PubkeyStr` | Base token mint |
| `quote_mint` | `PubkeyStr` | Quote token mint |
| `outcome_index` | `i16` | Which outcome |
| `status` | `OrderStatus` | `Filled`, `Cancelled`, or partially filled |
| `created_at` | `DateTime<Utc>` | Order creation timestamp |
| `fills` | `Vec<OrderFillEvent>` | Individual fill events |

### `OrderFillEvent`

| Field | Type | Description |
|-------|------|-------------|
| `fill_amount` | `Decimal` | Amount filled in this event |
| `tx_signature` | `String` | On-chain transaction signature |
| `filled_at` | `DateTime<Utc>` | When the fill occurred |

### `Role`

| Variant | Description |
|---------|-------------|
| `Maker` | User placed the order |
| `Taker` | User filled against the order |

### `SubmitOrderStatus`

Status of a successfully submitted order.

| Variant | Serializes as | Description |
|---------|---------------|-------------|
| `Accepted` | `"accepted"` | Order resting on the book, no immediate fills |
| `PartialFill` | `"partial_fill"` | Order partially filled, remainder resting |
| `Filled` | `"filled"` | Order fully filled immediately |

### `SubmitOrderResponse`

Response from a successful order submission.

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | `String` | Unique order identifier |
| `status` | `SubmitOrderStatus` | Outcome of the submission |
| `remaining` | `Decimal` | Remaining size after any immediate fills |
| `filled` | `Decimal` | Size filled immediately |
| `fills` | `Vec<FillInfo>` | Details of each immediate fill |

### `FillInfo`

| Field | Type | Description |
|-------|------|-------------|
| `counterparty` | `PubkeyStr` | Counterparty maker pubkey |
| `counterparty_order_hash` | `String` | Hash of the matched order |
| `fill_amount` | `Decimal` | Amount filled |
| `price` | `Decimal` | Effective fill price |
| `is_maker` | `bool` | Whether this order was the maker |

## Client Methods

Access via `client.orders()`.

### `limit_order`

```rust
async fn limit_order(&self) -> LimitOrderEnvelope
```

Create a `LimitOrderEnvelope` pre-seeded with the client's deposit source. Users can still override the deposit source on the returned envelope by calling `.deposit_source()` before signing.

### `submit`

```rust
async fn submit(&self, request: &SubmitOrderRequest) -> Result<SubmitOrderResponse, SdkError>
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

`CancelAllBody` must include:
- `orderbook_id` in the signed message, using `""` to mean all markets
- `salt`, a unique UUID-like string for replay protection

### `get_user_orders`

```rust
async fn get_user_orders(
    &self,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> Result<UserOrdersResponse, SdkError>
```

Fetch the authenticated user's supported open orders with cursor-based pagination. The server resolves the wallet from the session cookie. Refer to `get_user_orders_with_cookies` for server-side cookie forwarding.

### `get_user_order_fills`

```rust
async fn get_user_order_fills(
    &self,
    market_pubkey: Option<&str>,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> Result<UserOrderFillsResponse, SdkError>
```

Fetch the **authenticated** user's filled orders (with nested fill events). See `get_user_order_fills_with_cookies` for the SSR variant and `get_user_order_fills_by_wallet` for the public path-based variant.

### `get_user_orders_with_cookies` / `get_user_order_fills_with_cookies`

SSR / server-function variants — accept an explicit `auth_token: &str` instead of using the SDK's process-wide token store. Same wire contract, different credentials path. See [the top-level Authentication section](../../../README.md#authentication).

### `get_user_order_fills_by_wallet`

```rust
async fn get_user_order_fills_by_wallet(
    &self,
    wallet_address: &str,
    market_pubkey: Option<&str>,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> Result<UserOrderFillsResponse, SdkError>
```

Public path-based variant. Hits `GET /api/users/{wallet_address}/order-fills` and requires no auth.

Fetch a user's filled orders with nested fill events. Includes orders where the user was either maker or taker. Optionally filter by market. Returns orders sorted by most recent fill first.

### On-Chain Instruction & Transaction Builders

Each operation has an `_ix` method returning an `Instruction` and a `_tx` convenience method returning `Result<V1Transaction, SdkError>`.

#### `cancel_order_ix` / `cancel_order_tx`

```rust
fn cancel_order_ix(&self, maker: &Pubkey, market: &Pubkey, order: &OrderPayload) -> Instruction
fn cancel_order_tx(&self, maker: &Pubkey, market: &Pubkey, order: &OrderPayload, context: &V1TransactionContext) -> Result<V1Transaction, SdkError>
```

Build a CancelOrder instruction/transaction for on-chain order cancellation.

#### `increment_nonce_ix` / `increment_nonce_tx`

```rust
fn increment_nonce_ix(&self, user: &Pubkey) -> Instruction
fn increment_nonce_tx(&self, user: &Pubkey, context: &V1TransactionContext) -> Result<V1Transaction, SdkError>
```

Build an IncrementNonce instruction/transaction — invalidates all orders with a nonce lower than the new value.

#### `close_order_status_ix` / `close_order_status_tx`

```rust
fn close_order_status_ix(&self, params: &CloseOrderStatusParams) -> Instruction
fn close_order_status_tx(&self, params: CloseOrderStatusParams, context: &V1TransactionContext) -> Result<V1Transaction, SdkError>
```

Build a CloseOrderStatus instruction/transaction — close a fully-filled order status PDA.

### Order Helpers

#### `create_bid_order` / `create_ask_order`

```rust
fn create_bid_order(&self, params: BidOrderParams) -> OrderPayload
fn create_ask_order(&self, params: AskOrderParams) -> OrderPayload
```

Create unsigned bid or ask orders from raw parameters.

#### `create_signed_bid_order` / `create_signed_ask_order`

```rust
fn create_signed_bid_order(
    &self,
    params: BidOrderParams,
    keypair: &Keypair,
    rules: &OrderbookRules,
) -> SdkResult<OrderPayload>
fn create_signed_ask_order(
    &self,
    params: AskOrderParams,
    keypair: &Keypair,
    rules: &OrderbookRules,
) -> SdkResult<OrderPayload>
```

Preflight and sign orders in one step. Requires fetched trading rules and the
`native-auth` feature.

#### `hash_order`

```rust
fn hash_order(&self, order: &OrderPayload) -> [u8; 32]
```

Compute the Keccak256 hash of an order (excludes the signature field).

#### `sign_order`

```rust
fn sign_order(
    &self,
    order: &mut OrderPayload,
    keypair: &Keypair,
    rules: &OrderbookRules,
) -> SdkResult<()>
```

Preflight the raw ratio against fetched rules, then sign in place. Requires the
`native-auth` feature.

## Order Envelope Builder

The SDK provides a fluent builder API for constructing and signing orders. The
envelope uses fetched immutable trading rules and exact integer arithmetic. It
never rounds, truncates, or aligns a signed price or size implicitly.

### `LimitOrderEnvelope`

For standard limit orders:

```rust
use lightcone::prelude::*;

// Recommended: factory method pre-seeds client deposit source
let response = client.orders().limit_order().await
    .maker(keypair.pubkey())
    .market(market_pubkey)
    .base_mint(base_mint)
    .quote_mint(quote_mint)
    .bid()                          // or .ask()
    .price("0.55")                  // human-readable price
    .size("100")                    // human-readable size
    .nonce(nonce)
    .expiration(0)                  // 0 = no expiration
    // .deposit_source(DepositSource::Global) // override if needed
    .submit(&client, &orderbook).await?; // fetch/cache rules, validate, sign, submit

// Alternative: standalone use without a client
let rules = client.orderbooks().decimals(orderbook.orderbook_id.as_str()).await?;
let request = LimitOrderEnvelope::new()
    .maker(keypair.pubkey())
    // ... same chain as above
    .sign(&keypair, &orderbook, &rules)?;
```

### `OrderEnvelope` trait

`LimitOrderEnvelope` implements the `OrderEnvelope` trait with these methods:

| Method | Description |
|--------|-------------|
| `.new()` | Create a new envelope (prefer factory methods) |
| `.maker(pubkey)` | Set the maker public key |
| `.market(pubkey)` | Set the market public key |
| `.base_mint(pubkey)` | Set the base token mint |
| `.quote_mint(pubkey)` | Set the quote token mint |
| `.bid()` / `.ask()` | Set the order side |
| `.price(str)` | Set the human-readable price |
| `.size(str)` | Set the human-readable size |
| `.nonce(u32)` | Set the order nonce. When using `submit()`, auto-populated from `client.order_nonce()` if omitted (falls back to 0). |
| `.expiration(i64)` | Set expiration (0 = none) |
| `.deposit_source(ds)` | Set collateral source (`Global` or `Market`). Pre-seeded by factory methods. |
| `.sign(&keypair, orderbook, rules)` | Validate exact construction, sign, and produce `SubmitOrderRequest` |
| `.finalize(sig_bs58, orderbook, rules)` | Validate and attach an external signature |
| `.submit(client, orderbook)` | Fetch/cache rules, validate before wallet signing, and submit |
| `.payload()` | Get the raw `OrderPayload` (for manual signing) |

Use `LimitOrderEnvelope::time_in_force` to select an execution policy.

### Exact construction

`client.orderbooks().decimals()` returns the mandatory `OrderbookRules`.
Human values are parsed as exact decimal strings; raw amount callers are
preflighted against the same price, size, ratio, and signed-64-bit rules. All
validation completes before hashing or invoking a wallet signer.

## State Containers

### `AnyOrder`

Table wrapper for a supported `LimitOrder`. Implements `Order` by delegating to the inner value.

```rust
pub enum AnyOrder {
    Limit(LimitOrder),
}
```

| Method | Description |
|--------|-------------|
| `vec_from(limit_orders)` | Convert supported orders to entries sorted by creation time |

### `UserOpenLimitOrders`

Tracks a user's open limit orders grouped by market pubkey and orderbook ID. Updated from WebSocket user events.

| Method | Description |
|--------|-------------|
| `new()` | Create empty tracker |
| `get(&market_pubkey, &orderbook_id)` | Get orders for a specific orderbook |
| `get_by_market(&market_pubkey)` | Get orders for a market, grouped by orderbook |
| `upsert(&order_update)` | Insert or update an order from a WS event |
| `remove(order_hash)` | Remove a cancelled/filled order |
| `clear()` | Remove all tracked orders |

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
    let market = client.markets().get(None, Some(1)).await?.markets.into_iter().next().unwrap();
    let ob = &market.orderbook_pairs[0];
    let decimals = client.orderbooks().decimals(ob.orderbook_id.as_str()).await?;

    // 3. Place a bid
    let order_nonce = 1u32;
    let bid_request = client.orders().limit_order().await
        .maker(keypair.pubkey())
        .market(market.pubkey.to_pubkey().unwrap())
        .base_mint(ob.base.mint.to_pubkey().unwrap())
        .quote_mint(ob.quote.mint.to_pubkey().unwrap())
        .bid()
        .price("0.50")
        .size("100")
        .nonce(order_nonce.into())
        .sign(keypair, ob, &decimals)?;

    let response = client.orders().submit(&bid_request).await?;
    println!("Bid placed: {:?}", response);

    // 4. Monitor via WebSocket
    let mut ws = client.ws_native();
    ws.connect().await.unwrap();
    ws.subscribe(SubscribeParams::User {
        wallet_address: PubkeyStr::from(keypair.pubkey()),
    }).unwrap();

    let mut open_orders = UserOpenLimitOrders::new();
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
        user_pubkey: keypair.pubkey().into(),
        orderbook_id: OrderBookId::from(""),
        signature: "...".into(),
        timestamp: 1_710_300_000,
        salt: generate_cancel_all_salt(),
    }).await?;

    Ok(())
}
```

### Fetch filled orders with pagination

```rust
use lightcone::prelude::*;

async fn show_fill_history(
    client: &LightconeClient,
    market_pubkey: &str,
) -> Result<(), SdkError> {
    // Authenticated user — wallet from JWT cookie. (For a public lookup of
    // another wallet, use `get_user_order_fills_by_wallet(wallet, ...)`.)
    let response = client.orders().get_user_order_fills(
        Some(market_pubkey),
        Some(20),
        None,
    ).await?;

    for order in &response.orders {
        println!("{} {} ({:?}) @ {} — {}/{} filled",
            order.side, order.role, order.status,
            order.price, order.filled_size, order.size);
        for fill in &order.fills {
            println!("  fill: {} at {}", fill.fill_amount, fill.filled_at);
        }
    }

    // Paginate
    if response.has_more {
        let next_page = client.orders().get_user_order_fills(
            Some(market_pubkey),
            Some(20),
            response.next_cursor.as_deref(),
        ).await?;
    }

    Ok(())
}
```

## Wire Types

The `wire` module provides limit-order updates, account snapshots, balances, authentication updates, and fill-history types. REST and snapshot decoding construct `UserSnapshotOrder` directly. Invalid limit payloads fail the response. Live order updates use `OrderEvent::Limit`. `convert_snapshot_orders` returns `UserOpenLimitOrders`.

---

[← Overview](../../../README.md#orders)
