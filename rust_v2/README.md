# Lightcone SDK - Rust

A Rust SDK for trading on Lightcone, the impact market protocol.

## Table of contents
- [Overview](#overview)
- [Getting Started](#getting-started)
     - [Installation](#installation)
     - [Feature Flags](#feature-flags)
- [Trading Flow](#trading-flow)
     - [Step 1: Setup](#step-1-setup)
     - [Step 2: Find a Market](#step-2-find-a-market)
     - [Step 3: Deposit Collateral](#step-3-deposit-collateral)
     - [Step 4: Place an Order](#step-4-place-an-order)
     - [Step 5: Monitor](#step-5-monitor)
     - [Step 6: Cancel Orders](#step-6-cancel-orders)
     - [Step 7: Exit a Position](#step-7-exit-a-position)
- [Decimal Scaling](#decimal-scaling)
- [Nonce Management](#nonce-management)
- [Order Expiration](#order-expiration)
- [Authentication](#authentication)
- [Error Handling](#error-handling)
- [API Reference](#api-reference)
     - [REST Sub-Clients](#rest-sub-clients)
     - [WebSocket Streaming](#websocket-streaming)
     - [Program (On-Chain)](#program-on-chain)
- [Examples](#examples)

---
## Overview
Lightcone is an **impact market** protocol for trading assets inside hypothetical universes. The high-level documentation is available [here](https://lightconelabs.mintlify.app/learn/about-lightcone/the-trading) - understanding the mechanism will provide important context for interacting with the SDK.

Lightcone runs a hybrid-decentralised CLOB. Orders are Ed25519-signed payloads submitted to a centralised operator that handles matching. Settlement, token movement, and signature verification all happen on-chain. The SDK provides two entry points:

- **`LightconeClient`** - REST API and WebSocket streaming. Market discovery, order submission/cancellation, orderbook data, positions, trade history, and real-time event feeds.
- **`LightconePinocchioClient`** - Direct on-chain interaction. Deposit/withdraw collateral, mint/merge conditional tokens, redeem winnings, read on-chain state, and manage nonces.

The prelude (`use lightcone_sdk_v2::prelude::*`) exports the client, all domain types, WS message types, state containers, and errors.

---
## Getting Started
### Installation
```
cargo add lightcone-sdk-v2 --features native
```
or add the crate to `Cargo.toml`:
```toml
[dependencies]
lightcone-sdk-v2 = { version = "0.2", features = ["native"] }
```

The `native` feature flag bundles everything needed for server-side use: REST client, keypair-based signing, WebSocket via `tokio-tungstenite`, and Solana RPC access.

### Feature Flags
By default only `http` is enabled. The `native` bundle is recommended for server-side / CLI use:

| Feature       | Description                                               |
|---------------|-----------------------------------------------------------|
| `http`        | REST API client (default)                                 |
| `native-auth` | Keypair-based signing (`solana-keypair`, `solana-signer`) |
| `ws-native`   | WebSocket via `tokio-tungstenite`                         |
| `ws-wasm`     | WebSocket via `web-sys` (browser WASM)                    |
| `solana-rpc`  | On-chain RPC client (`solana-client`)                     |
| `native`      | Bundle: http + native-auth + ws-native + solana-rpc       |
| `wasm`        | Bundle: http + ws-wasm                                    |

---
## Trading Flow

The complete flow from first connection to first trade:

```
1. Setup        →  Create clients (REST + on-chain)
2. Find market  →  Discover a market and its orderbook
3. Deposit      →  Mint conditional tokens from collateral (on-chain)
4. Place order  →  Build, sign, and submit an order
5. Monitor      →  Track fills via REST or WebSocket
6. Cancel       →  Cancel open orders
7. Exit         →  Merge tokens back to collateral or redeem winnings
```

### Step 1: Setup

```rust
use lightcone_sdk_v2::prelude::*;
use lightcone_sdk_v2::program::builder::OrderBuilder;
use lightcone_sdk_v2::program::client::LightconePinocchioClient;
use lightcone_sdk_v2::program::types::*;
use lightcone_sdk_v2::shared::OrderbookDecimals;
use solana_signer::Signer;
use chrono::Utc;

let client = LightconeClient::builder().build()?;
let program = LightconePinocchioClient::new(&rpc_url);
```

`LightconeClient` handles all REST and WebSocket interaction. `LightconePinocchioClient` handles on-chain operations (deposits, withdrawals, nonce management) and requires a Solana RPC URL. All subsequent steps use these two clients.

### Step 2: Find a Market

```rust
let markets = client.markets().get(None, Some(1)).await?;
let market = markets.markets.into_iter().next().expect("no markets");

let pair = market.orderbook_pairs.first().expect("no orderbooks");
let orderbook_id = &pair.orderbook_id;
let base_mint = pair.base.pubkey().to_pubkey()?;
let quote_mint = pair.quote.pubkey().to_pubkey()?;

let market_pubkey = market.pubkey.to_pubkey()?;
let deposit_mint = market.deposit_assets.first().expect("no deposit assets").deposit_asset.to_pubkey()?;
let num_outcomes = market.outcomes.len() as u8;
```

A `Market` contains everything needed: the on-chain pubkey, orderbook pairs (with base/quote mints), deposit assets (collateral mints), and outcome definitions.

### Step 3: Deposit Collateral

Trading requires conditional tokens - one per outcome. These are obtained by depositing collateral (e.g. USDC) via `mint_complete_set`, which returns one of each outcome token in a single transaction.

```rust
let params = MintCompleteSetParams {
    user: keypair.pubkey(),
    market: market_pubkey,
    deposit_mint,
    amount: 1_000_000, // raw lamports (e.g. 1 USDC = 1_000_000 with 6 decimals)
};
let mut tx = program.mint_complete_set(params, num_outcomes).await?;
let blockhash = program.get_latest_blockhash().await?;
tx.partial_sign(&[&keypair], blockhash);
let sig = program.rpc_client.send_and_confirm_transaction(&tx).await?;
```

After this transaction confirms, the position account holds conditional tokens that can be placed on the orderbook.

### Step 4: Place an Order

Fetch decimals for price/size scaling, get a valid nonce, then build, sign, and submit.

```rust
// Fetch decimals (cached after first call)
let dec = client.orderbooks().decimals(orderbook_id.as_str()).await?;
let decimals = OrderbookDecimals {
    orderbook_id: orderbook_id.as_str().to_string(),
    base_decimals: dec.base_decimals,
    quote_decimals: dec.quote_decimals,
    price_decimals: dec.price_decimals,
};

// Get current nonce (orders must be signed with nonce >= on-chain value)
let nonce = program.get_current_nonce(&keypair.pubkey()).await?;

// Build, sign, and submit
let request = OrderBuilder::new()
    .nonce(nonce)
    .maker(keypair.pubkey())
    .market(market_pubkey)
    .base_mint(base_mint)
    .quote_mint(quote_mint)
    .bid()
    .price("0.65")
    .size("100")
    .apply_scaling(&decimals)?
    .to_submit_request(&keypair, orderbook_id.as_str())?;

let response = client.orders().submit(&request).await?;
// response.order_hash, response.filled, response.remaining, response.fills
```

The `SubmitOrderResponse` contains:
- `order_hash: String` - unique identifier for the order
- `filled: String` / `remaining: String` - how much was immediately filled vs resting on the book (string-encoded amounts)
- `fills: Vec<FillInfo>` - array of fill details: `counterparty`, `counterparty_order_hash`, `fill_amount`, `price` (all `String`), and `is_maker: bool`

### Step 5: Monitor

Both methods require [authentication](#authentication):

```rust
use lightcone_sdk_v2::auth::native::sign_login_message;

let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
let signed = sign_login_message(&keypair, timestamp);
client.auth().login_with_message(
    &signed.message, &signed.signature_bs58, &signed.pubkey_bytes, None,
).await?;
```

**Via REST** - poll open orders:

```rust
let orders = client.orders().get_user_orders(
    &keypair.pubkey().to_string(),
    Some(100),
    None,
).await?;
```

**Via WebSocket** - stream fills and balance changes in real-time:

```rust
use futures_util::StreamExt;

let mut ws = client.ws_native();
ws.connect().await?;
ws.send(MessageOut::subscribe_user(PubkeyStr::new(&keypair.pubkey().to_string())))?;

let mut stream = ws.events();
while let Some(event) = stream.next().await {
    match event {
        WsEvent::Message(Kind::User(update)) => {
            println!("{:?}", update); // Snapshot, OrderUpdate, or BalanceUpdate
        }
        _ => {}
    }
}
```

### Step 6: Cancel Orders

**Cancel a single order** by its hash:

```rust
let cancel = CancelBody::signed(
    order_hash.clone(),
    keypair.pubkey().to_string(),
    &keypair,
);
let result = client.orders().cancel(&cancel).await?;
// result.order_hash, result.remaining
```

**Cancel all orders** in an orderbook:

```rust
let cancel_all = CancelAllBody::signed(
    keypair.pubkey().to_string(),
    orderbook_id.as_str().to_string(),
    chrono::Utc::now().timestamp(),
    &keypair,
);
let result = client.orders().cancel_all(&cancel_all).await?;
// result.count, result.cancelled_order_hashes, result.user_pubkey, result.orderbook_id, result.message
```

**Mass cancel via nonce increment** - invalidates all orders signed with a nonce below the new value. See [Nonce Management](#nonce-management).

```rust
let mut tx = program.increment_nonce(&keypair.pubkey()).await?;
let blockhash = program.get_latest_blockhash().await?;
tx.partial_sign(&[&keypair], blockhash);
let sig = program.rpc_client.send_and_confirm_transaction(&tx).await?;
```

### Step 7: Exit a Position

**Merge a complete set** - burn one of each outcome token to recover collateral. This is the inverse of `mint_complete_set`:

```rust
let params = MergeCompleteSetParams {
    user: keypair.pubkey(),
    market: market_pubkey,
    deposit_mint,
    amount: 1_000_000,
};
let mut tx = program.merge_complete_set(params, num_outcomes).await?;
let blockhash = program.get_latest_blockhash().await?;
tx.partial_sign(&[&keypair], blockhash);
let sig = program.rpc_client.send_and_confirm_transaction(&tx).await?;
```

**Redeem winnings** - after a market is settled, burn winning outcome tokens for collateral:

```rust
let params = RedeemWinningsParams {
    user: keypair.pubkey(),
    market: market_pubkey,
    deposit_mint,
    amount: 1_000_000,
};
let mut tx = program.redeem_winnings(params, winning_outcome).await?;
let blockhash = program.get_latest_blockhash().await?;
tx.partial_sign(&[&keypair], blockhash);
let sig = program.rpc_client.send_and_confirm_transaction(&tx).await?;
```

**Withdraw from position** - move specific outcome tokens from the position account to a wallet:

```rust
let params = WithdrawFromPositionParams {
    user: keypair.pubkey(),
    market: market_pubkey,
    mint: deposit_mint,       // or a conditional token mint
    amount: 500_000,
    outcome_index: 0,         // 0 = first outcome, 255 = collateral
};
let mut tx = program.withdraw_from_position(params, false).await?;
let blockhash = program.get_latest_blockhash().await?;
tx.partial_sign(&[&keypair], blockhash);
let sig = program.rpc_client.send_and_confirm_transaction(&tx).await?;
```

---
## Decimal Scaling

All values submitted to the matching engine must be raw `u64` amounts (lamports), but the SDK offers automatic conversion from human-readable inputs like price=0.65 and size=100.

**The math:**
```
base_lamports  = size  * 10^base_decimals
quote_lamports = price * size * 10^quote_decimals
```

Then assign based on order side:

| Side | `amount_in` (what maker gives) | `amount_out` (what maker receives) |
|------|-------------------------------|-----------------------------------|
| BID  | quote_lamports                | base_lamports                     |
| ASK  | base_lamports                 | quote_lamports                    |

**Option 1: Auto-scaling via `OrderBuilder`** (recommended)

Fetch the decimals once, then let the builder handle conversion:

```rust
use lightcone_sdk_v2::program::builder::OrderBuilder;
use lightcone_sdk_v2::shared::OrderbookDecimals;

let dec = client.orderbooks().decimals(orderbook_id.as_str()).await?;
let decimals = OrderbookDecimals {
    orderbook_id: orderbook_id.as_str().to_string(),
    base_decimals: dec.base_decimals,
    quote_decimals: dec.quote_decimals,
    price_decimals: dec.price_decimals,
};

let request = OrderBuilder::new()
    .nonce(nonce)
    .maker(keypair.pubkey())
    .market(market_pubkey)
    .base_mint(base_mint)
    .quote_mint(quote_mint)
    .bid()
    .price("0.65")       // human-readable
    .size("100")          // human-readable
    .apply_scaling(&decimals)?  // converts to raw u64 amounts
    .to_submit_request(&keypair, orderbook_id.as_str())?;
```

**Option 2: Manual scaling**

For full control, `scale_price_size()` can be used directly:

```rust
use lightcone_sdk_v2::program::types::OrderSide;
use lightcone_sdk_v2::shared::{scale_price_size, OrderbookDecimals};
use rust_decimal::prelude::*;

let scaled = scale_price_size(
    Decimal::from_str("0.65").unwrap(),
    Decimal::from_str("100").unwrap(),
    OrderSide::Bid,
    &decimals,
)?;
// scaled.amount_in  = 65_000_000  (quote lamports)
// scaled.amount_out = 100_000_000 (base lamports)
```

**Option 3: Raw amounts**

With existing lamport values, scaling can be skipped entirely:

```rust
let request = OrderBuilder::new()
    .nonce(nonce)
    .maker(keypair.pubkey())
    .market(market_pubkey)
    .base_mint(base_mint)
    .quote_mint(quote_mint)
    .bid()
    .amount_in(65_000_000)    // raw u64
    .amount_out(100_000_000)  // raw u64
    .to_submit_request(&keypair, orderbook_id.as_str())?;
```

---
## Nonce Management

Every order is signed with a `nonce`. The on-chain program tracks a per-user nonce value - orders with a nonce **below** the on-chain value are rejected.

**Key rules:**
- **Same nonce, multiple orders:** Many orders can be signed with the same nonce. The nonce is a floor, not a sequence number.
- **Fetch before signing:** Always call `program.get_current_nonce(&pubkey)` before building orders to ensure the nonce is valid.
- **Nuclear cancel:** `program.increment_nonce()` bumps the on-chain nonce, instantly invalidating every outstanding order signed with a lower value. This is the fastest way to cancel all orders across all orderbooks.
- **Nonce too low:** Submitting an order with a nonce below the on-chain value causes the matching engine to reject it. Re-fetch the nonce and re-sign.

```rust
// Typical pattern: fetch nonce once, reuse for a batch of quotes
let nonce = program.get_current_nonce(&keypair.pubkey()).await?;

let bid = OrderBuilder::new().nonce(nonce).bid().price("0.60").size("100");
let ask = OrderBuilder::new().nonce(nonce).ask().price("0.70").size("100");

// Emergency: pull all quotes instantly
let mut tx = program.increment_nonce(&keypair.pubkey()).await?;
```

---
## Order Expiration

Orders default to no expiration (GTC). A TTL can be set using `.expiration()`:

```rust
let request = OrderBuilder::new()
    .nonce(nonce)
    .maker(keypair.pubkey())
    .market(market_pubkey)
    .base_mint(base_mint)
    .quote_mint(quote_mint)
    .bid()
    .price("0.65")
    .size("100")
    .expiration(chrono::Utc::now().timestamp() + 60)  // expires in 60 seconds
    .apply_scaling(&decimals)?
    .to_submit_request(&keypair, orderbook_id.as_str())?;
```

Set `expiration` to a unix timestamp. `0` (the default) means no expiration. Expired orders are automatically rejected by the matching engine.

---
## Authentication

Most endpoints and subscriptions are public. Authentication is only required for user-specific endpoints:

- **REST**: `client.orders().get_user_orders()`
- **WebSocket**: `MessageOut::subscribe_user()`

Order placement and cancellation do **not** require authentication - the Ed25519 signature embedded in each request proves ownership. Authentication is only needed to query open orders and subscribe to user-specific real-time events.

```rust
use lightcone_sdk_v2::auth::native::sign_login_message;
use std::time::{SystemTime, UNIX_EPOCH};

let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
let signed = sign_login_message(&keypair, timestamp);

let user = client.auth().login_with_message(
    &signed.message,
    &signed.signature_bs58,
    &signed.pubkey_bytes,
    None,
).await?;
```

The SDK stores the auth token internally and injects it as a cookie header on every request. The token is never exposed via public API. `AuthCredentials` only exposes: `user_id`, `wallet_address`, `expires_at`, `is_authenticated()`.

Call `client.auth().logout()` to invalidate the session both server-side and locally. Client-side clearing alone is insufficient - the backend must be told to invalidate.

---
## API Reference

### REST Sub-Clients

Sub-clients are accessed through borrowing accessors on `LightconeClient`. Each borrows `&self` with zero allocation.

**`client.markets()`** - market discovery

| Method | Returns | Description |
|--------|---------|-------------|
| `.get(cursor: Option<i64>, limit: Option<u32>)` | `MarketsResult` | Paginated listing of active/resolved markets. `MarketsResult` contains `markets: Vec<Market>` and `validation_errors: Vec<String>` for any markets that failed validation. |
| `.featured()` | `Vec<MarketSearchResult>` | Promoted/featured markets |
| `.get_by_slug(slug: &str)` | `Market` | Single market by URL slug (e.g. `"btc-100k"`) |
| `.get_by_pubkey(pubkey: &str)` | `Market` | Single market by on-chain pubkey |
| `.search(query: &str, limit: Option<u32>)` | `Vec<MarketSearchResult>` | Full-text search across market names, categories, and tags |

**`client.orderbooks()`** - orderbook data

| Method | Returns | Description |
|--------|---------|-------------|
| `.get(orderbook_id: &str, depth: Option<u32>)` | `OrderbookDepthResponse` | Live bids/asks with best_bid, best_ask, and `RestBookLevel` entries (price + size as `Decimal`, plus `orders` count). `depth` limits the number of levels returned. |
| `.decimals(orderbook_id: &str)` | `DecimalsResponse` | Returns `base_decimals`, `quote_decimals`, `price_decimals` needed for order scaling. Cached internally - only hits the API once per orderbook. |

**`client.orders()`** - order submission & cancellation

| Method | Returns | Description |
|--------|---------|-------------|
| `.submit(request: &impl Serialize)` | `SubmitOrderResponse` | Submit a signed order. Response contains `order_hash`, `filled`, `remaining`, and a `fills` array with counterparty details. |
| `.cancel(body: &CancelBody)` | `CancelSuccess` | Cancel a single order. `CancelBody::signed(order_hash, maker, &keypair)` builds the signed request. |
| `.cancel_all(body: &CancelAllBody)` | `CancelAllSuccess` | Cancel all orders in an orderbook. `CancelAllBody::signed(user_pubkey, orderbook_id, timestamp, &keypair)` builds the signed request. |
| `.get_user_orders(wallet: &str, limit: Option<u32>, cursor: Option<&str>)` | `UserOrdersResponse` | Open orders for an authenticated user. `limit` defaults to 200 (max 1000). `cursor` is the `next_cursor` from a previous response for pagination. Response contains `orders: Vec<UserOrder>`, `balances: Vec<MarketBalance>`, `next_cursor`, and `has_more`. |

**`client.positions()`** - user positions

| Method | Returns | Description |
|--------|---------|-------------|
| `.get(user_pubkey: &str)` | `PositionsResponse` | All positions across all markets. Contains `owner`, `total_markets`, `positions: Vec<PositionEntry>`, and `decimals: HashMap<String, u8>`. |
| `.get_for_market(user_pubkey: &str, market_pubkey: &str)` | `MarketPositionsResponse` | Positions in a specific market. Contains `owner`, `market_pubkey`, `positions: Vec<PositionEntry>`, and `decimals: HashMap<String, u8>`. |

**`client.trades()`** - trade history

| Method | Returns | Description |
|--------|---------|-------------|
| `.get(orderbook_id: &str, limit: Option<u32>, before: Option<i64>)` | `Vec<Trade>` | Executed trades for an orderbook. `before` is an integer cursor for pagination. Each `Trade` has `orderbook_id`, `trade_id`, `timestamp`, `price`, `size`, and `side`. |

**`client.price_history()`** - OHLCV candles

| Method | Returns | Description |
|--------|---------|-------------|
| `.get(orderbook_id: &str, resolution: Resolution, from: Option<u64>, to: Option<u64>)` | `serde_json::Value` | Candlestick data. `from`/`to` are optional unix timestamps to bound the query. |

**`client.auth()`** - session management

| Method | Returns | Description |
|--------|---------|-------------|
| `.login_with_message(message, signature_bs58, pubkey_bytes, use_embedded_wallet)` | `User` | Establish a session. `message: &str`, `signature_bs58: &str`, and `pubkey_bytes: &[u8; 32]` come from `sign_login_message()`. `use_embedded_wallet: Option<bool>`. |
| `.check_session()` | `User` | Validate the current session and return the user profile. Clears credentials on failure. |
| `.logout()` | `()` | Invalidate the session server-side and clear local credentials. |
| `.is_authenticated()` | `bool` | Check cached auth status (async - reads internal lock, does not call the server). |

### WebSocket Streaming

`client.ws_native()` creates an owned WebSocket client for real-time streaming. The connection lifecycle is application-managed: connect, subscribe, process events, disconnect. If authenticated via `client.auth().login_with_message()`, the WebSocket connection automatically inherits the session.

| Method | Description |
|--------|-------------|
| `ws.connect().await?` | Open the WebSocket connection |
| `ws.send(msg)?` | Send a subscription or ping message |
| `ws.subscribe(params)?` | Subscribe using `SubscribeParams` directly |
| `ws.unsubscribe(params)?` | Unsubscribe using `UnsubscribeParams` directly |
| `ws.events()` | Returns an async `Stream` of `WsEvent` |
| `ws.is_connected()` | Returns `bool` — whether the connection is open |
| `ws.ready_state()` | Returns `ReadyState` (`Connecting`, `Open`, `Closing`, `Closed`) |
| `ws.restart_connection().await` | Drop and re-establish the connection. Subscriptions are re-sent automatically. |
| `ws.clear_authed_subscriptions()` | Remove user-channel subscriptions (useful on logout) |
| `ws.disconnect().await?` | Close the connection |

**Subscriptions:**

| Helper | Channel | Description |
|--------|---------|-------------|
| `MessageOut::subscribe_books(orderbook_ids)` | `book_update` | Orderbook snapshots + deltas |
| `MessageOut::subscribe_trades(orderbook_ids)` | `trades` | Executed trades |
| `MessageOut::subscribe_ticker(orderbook_ids)` | `ticker` | Best bid/ask/mid changes |
| `MessageOut::subscribe_price_history(orderbook_id, resolution)` | `price_history` | OHLCV candles |
| `MessageOut::subscribe_user(wallet_address)` | `user` | Orders and balances (auth required) |
| `MessageOut::subscribe_market(market_pubkey)` | `market` | Lifecycle events (settled, opened, paused) |

Each has a corresponding `unsubscribe_*` helper. Subscriptions are automatically re-established on reconnect.

**Inbound events (`WsEvent`):**

| Event | Description |
|-------|-------------|
| `WsEvent::Connected` | Connection established |
| `WsEvent::Message(Kind::BookUpdate(..))` | Orderbook snapshot or delta. Apply to `OrderbookSnapshot` for `best_bid()`, `best_ask()`, `spread()`, `mid_price()`. |
| `WsEvent::Message(Kind::Trade(..))` | Executed trade. Push to `TradeHistory` for a rolling buffer. |
| `WsEvent::Message(Kind::Ticker(..))` | Best bid/ask/mid update |
| `WsEvent::Message(Kind::PriceHistory(..))` | OHLCV candle snapshot or update. Apply to `PriceHistoryState`. |
| `WsEvent::Message(Kind::User(..))` | User-specific: `Snapshot` (initial open orders + balances), `OrderUpdate` (fill/cancel), `BalanceUpdate` (idle/on_book per outcome) |
| `WsEvent::Message(Kind::Market(..))` | Market lifecycle: `Settled`, `Created`, `Opened`, `Paused`, `OrderbookCreated` |
| `WsEvent::Message(Kind::Auth(..))` | Auth confirmation: `Authenticated`, `Anonymous`, or `Failed` |
| `WsEvent::Message(Kind::Error(..))` | Server-side error with `error: String`, and optional `code`, `hint`, `details`, `orderbook_id`, `wallet_address` for diagnostics |
| `WsEvent::Disconnected { code, reason }` | Connection lost. Auto-reconnect handles recovery (exponential backoff, up to 10 attempts). |
| `WsEvent::MaxReconnectReached` | All reconnect attempts exhausted. Recreate the client. |
| `WsEvent::Error(msg)` | Client-side deserialization or protocol error |

**State containers:**

| Container | Source Event | Purpose |
|-----------|-------------|---------|
| `OrderbookSnapshot` | `Kind::BookUpdate` | Local orderbook state with `best_bid()`, `best_ask()`, `spread()`, `mid_price()` |
| `TradeHistory` | `Kind::Trade` | Rolling buffer of trades with configurable max size |
| `PriceHistoryState` | `Kind::PriceHistory` | Candle data keyed by orderbook + resolution |

### Program (On-Chain)

The `LightconePinocchioClient` handles direct Solana program interaction. Requires the `native` feature flag and an RPC URL.

```rust
use lightcone_sdk_v2::program::client::LightconePinocchioClient;
let program = LightconePinocchioClient::new("https://api.mainnet-beta.solana.com");
```

**Key types:**
- **`OrderBuilder`** - builder for constructing orders. Supports auto-scaling from human-readable price/size, or raw u64 amounts.
- **`SignedOrder`** - the 225-byte signed order payload (nonce, maker, market, base/quote mints, side, amounts, expiration, Ed25519 signature).
- **`CancelBody`** / **`CancelAllBody`** - signed cancel requests with `.signed()` helper for native keypairs.

**Account fetchers** (read on-chain state):
`get_exchange()`, `get_market(market_id: u64)`, `get_market_by_pubkey(market: &Pubkey)`, `get_position(owner, market) → Option<Position>`, `get_order_status(order_hash: &[u8; 32]) → Option<OrderStatus>`, `get_user_nonce(user)`, `get_current_nonce(user)`, `get_orderbook(mint_a, mint_b)`, `get_global_deposit_token(mint)`

**Transaction builders** (return unsigned `Transaction`):
`mint_complete_set(params, num_outcomes)`, `merge_complete_set(params, num_outcomes)`, `withdraw_from_position(params, is_token_2022)`, `increment_nonce(user)`, `cancel_order(maker, market, order)`, `redeem_winnings(params, winning_outcome)`, `deposit_to_global(params)`, `global_to_market_deposit(params, num_outcomes)`

**PDA helpers**:
`get_exchange_pda()`, `get_market_pda(id)`, `get_position_pda(owner, market)`, `get_user_nonce_pda(user)`, `get_orderbook_pda(mint_a, mint_b)`, `get_order_status_pda(hash)`, `get_global_deposit_token_pda(mint)`, `get_user_global_deposit_pda(user, mint)`

---

## Error Handling

The SDK has two error enums: `error::SdkError` for client operations (REST, WebSocket, auth) and `program::error::SdkError` for on-chain operations.

**`error::SdkError`** (client):

| Variant | Description |
|---------|-------------|
| `Http(HttpError)` | REST request failure (see below) |
| `Ws(WsError)` | WebSocket connection or protocol failure |
| `Auth(AuthError)` | Authentication failure (`NotAuthenticated`, `LoginFailed`, `SignatureVerificationFailed`, `TokenExpired`) |
| `Validation(String)` | Market or response validation failure |
| `Serde(serde_json::Error)` | JSON serialization/deserialization failure |
| `Other(String)` | Order rejected, cancel failed, or other server-side error |

**`HttpError`** variants:

| Variant | Description |
|---------|-------------|
| `Reqwest(reqwest::Error)` | Underlying HTTP client error (network failure, TLS, DNS, etc.) |
| `RateLimited { retry_after_ms }` | 429 response. GET requests auto-retry; POST requests do not. |
| `ServerError { status, body }` | 5xx response. GET requests auto-retry on 502/503/504. |
| `MaxRetriesExceeded { attempts, last_error }` | All automatic retry attempts exhausted |
| `Unauthorized` | 401 response. Session expired or missing. |
| `BadRequest(String)` | 400-499 response. Malformed request. |
| `NotFound(String)` | 404 response. |
| `Timeout` | Request exceeded 30s timeout. |

**`program::error::SdkError`** (on-chain):

| Variant | Description |
|---------|-------------|
| `Rpc(ClientError)` | Solana RPC call failed |
| `Scaling(ScalingError)` | Price/size conversion failed (non-positive values, overflow, fractional lamports) |
| `UnsignedOrder` | Attempted to submit an order that has not been signed |
| `AccountNotFound(String)` | On-chain account does not exist |
| `Overflow` | Arithmetic overflow (e.g. nonce exceeds u32) |
| `InvalidSignature` | Signature could not be parsed |
| `InvalidDiscriminator { expected, actual }` | On-chain account has wrong discriminator bytes |
| `InvalidDataLength { expected, actual }` | On-chain account data is the wrong size |
| `InvalidOutcomeCount { count }` | Outcome count outside valid range |
| `InvalidOutcomeIndex { index, max }` | Outcome index exceeds market's outcome count |
| `MissingField(String)` | Required field not set on a builder |
| `Serialization(String)` | Failed to serialize/deserialize account data |
| `InvalidPubkey(String)` | Could not parse a public key string |
| `DivisionByZero` | Division by zero in amount calculation |

---

## Examples
All examples are runnable with `cargo run --example <name> --features native`. Set environment variables in a `.env` file - see [`.env.example`](.env.example) for the template.

### Setup & Authentication

| Example | Description |
|---------|-------------|
| [`login`](examples/login.rs) | Full auth lifecycle: sign message, login, check session, logout |

### Market Discovery & Data

| Example | Description |
|---------|-------------|
| [`markets`](examples/markets.rs) | Featured markets, paginated listing, fetch by slug/pubkey, search |
| [`orderbook`](examples/orderbook.rs) | Fetch orderbook depth (bids/asks) and decimal precision metadata |
| [`trades`](examples/trades.rs) | Recent trade history with cursor-based pagination |
| [`price_history`](examples/price_history.rs) | Historical candlestick data (OHLCV) at various resolutions |
| [`positions`](examples/positions.rs) | User positions across all markets and per-market |

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.rs) | `OrderBuilder` with human-readable price/size, auto-scaling, and fill tracking |

### Cancelling Orders

| Example | Description |
|---------|-------------|
| [`cancel_order`](examples/cancel_order.rs) | Cancel a single order by hash and cancel all orders in an orderbook |
| [`user_orders`](examples/user_orders.rs) | Fetch open orders for an authenticated user |

### On-Chain Operations

| Example | Description |
|---------|-------------|
| [`read_onchain`](examples/read_onchain.rs) | Read exchange state, market state, user nonce, and PDA derivations via RPC |
| [`onchain_transactions`](examples/onchain_transactions.rs) | Build and sign mint/merge complete set, withdraw from position, increment nonce |

### WebSocket Streaming

| Example | Description |
|---------|-------------|
| [`ws_book_and_trades`](examples/ws_book_and_trades.rs) | Live orderbook depth with `OrderbookSnapshot` state + rolling `TradeHistory` buffer |
| [`ws_ticker_and_prices`](examples/ws_ticker_and_prices.rs) | Best bid/ask ticker + price history candles with `PriceHistoryState` |
| [`ws_user_and_market`](examples/ws_user_and_market.rs) | Authenticated user stream (orders, balances) + market lifecycle events |