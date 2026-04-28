# Lightcone SDK

Rust SDK for the Lightcone impact market protocol on Solana.

## Table of Contents
- [Installation](#installation)
- [Feature Flags](#feature-flags)
- [Quick Start](#quick-start)
- [Start Trading](#start-trading)
     - [Step 1: Find a Market](#step-1-find-a-market)
     - [Step 2: Deposit Collateral](#step-2-deposit-collateral)
     - [Step 3: Place an Order](#step-3-place-an-order)
     - [Step 4: Monitor](#step-4-monitor)
     - [Step 5: Cancel an Order](#step-5-cancel-an-order)
     - [Step 6: Exit a Position](#step-6-exit-a-position)
     - [Step 7: Withdraw](#step-7-withdraw)
- [Examples](#examples)
- [Authentication](#authentication)
- [Error Handling](#error-handling)
- [Retry Strategy](#retry-strategy)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lightcone = { version = "0.4.1", features = ["native"] }
```

For browser/WASM targets:

```toml
[dependencies]
lightcone = { version = "0.4.1", features = ["wasm"] }
```

## Feature Flags

| Feature | What it enables | Use case |
|---------|-----------------|----------|
| **`native`** | `http` + `native-auth` + `ws-native` + `solana-rpc` | **Market makers, bots, CLI tools** |
| **`wasm`** | `http` + `ws-wasm` | **Browser applications** |

## Quick Start

```rust
use lightcone::prelude::*;
use lightcone::auth::native::sign_login_message;
use solana_keypair::Keypair;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Defaults to Prod. Use .env(LightconeEnv::Staging) for staging.
    let client = LightconeClient::builder()
        .deposit_source(DepositSource::Market)
        .build()?;
    let keypair = Keypair::new();

    // 1. Authenticate
    let nonce = client.auth().get_nonce().await?;
    let signed = sign_login_message(&keypair, &nonce);
    let user = client.auth().login_with_message(
        &signed.message,
        &signed.signature_bs58,
        &signed.pubkey_bytes,
        None,
    ).await?;

    // 2. Find a market
    let market = client.markets().get(None, Some(1)).await?.markets.into_iter().next().unwrap();
    let orderbook = &market.orderbook_pairs[0];

    // 3. Deposit collateral to the global pool
    let deposit_mint = market.deposit_assets[0].pubkey().to_pubkey()?;
    let deposit_ix = client.positions().deposit().await
        .user(keypair.pubkey())
        .mint(deposit_mint)
        .amount(1_000_000)
        .build_ix()
        .await?;

    // 4. Build, sign, and submit a limit order
    let request = client.orders().limit_order().await
        .maker(keypair.pubkey())
        .bid()
        .price("0.55")
        .size("100")
        .sign(&keypair, &orderbook)?;

    let response = client.orders().submit(&request).await?;
    println!("Order submitted: {:?}", response);

    // 5. Withdraw from the global pool
    let withdraw_ix = client.positions().withdraw().await
        .user(keypair.pubkey())
        .mint(deposit_mint)
        .amount(1_000_000)
        .build_ix()
        .await?;

    // 6. Stream real-time updates
    let mut ws = client.ws_native();
    ws.connect().await?;
    ws.subscribe(SubscribeParams::Books {
        orderbook_ids: vec![orderbook.orderbook_id.clone()],
    })?;

    Ok(())
}
```

## Start Trading

```rust
use lightcone::prelude::*;
use solana_keypair::read_keypair_file;
use solana_signer::Signer;

// Defaults to Prod. Use .env(LightconeEnv::Staging) for staging.
let client = LightconeClient::builder()
    .deposit_source(DepositSource::Market)
    .build()?;
let keypair = read_keypair_file("~/.config/solana/id.json")?;
```

### Step 1: Find a Market

```rust
let market = client.markets().get(None, Some(1)).await?.markets.into_iter().next().unwrap();
let orderbook = market
    .orderbook_pairs
    .iter()
    .find(|pair| pair.active)
    .or_else(|| market.orderbook_pairs.first())
    .expect("market has no orderbooks");
```

### Step 2: Deposit Collateral

```rust
let deposit_mint = market.deposit_assets[0].pubkey().to_pubkey()?;
let deposit_ix = client.positions().deposit().await
    .user(keypair.pubkey())
    .mint(deposit_mint)
    .amount(1_000_000)
    .build_ix()
    .await?;
```

### Step 3: Place an Order

```rust
let request = client.orders().limit_order().await
    .maker(keypair.pubkey())
    .bid()
    .price("0.55")
    .size("1")
    .sign(&keypair, &orderbook)?;
let order = client.orders().submit(&request).await?;
```

### Step 4: Monitor

```rust
let open = client
    .orders()
    .get_user_orders(&keypair.pubkey().to_string(), Some(50), None)
    .await?;
let mut ws = client.ws_native();
ws.connect().await?;
ws.subscribe(SubscribeParams::Books {
    orderbook_ids: vec![orderbook.orderbook_id.clone()],
})?;
ws.subscribe(SubscribeParams::User {
    wallet_address: keypair.pubkey().into(),
})?;
```

### Step 5: Cancel an Order

```rust
let cancel = CancelBody::signed(order.order_hash.clone(), keypair.pubkey().into(), &keypair);
client.orders().cancel(&cancel).await?;
```

### Step 6: Exit a Position

```rust
// sign_and_submit builds the tx, signs it using the client's signing strategy, and submits
let tx_hash = client.positions().merge()
    .user(keypair.pubkey())
    .market(&market)
    .mint(deposit_mint)
    .amount(1_000_000)
    .sign_and_submit()
    .await?;
```

### Step 7: Withdraw

```rust
let withdraw_ix = client.positions().withdraw().await
    .user(keypair.pubkey())
    .mint(deposit_mint)
    .amount(1_000_000)
    .build_ix()
    .await?;
```

## Authentication
Authentication is only required for user-specific endpoints. Authentication is session-based using ED25519 signed messages. The flow is: request a nonce, sign it with your wallet, and exchange it for a session token.

### Cookie handling

After `client.auth().login_with_message(...)` succeeds, the SDK stores the session token internally and attaches it as `Cookie: auth_token=…` on every authenticated request. The behaviour depends on the build target:

- **Native builds**: token lives in a process-wide `Arc<RwLock<Option<String>>>` on the `LightconeClient`. Every authed call reads from it.
- **WASM builds**: requests use `credentials: "include"` and the browser supplies the cookie automatically — the SDK's internal store is unused.

### Server-side cookie forwarding (`_with_auth` variants)

When the SDK runs on a server (SSR, server functions, an axum handler, etc.) and the *user's* `auth_token` cookie arrives on an incoming HTTP request, the SDK's process-wide token store is the wrong place to route it through — the store is shared across all users of that server process.

For these cases, authed methods that need per-call forwarding ship a `_with_auth(auth_token)` sibling that injects the cookie just for that one call:

```rust
// Inside an axum / dioxus server function, after extracting the
// auth_token cookie from the incoming request:
let balances = client
    .positions()
    .deposit_token_balances_with_auth(&auth_token)
    .await?;

let positions = client
    .positions()
    .positions_with_auth(&auth_token)
    .await?;
```

On WASM these methods are equivalent to their non-`_with_auth` counterparts because the browser is already attaching the cookie via credentials mode.

If you maintain a non-Rust SDK (TypeScript, Python) and need to support an SSR consumer, mirror the same pattern: the wire contract is unchanged — only the per-call `Cookie: auth_token=<token>` header attachment differs.

## Environment Configuration

The SDK defaults to the **production** environment. Use `LightconeEnv` to target a different deployment:

```rust
// Production (default — no .env() call needed)
let client = LightconeClient::builder().build()?;

// Staging
let client = LightconeClient::builder()
    .env(LightconeEnv::Staging)
    .build()?;

// Local development
let client = LightconeClient::builder()
    .env(LightconeEnv::Local)
    .build()?;
```

Each environment configures the API URL, WebSocket URL, Solana RPC URL, and on-chain program ID automatically. Individual URL overrides (`.base_url()`, `.ws_url()`, `.rpc_url()`) take precedence when called after `.env()`.

## Examples
All examples are runnable with `cargo run --example <name> --features native`. Examples default to the production environment and read the wallet keypair from `~/.config/solana/id.json`.

### Setup & Authentication

| Example | Description |
|---------|-------------|
| [`login`](examples/login.rs) | Full auth lifecycle: sign message, login, check session, logout |
| [`auth_override`](examples/auth_override.rs) | Per-call cookie override for SSR / server-function consumers — logs in, captures the token via `client.auth_token()`, clears the SDK's internal store, and exercises every `_with_auth_override` variant |

### Market Discovery & Data

| Example | Description |
|---------|-------------|
| [`markets`](examples/markets.rs) | Featured markets, paginated listing, fetch by pubkey, search, platform deposit assets via `global_deposit_assets()` |
| [`market_deposit_assets`](examples/market_deposit_assets.rs) | List the deposit assets and conditional mints for a specific market |
| [`orderbook`](examples/orderbook.rs) | Fetch orderbook depth (bids/asks) and decimal precision metadata |
| [`trades`](examples/trades.rs) | Recent trade history with cursor-based pagination |
| [`price_history`](examples/price_history.rs) | Historical candlestick data (OHLCV) at various resolutions |
| [`positions`](examples/positions.rs) | User positions across all markets and per-market |
| [`metrics_all`](examples/metrics_all.rs) | Exercise every endpoint on `client.metrics()` — platform, markets, categories, orderbook, leaderboard, history |

### Admin & Testnet

| Example | Description |
|---------|-------------|
| [`faucet_claim`](examples/faucet_claim.rs) | Request testnet SOL + deposit tokens via `client.claim()` |

Admin API methods (`client.admin()`) live in the SDK but are not exercised by an example because they require an admin keypair the CI runner doesn't have. See [`domain/admin/ADMIN.md`](src/domain/admin/ADMIN.md) for usage.

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.rs) | Deposit the quote amount into the global pool, then place a limit order via `client.orders().limit_order()` with human-readable price/size, auto-scaling, and fill tracking. Companion `cancel_order` cancels it and withdraws to stay net-neutral |

### Cancelling Orders

| Example | Description |
|---------|-------------|
| [`cancel_order`](examples/cancel_order.rs) | Cancel a single order by hash, cancel all orders in an orderbook, and withdraw the released collateral from the global pool |
| [`user_orders`](examples/user_orders.rs) | Fetch open orders for an authenticated user |

### On-Chain Operations

| Example | Description |
|---------|-------------|
| [`read_onchain`](examples/read_onchain.rs) | Read exchange state, market state, user nonce, and PDA derivations via RPC |
| [`onchain_transactions`](examples/onchain_transactions.rs) | Build, sign, and submit mint/merge complete set and increment nonce on-chain |
| [`global_deposit_withdrawal`](examples/global_deposit_withdrawal.rs) | Init position tokens, deposit to global pool, move capital into a market, extend an existing ALT, withdraw from global, and merge back to keep the run net-neutral |

### WebSocket Streaming

| Example | Description |
|---------|-------------|
| [`ws_book_and_trades`](examples/ws_book_and_trades.rs) | Live orderbook depth with `OrderbookState` state + rolling `TradeHistory` buffer |
| [`ws_ticker_and_prices`](examples/ws_ticker_and_prices.rs) | Best bid/ask ticker + price history candles with `PriceHistoryState` |
| [`ws_user_and_market`](examples/ws_user_and_market.rs) | Authenticated user stream (orders, balances) + market lifecycle events |

## Error Handling

All SDK operations return `Result<T, SdkError>`:

| Variant | When |
|---------|------|
| `SdkError::Http(HttpError)` | REST request failures |
| `SdkError::Ws(WsError)` | WebSocket connection/protocol errors |
| `SdkError::Auth(AuthError)` | Authentication failures |
| `SdkError::Validation(String)` | Domain type conversion failures |
| `SdkError::Serde(serde_json::Error)` | Serialization errors |
| `SdkError::MissingMarketContext(string)` | Market context not provided for operation requiring `DepositSource::Market` |
| `SdkError::Signing(String)` | Signing operation failures |
| `SdkError::UserCancelled` | User cancelled wallet signing prompt |
| `SdkError::ApiRejected(ApiRejectedDetails)` | Backend rejected the request (see [API Rejections](#api-rejections)) |
| `SdkError::Program(program::SdkError)` | On-chain program errors (RPC, account parsing) |
| `SdkError::Other(String)` | Catch-all |

### API Rejections

When the backend rejects a request (insufficient balance, expired order, etc.), the SDK returns `SdkError::ApiRejected(details)` where `details` is an `ApiRejectedDetails` containing:

| Field | Type | Description |
|-------|------|-------------|
| `reason` | `String` | Human-readable error message |
| `rejection_code` | `Option<RejectionCode>` | Machine-readable rejection code (see below) |
| `error_code` | `Option<String>` | API-level error code (e.g. `"NOT_FOUND"`, `"INVALID_ARGUMENT"`) |
| `error_log_id` | `Option<String>` | Backend support correlation ID (`LCERR_*`) |
| `request_id` | `Option<String>` | SDK-generated `x-request-id` for cross-service tracing |

`Display` formats all present fields as a multi-line report. Use `.to_string()` for logging or clipboard.

#### `RejectionCode`

Machine-readable rejection codes with a human-readable `.label()` method. Unrecognized codes from the backend are captured as `Unknown(String)` for forward compatibility.

| Variant | Label | When |
|---------|-------|------|
| `InsufficientBalance` | "Insufficient Balance" | Not enough funds to fill the order |
| `Expired` | "Expired" | Order expiration time has passed |
| `NonceMismatch` | "Nonce Mismatch" | Order nonce doesn't match current user nonce |
| `SelfTrade` | "Self Trade" | Order would match against the maker's own order |
| `MarketInactive` | "Market Inactive" | Market is not accepting orders |
| `BelowMinOrderSize` | "Below Min Order Size" | Order size is below the minimum |
| `InvalidNonce` | "Invalid Nonce" | Nonce is invalid |
| `BroadcastFailure` | "Broadcast Failure" | Failed to broadcast to the network |
| `OrderNotFound` | "Order Not Found" | Order does not exist |
| `NotOrderMaker` | "Not Order Maker" | Caller is not the order maker |
| `OrderAlreadyFilled` | "Order Already Filled" | Order has already been fully filled |
| `OrderAlreadyCancelled` | "Order Already Cancelled" | Order was already cancelled |
| `Unknown(String)` | *(raw code)* | Unrecognized code (forward compatible) |

```rust
match client.orders().submit(&request).await {
    Ok(response) => println!("Order placed: {}", response.order_hash),
    Err(SdkError::ApiRejected(details)) => {
        if let Some(code) = &details.rejection_code {
            println!("Rejected ({}): {}", code.label(), details.reason);
        }
        if let Some(log_id) = &details.error_log_id {
            println!("Support code: {}", log_id);
        }
    }
    Err(other) => eprintln!("Error: {}", other),
}
```

### Request Correlation

The SDK generates a UUID v4 `x-request-id` header on every HTTP request. On rejection, this ID is attached to `ApiRejectedDetails.request_id` for cross-service tracing. The same ID is sent to the backend for correlation in logs and error events.

`HttpError` variants:

| Variant | Meaning |
|---------|---------|
| `Reqwest(reqwest::Error)` | Network/transport failure |
| `ServerError { status, body }` | Non-2xx response from the backend |
| `RateLimited { retry_after_ms }` | 429 - back off and retry |
| `Unauthorized` | 401 - session expired or missing |
| `NotFound(String)` | 404 - resource not found |
| `BadRequest(String)` | 400 - invalid request |
| `Timeout` | Request timed out |
| `MaxRetriesExceeded { attempts, last_error }` | All retry attempts exhausted |

## Retry Strategy

- **GET requests**: `RetryPolicy::Idempotent` - retries on transport failures and 502/503/504, backs off on 429 with exponential backoff + jitter.
- **POST requests** (order submit, cancel, auth): `RetryPolicy::None` - no automatic retry. Non-idempotent actions are never retried to prevent duplicate side effects.
- Customizable per-call with `RetryPolicy::Custom(RetryConfig { .. })`.
