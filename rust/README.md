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
- [Examples](#examples)
- [Error Handling](#error-handling)
- [Retry Strategy](#retry-strategy)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lightcone = { version = "0.3", features = ["native"] }
```

For browser/WASM targets:

```toml
[dependencies]
lightcone = { version = "0.3", features = ["wasm"] }
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
    let client = LightconeClient::builder().build()?;
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
    let market = client.markets().get_by_slug("some-market").await?;
    let orderbook = &market.orderbook_pairs[0];

    // 3. Get orderbook decimals for price scaling
    let decimals = client.orderbooks()
        .decimals(orderbook.orderbook_id.as_str()).await?;

    // 4. Build, sign, and submit a limit order
    let nonce = 1u32;
    let request = LimitOrderEnvelope::new()
        .maker(keypair.pubkey())
        .market(market.pubkey.to_pubkey()?)
        .base_mint(orderbook.base.mint.to_pubkey()?)
        .quote_mint(orderbook.quote.mint.to_pubkey()?)
        .bid()
        .price("0.55")
        .size("100")
        .nonce(nonce)
        .apply_scaling(&decimals)?
        .sign(&keypair, orderbook.orderbook_id.as_str())?;

    let response = client.orders().submit(&request).await?;
    println!("Order submitted: {:?}", response);

    // 5. Stream real-time updates
    let mut ws = client.ws_native();
    ws.connect().await?;
    ws.subscribe(SubscribeParams::Books {
        orderbook_ids: vec![orderbook.orderbook_id.clone()],
    })?;

    Ok(())
}
```

## Start Trading

### Step 1: Find a Market

### Step 2: Deposit Collateral

### Step 3: Place an Order

### Step 4: Monitor

### Step 5: Cancel an Order

### Step 6: Exit a Position

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

## Error Handling

All SDK operations return `Result<T, SdkError>`:

| Variant | When |
|---------|------|
| `SdkError::Http(HttpError)` | REST request failures |
| `SdkError::Ws(WsError)` | WebSocket connection/protocol errors |
| `SdkError::Auth(AuthError)` | Authentication failures |
| `SdkError::Validation(String)` | Domain type conversion failures |
| `SdkError::Serde(serde_json::Error)` | Serialization errors |
| `SdkError::Other(String)` | Catch-all |

Notable `HttpError` variants:

| Variant | Meaning |
|---------|---------|
| `ServerError { status, body }` | Non-2xx response from the backend |
| `RateLimited { retry_after_ms }` | 429 - back off and retry |
| `Unauthorized` | 401 - session expired or missing |
| `MaxRetriesExceeded { attempts, last_error }` | All retry attempts exhausted |

## Retry Strategy

- **GET requests**: `RetryPolicy::Idempotent` - retries on transport failures and 502/503/504, backs off on 429 with exponential backoff + jitter.
- **POST requests** (order submit, cancel, auth): `RetryPolicy::None` - no automatic retry. Non-idempotent actions are never retried to prevent duplicate side effects.
- Customizable per-call with `RetryPolicy::Custom(RetryConfig { .. })`.

