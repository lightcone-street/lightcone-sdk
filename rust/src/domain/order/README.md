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

### `Order` trait

Common interface shared by `LimitOrder` and `TriggerOrder`. Provides accessors for the six fields present on both types:

| Method | Returns | Description |
|--------|---------|-------------|
| `id()` | `&str` | Unique identifier (`order_hash` for limit, `trigger_order_id` for trigger) |
| `order_hash()` | `&str` | Underlying order hash |
| `market_pubkey()` | `&PubkeyStr` | Parent market |
| `orderbook_id()` | `&OrderBookId` | Which orderbook |
| `side()` | `Side` | `Bid` or `Ask` |
| `created_at()` | `DateTime<Utc>` | Creation timestamp |

Also implemented on `AnyOrder` (delegates to the inner variant).

### `LimitOrder`

A limit order's committed state, built from the WS `user` snapshot, live `order` facts, or REST pages. Size fields are in base-token units and satisfy `size = filled_size + pending_size + remaining_size + cancelled_size`.

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | `String` | Unique order identifier |
| `market_pubkey` | `PubkeyStr` | Parent market |
| `orderbook_id` | `OrderBookId` | Which orderbook |
| `side` | `Side` | `Bid` (buy) or `Ask` (sell) |
| `price` | `Decimal` | Limit price, quote units per base unit |
| `size` | `Decimal` | Original order size |
| `filled_size` | `Decimal` | Base filled by confirmed (on-chain authenticated) executions |
| `pending_size` | `Decimal` | Base matched but awaiting on-chain confirmation |
| `remaining_size` | `Decimal` | Base still resting on the book |
| `cancelled_size` | `Decimal` | Base cancelled (explicitly, by expiry, IOC/FOK remainder, or closure) |
| `time_in_force` | `TimeInForce` | `Gtc`, `Ioc`, or `Fok` |
| `funding_source` | `FundingSource` | `Global` or `Conditional` custody account |
| `closed_reason` | `Option<String>` | Why the order stopped resting |
| `status` | `OrderStatus` | Derived committed status |
| `accepted_seq` | `u64` | Engine acceptance sequence (closure cutoffs compare against it) |
| `committed_revision` | `u64` | Revision of this state; newer revisions supersede older ones |
| `base_mint` / `quote_mint` | `PubkeyStr` | Token mints |
| `created_at` | `DateTime<Utc>` | Acceptance time |
| `expiration` | `i64` | Unix seconds; `0` = none |

`is_live()` is true while the order rests or has fills awaiting confirmation.

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

Serialized lowercase, as in `GET /api/users/order-fills`. `OrderStatus::derive` applies the backend's precedence: a closure reason wins, then a complete confirmed fill, then pending fills, otherwise open.

| Variant | Description |
|---------|-------------|
| `Open` | Resting, or awaiting its first match |
| `Pending` | Some matched base awaits on-chain confirmation |
| `Filled` | The whole original base is confirmed filled |
| `Closed` | Stopped resting (cancelled, expired, closure cutoff, IOC/FOK remainder) |

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

The backend has no post-only policy and rejects `"ALO"`.

### `TriggerType`

| Variant | Serializes as | Description |
|---------|---------------|-------------|
| `TakeProfit` | `"TP"` | Fires when price rises above trigger |
| `StopLoss` | `"SL"` | Fires when price falls below trigger |

### `UserOrderFill`

An order the wallet participated in (as maker or taker) with its oldest-first page of at most 16 fills. Quantities are base-token units.

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | `String` | Unique order identifier |
| `market_pubkey` / `orderbook_id` | | Order location |
| `side` | `Side` | The wallet's order side |
| `original_base` / `confirmed_base` / `pending_base` / `open_base` / `cancelled_base` | `Decimal` | Committed quantities |
| `confirmed_quote` | `Decimal` | Quote exchanged by confirmed fills (quote-token units) |
| `status` | `OrderStatus` | `closed`, `filled`, `pending`, or `open` |
| `closed_reason` | `Option<String>` | Why the order closed |
| `created_at` | `DateTime<Utc>` | Acceptance time |
| `fills` | `Vec<OrderFillEvent>` | First fill page |
| `fills_has_more` / `fills_next_cursor` | | Continue with `get_order_fill_page(order_hash, fill_cursor)` |

### `OrderFillEvent`

| Field | Type | Description |
|-------|------|-------------|
| `fill_id` | `String` | `"<execution_id>:<leg_index>:<projection_generation>"` (same as REST `trade_id`) |
| `counterparty` / `counterparty_order_hash` | | Other side of the fill |
| `role` | `Role` | This order's role: `Maker` or `Taker` |
| `base_amount` / `quote_amount` | `Decimal` | Filled size (base units) and notional (quote units); `price()` = quote / base |
| `fee_estimate_atoms` | `i128` | Signed fee estimate in raw `fee_mint` atoms (negative = rebate) |
| `fee_mint` | `PubkeyStr` | Fee token |
| `maker_fee_bps` / `taker_fee_bps` | `i16` | Captured fee rates |
| `tx_signature` | `String` | Settlement transaction |
| `filled_at` | `DateTime<Utc>` | Fill time |

### `SubmitOrderStatus`

| Variant | Serializes as | Description |
|---------|---------------|-------------|
| `Accepted` | `"accepted"` | Accepted, no fill awaiting confirmation (IOC with no fill: `cancelled_base == original_base`) |
| `AcceptedPending` | `"accepted_pending"` | Matched base awaits confirmation, or the committed view is not yet readable (`state`/`initial_cohort` are `None`) |
| `Filled` | `"filled"` | The whole original base is confirmed filled |

### `SubmitOrderResponse`

| Field | Type | Description |
|-------|------|-------------|
| `order_hash` | `String` | Unique order identifier |
| `status` | `SubmitOrderStatus` | Outcome of the submission |
| `state` | `Option<OrderState>` | Committed cumulative state (`original/confirmed/pending/open/executable/cancelled_base`, `ready`, `closed_reason`, `committed_revision`, `accepted_seq`) |
| `initial_cohort` | `Option<InitialCohort>` | Executions selected at acceptance (`state`, `selected_base`, `known_confirmed_base`, `known_failed_unfilled_base`, `unresolved_base`) |
| `fills` | `Vec<FillInfo>` | Up to 16 authenticated fills |
| `fills_complete` | `bool` | False when more fills exist or a newer revision prevented capture |
| `fills_next_cursor` | `Option<String>` | Continue with `get_order_fill_page` |

`filled_base()` and `open_base()` read the corresponding `state` quantities. Business rejections arrive as `SdkError::ApiRejected` with a `RejectionCode`; an `ENGINE_UNAVAILABLE` error means the outcome is unknown (see [`submit`](#submit)).

### `FillInfo`

| Field | Type | Description |
|-------|------|-------------|
| `fill_id` / `execution_id` | `String` | Fill identity |
| `counterparty` / `counterparty_order_hash` | | Other side of the fill |
| `base_amount` / `quote_amount` | `Decimal` | Base-unit size and quote-unit notional; `price()` = quote / base |
| `is_maker` | `bool` | Whether the submitted order was the maker |
| `fee_estimate_atoms` | `i128` | Signed fee estimate in raw `fee_mint` atoms |
| `fee_bps` | `i32` | Captured fee rate |
| `fee_mint` | `PubkeyStr` | Fee token |

### `CancelSuccess` / `CancelAllSuccess`

`CancelSuccess` has `status` (`Cancelled`, `AlreadyClosed`, `AlreadyFilled`), `order_hash`, `quantities: Option<CancelQuantities>` (raw base atoms: `newly_cancelled_base`, `confirmed_base`, `pending_base`, `remaining_open_base`), `quantity_unit` (`"base_atoms"`), `revision`, and `closed_reason`. An unknown hash is rejected with `RejectionCode::OrderNotFound`.

`CancelAllSuccess` has `status`, `user_pubkey`, `orderbook_id`, `message`, and `closure: Option<ClosureAck>` (`operation_id`, `committed_revision`, `scope` — `"Wallet:<w>"` or `"WalletBook:<w>:<book>"` — `accepted_seq_cutoff`, `cleanup_pending`). It commits an accepted-order cutoff; per-order facts follow on the WS `user` channel.

## Client Methods

Access via `client.orders()`.

### `limit_order`

```rust
async fn limit_order(&self) -> LimitOrderEnvelope
```

Create a `LimitOrderEnvelope` pre-seeded with the client's deposit source. Users can still override the deposit source on the returned envelope by calling `.deposit_source()` before signing.

### `trigger_order`

```rust
async fn trigger_order(&self) -> TriggerOrderEnvelope
```

Create a `TriggerOrderEnvelope` pre-seeded with the client's deposit source. Users can still override the deposit source on the returned envelope by calling `.deposit_source()` before signing.

### `submit`

```rust
async fn submit(&self, request: &SubmitOrderRequest) -> Result<SubmitOrderResponse, SdkError>
```

Submit a signed limit order, typically the `SubmitOrderRequest` produced by an order envelope's `.sign()` or `.finalize()` method. **Not retried.**

An `ENGINE_UNAVAILABLE` error means the outcome is unknown. Keep the signed `SubmitOrderRequest` (it is `Clone`) and submit that identical request again: a `DUPLICATE_ORDER` rejection or an `ALREADY_EXISTS` (409) error proves the first submission was accepted, and any other result is the outcome of this one. Never sign a replacement with a new salt, which is a different order. `get_user_orders` lists only open and pending orders, so it cannot prove an order was never accepted.

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

### `submit_trigger`

```rust
async fn submit_trigger(&self, request: &SubmitTriggerOrderRequest) -> Result<TriggerOrderResponse, SdkError>
```

Submit a signed trigger order (take-profit or stop-loss). `SubmitTriggerOrderRequest` flattens the limit-order `SubmitOrderRequest` and adds `trigger_price` and `trigger_type`; the plain limit request never carries them. The current backend has no trigger orders and rejects this request. **Not retried.**

### `cancel_trigger`

```rust
async fn cancel_trigger(&self, body: &CancelTriggerBody) -> Result<CancelTriggerSuccess, SdkError>
```

Cancel a trigger order by its ID. **Not retried.**

### `get_user_orders`

```rust
async fn get_user_orders(
    &self,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> Result<UserOrdersResponse, SdkError>
```

Fetch the **authenticated** user's open and pending orders (`UserOrder`, with nested `RecordedOrderState`) plus a first page of `FundingAccount`s, with cursor-based pagination (`limit` defaults to 200, clamped to 1..=256; a page can be empty while `has_more` is true). Convert entries with `UserOrder::into_limit_order()`. Continue funding pages with `positions().positions_page(None, next_funding_cursor, ..)`. See `get_user_orders_with_cookies` for the SSR variant.

### `get_user_order_fills`

```rust
async fn get_user_order_fills(
    &self,
    market_pubkey: Option<&str>,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> Result<UserOrderFillsResponse, SdkError>
```

Fetch the **authenticated** user's filled orders (with nested fill events; `limit` clamped to 1..=100). See `get_user_order_fills_with_cookies` for the SSR variant and `get_user_order_fills_by_wallet` for the public path-based variant.

### `get_order_fill_page`

```rust
async fn get_order_fill_page(
    &self,
    order_hash: &str,
    fill_cursor: Option<&str>,
) -> Result<UserOrderFillsResponse, SdkError>
```

Fetch one order with the next page of its fills (continue a submission's or an order's `fills_next_cursor`). `order_hash` must be 64 lowercase hex characters. Variants: `get_order_fill_page_with_cookies` and the public `get_order_fill_page_by_wallet`.

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

Compute the Keccak256 hash of an order's 161-byte signing preimage (salt first; the signature is excluded). There is no per-user nonce, so the salt is the order's only identity.

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

### `TriggerOrderEnvelope`

> **Feature-gated** (`trigger_orders`): The types and methods below require the `trigger_orders` Cargo feature, which is disabled by default. Trigger orders are under development and not yet available. For internal use only.

For take-profit and stop-loss orders:

```rust
use lightcone::prelude::*;

// Recommended: factory method pre-seeds client deposit source
let request = client.orders().trigger_order().await
    .maker(keypair.pubkey())
    .market(market_pubkey)
    .base_mint(base_mint)
    .quote_mint(quote_mint)
    .bid()
    .price("0.55")
    .size("100")
    .take_profit("0.65")            // exact decimal string; or .stop_loss("0.45")
    .gtc()                          // or .ioc(), .fok()
    // .deposit_source(DepositSource::Global) // override if needed
    .submit(&client, &orderbook).await?;

// Alternative: standalone use without a client
let request = TriggerOrderEnvelope::new()
    .maker(keypair.pubkey())
    // ... same chain as above
    .sign(&keypair, &orderbook, &rules)?;
```

### `OrderEnvelope` trait

Both envelope types implement the `OrderEnvelope` trait with these shared methods:

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
| `.salt(u64)` | Set the order's identity salt (any u64). When omitted, a random salt is drawn on the first `payload()`, `sign()`, `finalize()`, or `submit()` and reused, so the hash a wallet signs is the order submitted. |
| `.expiration(i64)` | Set expiration (0 = none) |
| `.deposit_source(ds)` | Set collateral source (`Global`, or `Market`, which serializes as `"conditional"`). Pre-seeded by factory methods. |
| `.sign(&keypair, orderbook, rules)` | Validate exact construction, sign, and produce the envelope's `Request` (`SubmitOrderRequest` for limit orders) |
| `.finalize(sig_bs58, orderbook, rules)` | Validate and attach an external signature |
| `.submit(client, orderbook)` | Fetch/cache rules, validate before wallet signing, and submit |
| `.payload()` | Get the raw `OrderPayload` (for manual signing) |

`TriggerOrderEnvelope` adds:

| Method | Description |
|--------|-------------|
| `.take_profit(price)` | Set trigger type to take-profit at the given price |
| `.stop_loss(price)` | Set trigger type to stop-loss at the given price |
| `.gtc()` / `.ioc()` / `.fok()` | Set time-in-force |

### Exact construction

`client.orderbooks().decimals()` returns the mandatory `OrderbookRules`.
Human values are parsed as exact decimal strings; raw amount callers are
preflighted against the same price, size, ratio, and nonzero-u64 rules. All
validation completes before hashing or invoking a wallet signer.

## State Containers

### `AnyOrder`

Enum wrapping either a `LimitOrder` or `TriggerOrder`. Implements the `Order` trait by delegating to the inner type.

```rust
pub enum AnyOrder {
    Limit(LimitOrder),
    Trigger(TriggerOrder),
}
```

| Method | Description |
|--------|-------------|
| `vec_from(limit_orders, trigger_orders)` | Combine both types into a sorted `Vec<AnyOrder>` |

### `UserOpenLimitOrders`

Tracks a wallet's live limit orders (resting, or with fills awaiting confirmation) grouped by market pubkey and orderbook ID. Seed it with `convert_snapshot_orders(snapshot.orders)`, then apply every live `order` fact: each fact carries the complete order state, so application is a revision-guarded replace. The tracker remembers the revision at which each order stopped being live, and every applied closure, so older state (a REST page or snapshot racing live facts) cannot reopen a closed order.

| Method | Description |
|--------|-------------|
| `new()` | Create empty tracker |
| `get(&market_pubkey, &orderbook_id)` | Get orders for a specific orderbook |
| `get_by_market(&market_pubkey)` | Get orders for a market, grouped by orderbook |
| `get_by_hash(order_hash)` / `all()` | Lookup and iteration |
| `apply(&order_update) -> ApplyOutcome` | Apply a live WS `order` fact (`Inserted`, `Updated`, `Removed`, `Stale`, `Ignored`) |
| `apply_order(limit_order)` | Same for converted snapshot or REST orders |
| `apply_closure(&closure_update)` | Close orders in scope up to the cutoff; `None` when the scope needs a refetch |
| `upsert(&order_update)` | Alias of `apply` |
| `remove(order_hash)` | Stop tracking an order; older state can add it again |
| `clear()` | Forget all orders, retired revisions, and closures (before reseeding) |

### `UserTriggerOrders`

Tracks trigger orders grouped by market pubkey and orderbook ID.

| Method | Description |
|--------|-------------|
| `new()` | Create empty tracker |
| `get(&market_pubkey, &orderbook_id)` | Get trigger orders for a specific orderbook |
| `get_by_market(&market_pubkey)` | Get trigger orders for a market, grouped by orderbook |
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
    let market = client.markets().get(None, Some(1)).await?.markets.into_iter().next().unwrap();
    let ob = &market.orderbook_pairs[0];
    let decimals = client.orderbooks().decimals(ob.orderbook_id.as_str()).await?;

    // 3. Place a bid
    let bid_request = client.orders().limit_order().await
        .maker(keypair.pubkey())
        .market(market.pubkey.to_pubkey().unwrap())
        .base_mint(ob.base.mint.to_pubkey().unwrap())
        .quote_mint(ob.quote.mint.to_pubkey().unwrap())
        .bid()
        .price("0.50")
        .size("100")
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
            WsEvent::Message(Kind::User(UserUpdate::Snapshot(snapshot))) => {
                open_orders = convert_snapshot_orders(snapshot.orders);
            }
            WsEvent::Message(Kind::User(UserUpdate::Order(update))) => {
                let outcome = open_orders.apply(&update);
                println!("Order {}: open={} ({outcome:?})", update.order_hash, update.open_base);
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

### Place a take-profit trigger order

```rust
use lightcone::prelude::*;

async fn place_take_profit(
    client: &LightconeClient,
    keypair: &solana_keypair::Keypair,
    ob: &OrderBookPair,
) -> Result<(), SdkError> {
    let rules = client.orderbooks().decimals(ob.orderbook_id.as_str()).await?;
    let request = client.orders().trigger_order().await
        .maker(keypair.pubkey())
        .market(ob.market_pubkey.to_pubkey().unwrap())
        .base_mint(ob.base.mint.to_pubkey().unwrap())
        .quote_mint(ob.quote.mint.to_pubkey().unwrap())
        .ask()
        .price("0.70")
        .size("50")
        .take_profit("0.65")
        .gtc()
        .sign(keypair, ob, &rules)?;

    let response = client.orders().submit_trigger(&request).await?;
    println!("Trigger order placed: {:?}", response);
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
        println!("{} ({:?}) — {}/{} confirmed",
            order.side, order.status, order.confirmed_base, order.original_base);
        for fill in &order.fills {
            println!("  {:?} fill: {} @ {:?} at {}",
                fill.role, fill.base_amount, fill.price(), fill.filled_at);
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

Raw types in `lightcone::domain::order::wire` include `OrderState`, `RecordedOrderState`, `InitialCohort`, `UserOrder`, `UserOrderFillsResponse`, `UserOrderFill`, `OrderFillEvent`, `Role`, and the WS `user` channel types `UserUpdate`, `UserSnapshot`, `UserSnapshotOrder`, `OrderUpdate`, `ClosureUpdate`, `RecoveryCompleted`, `NotificationUpdate`, `CommitInfo`, and `AuthUpdate`. Funding types (`FundingAccount`, `FundingUpdate`, `FundingSource`) live in `lightcone::domain::position::wire`. Integer revisions and sequences accept both the string (REST/snapshot) and number (live fact) encodings.

---

[← Overview](../../../README.md#orders)
