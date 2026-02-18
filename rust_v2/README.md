# Lightcone SDK v2

Unified Rust SDK for the Lightcone protocol. Supports **both native and WASM** targets under a single crate with compile-time feature dispatch.

## Why v2?

The v1 SDK (`deps/sdk/rust/`) was built as a native-first Rust library with three independent modules (`program`, `api`, `websocket`). It worked well for CLI tooling and server-side use, but introduced significant friction when the Dioxus fullstack app needed to use it on both native (server) and WASM (browser) targets simultaneously.

### Problems with v1

- **Not WASM-compatible.** The API client used `tokio::sync::RwLock`, `tokio::time::sleep`, and `reqwest` features (`.timeout()`, `.is_connect()`, `cookies`) that don't compile to `wasm32-unknown-unknown`. The WebSocket client was `tokio-tungstenite`-only.
- **No domain types.** v1 only exposed raw wire types (REST response structs). The app had to define its own domain types (`Market`, `Order`, `OrderBookPair`, etc.) in `src/domain/`, implement all `TryFrom` conversions, validation logic, and business methods. This duplicated work that belongs in the SDK.
- **No WebSocket message types.** The app defined its own `MessageIn`, `MessageOut`, `Kind`, `SubscribeParams`, and `Subscription` types in `src/service/ws/`. These should be SDK-provided so other consumers get the same typed WS interface.
- **Flat client API.** All endpoints were top-level methods on a single `LightconeApiClient` struct, making it hard to manage per-domain caching or organize related operations.
- **All-or-nothing retry.** Retry was a global setting. Non-idempotent POSTs (order submission, cancellation) were retried the same as GETs, risking duplicate actions.
- **No auth abstraction.** Auth was a standalone function that returned a raw JWT string. No credential management, no platform-aware token handling, no logout flow.

### What v2 adds

- **Unified native + WASM support.** Compile-time `cfg` dispatch for platform-specific code (async locks, timers, HTTP features, WebSocket transport). One crate, two targets, zero runtime overhead.
- **Rich domain types with validation.** `Market`, `Order`, `OrderBookPair`, `Outcome`, `ConditionalToken`, `DepositAsset`, `TokenBalance`, `Portfolio`, etc. — all migrated from the app with their `TryFrom` conversions and business logic.
- **Wire types as a public secondary API.** Raw serde structs for REST and WS are public under `domain::*/wire` for consumers who need raw access (server functions, debugging), but domain types are the primary interface.
- **Typed WebSocket messages.** `MessageIn`, `MessageOut`, `Kind` enum, `SubscribeParams`, `UnsubscribeParams`, and `Subscription` trait — all SDK-provided with full channel coverage (book, trades, user, price_history, ticker, market).
- **Nested sub-client API.** `client.markets().get_by_slug(...)`, `client.orderbooks().decimals(...)`, `client.orders().submit(...)` — organized by domain for clean ergonomics. The SDK is stateless for HTTP data; caching is the consumer's responsibility.
- **Granular retry policies.** `RetryPolicy::Idempotent` for GETs, `RetryPolicy::None` for non-idempotent POSTs, `RetryPolicy::Custom` for anything else. Backoff + jitter, 429 `Retry-After` support.
- **Secure auth with platform-aware token handling.** HTTP-only cookies on WASM (SDK never touches the token), internal private storage on native (never exposed via public API). Logout calls the backend endpoint to clear cookies server-side.
- **App-owned WS state containers.** `OrderbookSnapshot`, `UserOpenOrders`, `TradeHistory`, `PriceHistoryState` — standalone types with update methods that the app wraps in its own reactive state (e.g. Dioxus `Signal`). Avoids the `RwLock` vs reactive signal mismatch.
- **On-chain program module carried over from v1.** Instructions, orders, PDAs, accounts — unchanged.

### Coexistence with v1

Both SDKs can be imported simultaneously for incremental migration:

```rust
use lightcone_sdk::prelude::*;     // v1 — existing code
use lightcone_sdk_v2::prelude::*;  // v2 — new code
```

The v2 crate is named `lightcone-sdk-v2` in `Cargo.toml` and imported as a non-optional dependency. Migrate domain-by-domain, then drop v1.

## Architecture

The SDK is organized in five layers:

| Layer | Module | Purpose |
|-------|--------|---------|
| 1 | `shared`, `domain`, `program` | Core types, domain models, on-chain program logic. Always available, WASM-safe. |
| 2 | `auth` | Message generation + platform-dependent signing. |
| 3 | `http` | `LightconeHttp` — low-level HTTP client with per-endpoint retry policies. |
| 4 | `ws` | WebSocket — compile-time dispatch: `tokio-tungstenite` (native) / `web-sys` (WASM). |
| 5 | `client` | `LightconeClient` — high-level nested sub-client API with caching. |

### Module Layout

```
src/
  lib.rs               # Public re-exports + prelude
  client.rs            # LightconeClient, LightconeClientBuilder, sub-client accessors
  error.rs             # SdkError, HttpError, WsError, AuthError
  network.rs           # DEFAULT_API_URL, DEFAULT_WS_URL
  shared/
    mod.rs             # OrderBookId, PubkeyStr, Side, Resolution, SubmitOrderRequest
    scaling.rs         # Price/size → raw lamport conversion
    price.rs           # Decimal formatting utilities
  domain/
    mod.rs
    market/
      mod.rs           # Market, Status, ValidationError
      client.rs        # Markets sub-client (get, get_by_slug, search, featured, cache)
      outcome.rs       # Outcome, OutcomeValidationError (sub-entity of market)
      tokens.rs        # Token trait, ConditionalToken, DepositAsset, ValidatedTokens, TokenMetadata + TryFrom
      wire.rs          # MarketResponse, OutcomeResponse, DepositAssetResponse, etc.
      convert.rs       # TryFrom<MarketResponse> for Market
    orderbook/
      mod.rs           # OrderBookPair, OrderBookValidationError, OutcomeImpact
      client.rs        # Orderbooks sub-client (get, decimals, cache)
      ticker.rs        # TickerData (best bid/ask/mid)
      wire.rs          # OrderbookResponse, DecimalsResponse, BookOrder, etc.
      state.rs         # OrderbookSnapshot with apply(), bids(), asks(), best_bid()
      convert.rs       # TryFrom<(OrderbookResponse, &[ConditionalToken])> for OrderBookPair
    order/
      mod.rs           # Order, OrderType, OrderStatus
      client.rs        # Orders sub-client (submit, cancel, cancel_all, get_user_orders)
      state.rs         # UserOpenOrders (app-owned state container)
      wire.rs          # OrderUpdate, UserSnapshot, UserUpdate, AuthUpdate + WS balance types
      convert.rs       # From<OrderUpdate/UserSnapshotOrder> for Order
    trade/
      mod.rs           # Trade
      client.rs        # Trades sub-client (get)
      state.rs         # TradeHistory (rolling buffer, app-owned)
      wire.rs          # TradeResponse, WsTrade, TradesResponse
      convert.rs       # From<TradeResponse/WsTrade> for Trade
    price_history/
      mod.rs           # LineData
      client.rs        # PriceHistoryClient sub-client (get)
      state.rs         # PriceHistoryState (app-owned state container)
      wire.rs          # PriceHistory snapshot/update (WS)
    position/
      mod.rs           # Portfolio, Position, PositionOutcome, WalletHolding, TokenBalance types
      client.rs        # Positions sub-client (get, get_for_market)
      wire.rs          # PositionsResponse, PositionResponse (REST)
    admin/
      mod.rs           # AdminEnvelope (domain-level signed envelope)
      client.rs        # Admin sub-client (upsert_metadata)
      wire.rs          # UnifiedMetadataRequest/Response, *MetadataPayload types
  program/             # On-chain: instructions, orders, PDAs, accounts (from v1)
  auth/
    mod.rs             # generate_signin_message, AuthCredentials, LoginRequest/Response
    client.rs          # Auth sub-client (login_with_message, logout, credentials)
    native.rs          # KeypairAuth — sign_login_message (native-auth feature)
  http/
    mod.rs
    client.rs          # LightconeHttp (internal)
    retry.rs           # RetryPolicy, RetryConfig
  ws/
    mod.rs             # MessageIn, MessageOut, Kind, WsEvent, WsConfig
    subscriptions.rs   # SubscribeParams, UnsubscribeParams, Subscription trait
    native.rs          # tokio-tungstenite (ws-native feature) [TODO]
    wasm.rs            # web-sys WebSocket (ws-wasm feature) [TODO]
```

## Features

| Feature | What it enables |
|---------|-----------------|
| `http` (default) | HTTP client (Layers 1-3). Works on both native and WASM. |
| `native-auth` | Keypair-based authentication (Layer 2 native). |
| `ws-native` | WebSocket via `tokio-tungstenite` (Layer 4 native). |
| `ws-wasm` | WebSocket via `web-sys::WebSocket` (Layer 4 WASM). |
| `solana-rpc` | `solana-client` for on-chain reads (native only). |
| `native` | Bundle: `http` + `native-auth` + `ws-native` + `solana-rpc`. |
| `full-native` | Everything for CLI/server use. |
| `full-wasm` | Everything for browser use: `http` + `ws-wasm`. |

## Quick Start

```rust
use lightcone_sdk_v2::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LightconeClient::builder()
        .base_url("https://tapi.lightcone.xyz")
        .build()?;

    // Nested sub-client API
    let featured = client.markets().featured().await?;
    let market = client.markets().get_by_slug("some-market").await?;
    let orderbook = client.orderbooks().get("orderbook_id", Some(10)).await?;
    let decimals = client.orderbooks().decimals("orderbook_id").await?;

    Ok(())
}
```

## Retry Strategy

- **GET requests**: `RetryPolicy::Idempotent` by default — retries on transport failures + 502/503/504, backs off on 429.
- **POST /orders/submit, /orders/cancel, /orders/cancel-all**: `RetryPolicy::None` by default.
- **POST /auth/login**: `RetryPolicy::None`.
- All retry policies are overridable per-call.

**Rule of thumb**: Retry transport failures and status checks; don't retry "actions" unless you have idempotency guarantees.

## Domain Types vs Wire Types

The SDK exposes two levels of types:

- **Domain types** (`lightcone_sdk::domain::*`) — Rich, validated types with business logic methods. These are the primary public API. Example: `Market`, `Order`, `OrderBookPair`.
- **Wire types** (`lightcone_sdk::domain::*/wire`) — Raw serde structs matching backend REST responses and WS messages. Also public for consumers who need raw access (forwarding data, debugging, server functions).

Shared newtypes like `OrderBookId`, `PubkeyStr`, `Side`, and `Resolution` are used directly in wire types because they are serialization-transparent — they deserialize identically to the raw format the backend sends.

## State Management

### HTTP: Stateless (consumer owns caching)

The SDK is intentionally **stateless** for HTTP data. Sub-clients are thin, typed wrappers over the REST API — they fetch, convert wire → domain types, and return. No market cache, no slug index, no TTLs.

**Why?** Different consumers want radically different caching strategies:
- Dioxus server functions use `static LazyLock<Mutex<Cache>>` with 1-hour TTLs
- CLI tools may want no caching at all
- Admin dashboards may want short TTLs with manual invalidation

The SDK shouldn't pick one strategy. The consumer knows best.

**The only exception:** `orderbooks().decimals()` — orderbook decimals are effectively immutable. This is a pure memoization, not a caching policy.

### WS-Driven Live State (app-owned)

The SDK does NOT store WS-driven state internally. Instead it provides standalone state container types with update methods:

- `OrderbookSnapshot` — `apply()` for book snapshots/deltas, `best_bid()`, `best_ask()`, `mid_price()`
- `UserOpenOrders` — `upsert()`, `remove()` for order updates
- `TradeHistory` — rolling buffer with `push()`
- `PriceHistoryState` — `apply_snapshot()`, `apply_update()`

The app instantiates these, wraps them in its own reactive state (e.g. Dioxus `Signal`), and calls SDK update methods when WS events arrive. This avoids the `RwLock` vs reactive signal mismatch.

## Authentication — CRITICAL Security Model

**This section is mandatory reading for any SDK implementation in any language.**

### Wasm/Browser

- Token lives **ONLY** in the HTTP-only cookie set by the backend.
- The SDK **never** reads, stores, or exposes the token programmatically.
- Authenticated requests work because the browser auto-includes cookies.
- **Browser SDKs MUST use HTTP-only cookies** — never store tokens in localStorage, sessionStorage, or any JS/WASM-accessible location.

### Native/CLI

- The SDK stores the token **internally** (private field) and injects it as a `Cookie: auth_token=<token>` header, matching the backend's cookie-only auth model.
- Token is **NEVER** exposed via public API — no `.token()` accessor.
- `AuthCredentials` only exposes: `user_id`, `wallet_address`, `expires_at`, `is_authenticated()`.
- We manually inject the cookie rather than using reqwest's `cookie_store(true)` because the backend hardcodes `Domain=.lightcone.xyz` on the Set-Cookie, which would break local development (requests to `localhost` wouldn't match the domain).

### Logout

On **both** platforms, `client.auth().logout()`:
1. Calls `POST /api/auth/logout` to clear the HTTP-only cookie server-side.
2. On native: clears the internal token.
3. Clears all sub-client HTTP caches.

Client-side clearing alone is **insufficient** for cookie-based auth — the backend must be told to invalidate.

## Backend API Alignment

The SDK aligns with `lightcone-backend` API routes:

| SDK Method | Backend Route | HTTP Method |
|------------|--------------|-------------|
| `markets().get()` | `/api/markets` | GET |
| `markets().get_by_slug(slug)` | `/api/markets/by-slug/{slug}` | GET |
| `markets().search(q, limit)` | `/api/markets/search` | GET |
| `markets().featured()` | `/api/markets/featured` | GET |
| `orderbooks().get(id, depth)` | `/api/orderbook/{id}` | GET |
| `orderbooks().decimals(id)` | `/api/orderbooks/{id}/decimals` | GET |
| `orders().submit(req)` | `/api/orders/submit` | POST |
| `orders().cancel(req)` | `/api/orders/cancel` | POST |
| `orders().cancel_all(req)` | `/api/orders/cancel-all` | POST |
| `orders().get_user_orders(req)` | `/api/users/orders` | POST |
| `positions().get(pubkey)` | `/api/users/{pubkey}/positions` | GET |
| `positions().get_for_market(pubkey, market)` | `/api/users/{pubkey}/positions?market={market}` | GET |
| `trades().get(id, limit, before)` | `/api/trades` | GET |
| `price_history().get(...)` | `/api/price-history` | GET |
| `admin().upsert_metadata(env)` | `/api/admin/metadata` | POST |
| `auth().login_with_message(...)` | `/api/auth/login_or_register_with_message` | POST |
| `auth().logout()` | `/api/auth/logout` | POST |

## WebSocket Channels

| Channel | Subscribe | Events |
|---------|-----------|--------|
| `book` | `SubscribeParams::Books { orderbook_ids }` | `Kind::BookUpdate` — snapshot + delta |
| `trades` | `SubscribeParams::Trades { orderbook_ids }` | `Kind::Trade` |
| `user` | `SubscribeParams::User` | `Kind::User` — snapshot, order_update, balance_update |
| `price_history` | `SubscribeParams::PriceHistory { orderbook_id, resolution }` | `Kind::PriceHistory` — snapshot + update |
| `ticker` | `SubscribeParams::Ticker { orderbook_ids }` | `Kind::Ticker` — best bid/ask/mid |
| `market` | `SubscribeParams::Market { market_pubkey }` | `Kind::Market` — settled, created, opened, paused, orderbook_created |

WS `book_update` uses `conditional_pair: "market_pubkey:orderbook_id"` (combined string) — wire type parses/splits this.
