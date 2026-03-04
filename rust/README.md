# Lightcone SDK

Rust SDK for the [Lightcone](https://lightcone.xyz) impact market protocol on Solana. Supports both native and WASM targets under a single crate with compile-time feature dispatch.

## Table of Contents

- [Installation](#installation)
- [Feature Flags](#feature-flags)
- [How Lightcone Works](#how-lightcone-works)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Domain Types vs Wire Types](#domain-types-vs-wire-types)
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


## How Lightcone Works

Lightcone is an impact market protocol on Solana. Before writing integration code, read the [protocol overview](https://lightconelabs.mintlify.app/learn/about-lightcone/what-is-lightcone) to understand the core concepts: markets, conditional tokens, orderbooks, and the trading lifecycle.

### Markets

A **market** represents a question with N outcomes (e.g., "Who wins the election?" with outcomes "Candidate A", "Candidate B"). Each market has:

- **Conditional tokens** -- SPL tokens representing bets on outcomes. Each deposit asset produces its own set of conditional tokens (one per outcome), so a market with 2 outcomes and 2 deposit assets has 4 conditional tokens total.
- **Deposit assets** -- collateral tokens (e.g., USDC) used to mint complete sets of conditional tokens. A market can accept multiple deposit assets.
- **Lifecycle**: `Pending` -> `Active` (accepting orders) -> `Resolved` (winning outcome determined).

[Module docs](src/domain/market/README.md)

### Orderbooks

Each market has one or more **orderbooks**, each representing a tradable pair of conditional tokens (base/quote). Orderbooks are identified by an `OrderBookId` derived from the base and quote token mint prefixes (e.g., `"7BgBvyjr_EPjFWdd5"`). The orderbook's **decimals** define the precision for price and size values.

[Module docs](src/domain/orderbook/README.md)

### Orders

Orders are **signed off-chain** and submitted to the matching engine via REST. The signing flow:

1. Build an `OrderPayload` using the envelope builder (`LimitOrderEnvelope` or `TriggerOrderEnvelope`)
2. Apply scaling to convert human-readable price/size to raw amounts based on orderbook decimals
3. Sign with your keypair (native), browser extension wallet (browser), or Privy embedded wallet (browser)
4. Submit via `client.orders().submit(&request)`

**Limit orders** sit on the book until filled, cancelled, or expired. **Trigger orders** (take-profit / stop-loss) are held server-side and fire when a price threshold is hit.

[Module docs](src/domain/order/README.md)

### Positions

After orders fill, users hold **conditional token balances** per market. Positions track:

- **Idle balance** -- tokens available for new orders or withdrawal
- **On-book balance** -- tokens locked in open orders

When a market resolves, holders of the winning outcome's conditional tokens can **redeem** them for the deposit asset (e.g., USDC).

[Module docs](src/domain/position/README.md)

### Trades

Every fill produces a **trade record** with price, size, side, and timestamp. Trades are queryable per-orderbook with cursor-based pagination and available in real-time via WebSocket.

[Module docs](src/domain/trade/README.md)

### Price History

OHLCV candle data for orderbooks, available at multiple resolutions. Used for charting and historical analysis. Candle updates are also streamed in real-time via WebSocket.

[Module docs](src/domain/price_history/README.md)

### WebSocket

The WebSocket feed provides real-time updates for:

- **Book depth** -- snapshots and deltas for orderbook state
- **Trades** -- individual trade executions
- **User events** -- order placements, fills, cancellations, balance updates
- **Ticker** -- best bid/ask/mid prices
- **Price history** -- OHLCV candle updates
- **Market events** -- settlement, activation, pausing

For live market making, the WebSocket feed is the primary data source.

[Module docs](src/ws/README.md)

### Authentication

Session-based authentication using ED25519 signed messages. The flow is: request a nonce, sign it with your wallet, and exchange it for a session token. Native clients use keypair signing directly; browser clients can use a wallet adapter or Privy embedded wallet.

[Module docs](src/auth/README.md)

### On-Chain Program

Low-level interaction with the Lightcone Solana smart contract: account type definitions, instruction builders, PDA derivation, and order signing/verification. Used by the backend and external integrators; the SDK's HTTP client handles most use cases without touching this directly.

[Module docs](src/program/README.md)

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

## Architecture

The SDK is organized in five layers:

| Layer | Module | Purpose |
|-------|--------|---------|
| 1 | `shared`, `domain`, `program` | Core types, domain models, on-chain program logic. Always available, WASM-safe. |
| 2 | `auth` | Message generation + platform-dependent signing. |
| 3 | `http` | Low-level HTTP client with per-endpoint retry policies. |
| 4 | `ws` | WebSocket with compile-time dispatch: `tokio-tungstenite` (native) / `web-sys` (WASM). |
| 5 | `client` | `LightconeClient` -- high-level nested sub-client API. |

```
use lightcone::prelude::*;

client.markets()       -> Markets API
client.orderbooks()    -> Orderbooks API
client.orders()        -> Orders API
client.positions()     -> Positions API
client.trades()        -> Trades API
client.price_history() -> Price History API
client.auth()          -> Authentication
client.referrals()     -> Referral System
client.admin()         -> Admin Operations
client.privy()         -> Privy Embedded Wallet
client.ws_native()     -> Native WebSocket Client
```

### State Management

The SDK is intentionally **stateless** for HTTP data. Sub-clients fetch, convert, and return -- no internal caching. Different consumers want different caching strategies, so the SDK leaves that decision to you.

The only exception: `orderbooks().decimals()` caches results because orderbook decimals are effectively immutable.

For **WebSocket-driven live state**, the SDK provides standalone state containers with update methods:

- `OrderbookSnapshot` -- apply book snapshots/deltas, query best bid/ask/mid
- `UserOpenOrders` -- track open orders by market
- `TradeHistory` -- rolling trade buffer per orderbook
- `PriceHistoryState` -- manage price chart data

You instantiate these, wrap them in your application's reactive state, and feed them WS events.

## Domain Types vs Wire Types

The SDK exposes two type levels:

- **Domain types** (`lightcone::domain::*`) -- validated, rich types with business logic. These are the primary API. Example: `Market`, `Order`, `OrderBookPair`.
- **Wire types** (`lightcone::domain::*/wire`) -- raw serde structs matching backend REST/WS responses. Public for consumers who need raw access (forwarding data, debugging).

Always prefer domain types unless you have a specific reason to work with wire types.

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
| `RateLimited { retry_after_ms }` | 429 -- back off and retry |
| `Unauthorized` | 401 -- session expired or missing |
| `MaxRetriesExceeded { attempts, last_error }` | All retry attempts exhausted |

## Retry Strategy

- **GET requests**: `RetryPolicy::Idempotent` -- retries on transport failures and 502/503/504, backs off on 429 with exponential backoff + jitter.
- **POST requests** (order submit, cancel, auth): `RetryPolicy::None` -- no automatic retry. Non-idempotent actions are never retried to prevent duplicate side effects.
- Customizable per-call with `RetryPolicy::Custom(RetryConfig { .. })`.

