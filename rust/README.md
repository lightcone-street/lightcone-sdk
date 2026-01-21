# Lightcone Rust SDK

Rust SDK for the Lightcone prediction market protocol on Solana.

## Installation

```toml
[dependencies]
lightcone-pinocchio-sdk = "0.1.0"
```

## Modules

| Module | Description |
|--------|-------------|
| [`program`](src/program/README.md) | On-chain Solana program interaction (accounts, transactions, orders, Ed25519 verification) |
| [`api`](src/api/README.md) | REST API client for market data and order management |
| [`websocket`](src/websocket/README.md) | Real-time data streaming via WebSocket |
| [`shared`](src/shared/README.md) | Shared utilities (Resolution, decimal helpers) |

## Quick Start

```rust
use lightcone_sdk::prelude::*;
```

### REST API

Query markets, submit orders, and manage positions via the REST API.

```rust
use lightcone_sdk::api::LightconeApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = LightconeApiClient::new("https://api.lightcone.xyz");

    // Get markets
    let markets = api.get_markets().await?;

    // Get orderbook depth
    let orderbook = api.get_orderbook("orderbook_id", Some(10)).await?;
    println!("Best bid: {:?}", orderbook.best_bid);
    println!("Best ask: {:?}", orderbook.best_ask);

    // Get user positions
    let positions = api.get_user_positions("user_pubkey").await?;

    Ok(())
}
```

### On-Chain Program

Build transactions for on-chain operations: minting positions, matching orders, redeeming winnings.

```rust
use lightcone_sdk::program::LightconePinocchioClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LightconePinocchioClient::new("https://api.devnet.solana.com");

    // Fetch account state
    let exchange = client.get_exchange().await?;
    let market = client.get_market(0).await?;

    // Build transactions
    let tx = client.mint_complete_set(MintCompleteSetParams {
        user: &user,
        market: &market_pda,
        deposit_mint: &usdc_mint,
        amount: 1_000_000,
    }, 2).await?;

    Ok(())
}
```

### WebSocket

Stream real-time orderbook updates, trades, and user events.

```rust
use lightcone_sdk::websocket::*;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), WebSocketError> {
    let mut client = LightconeWebSocketClient::connect_default().await?;

    // Subscribe to orderbook updates
    client.subscribe_book_updates(vec!["market:orderbook".to_string()]).await?;

    // Process events
    while let Some(event) = client.next().await {
        match event {
            WsEvent::BookUpdate { orderbook_id, .. } => {
                if let Some(book) = client.get_orderbook(&orderbook_id) {
                    println!("Best bid: {:?}", book.best_bid());
                    println!("Best ask: {:?}", book.best_ask());
                }
            }
            WsEvent::Trade { trade, .. } => {
                println!("Trade: {} @ {}", trade.size, trade.price);
            }
            _ => {}
        }
    }
    Ok(())
}
```

## Module Overview

### Program Module

Direct interaction with the Lightcone Solana program:

- **Account Types**: Exchange, Market, Position, OrderStatus, UserNonce
- **Transaction Builders**: All 14 instructions (mint, merge, match, settle, etc.)
- **PDA Derivation**: 8 PDA functions with seeds
- **Order Types**: FullOrder (225 bytes), CompactOrder (65 bytes)
- **Ed25519 Verification**: Three strategies (individual, batch, cross-reference)

```rust
use lightcone_sdk::program::*;

// Create and sign an order
let mut order = FullOrder::new_bid(BidOrderParams {
    nonce: 1,
    maker: pubkey,
    market: market_pda,
    base_mint: yes_token,
    quote_mint: no_token,
    maker_amount: 100_000,
    taker_amount: 50_000,
    expiration: 0,
});
order.sign(&keypair);
let hash = order.hash();
```

### API Module

REST API client with typed requests and responses:

- **Markets**: get_markets, get_market, get_market_by_slug
- **Orderbooks**: get_orderbook with depth parameter
- **Orders**: submit_order, cancel_order, cancel_all_orders
- **Positions**: get_user_positions, get_user_market_positions
- **Trades**: get_trades with filters
- **Price History**: get_price_history with OHLCV

```rust
use lightcone_sdk::api::*;

let response = client.submit_order(SubmitOrderRequest {
    maker: "pubkey".to_string(),
    nonce: 1,
    market_pubkey: "market".to_string(),
    base_token: "base".to_string(),
    quote_token: "quote".to_string(),
    side: 0,  // BID
    maker_amount: 1000000,
    taker_amount: 500000,
    expiration: 0,
    signature: "hex_signature".to_string(),
    orderbook_id: "orderbook".to_string(),
}).await?;
```

### WebSocket Module

Real-time streaming with automatic state management:

- **Subscriptions**: book_updates, trades, user, price_history, market
- **State Management**: LocalOrderbook, UserState, PriceHistory
- **Authentication**: Ed25519 sign-in for user streams
- **Auto-Reconnect**: Configurable reconnection with exponential backoff

```rust
use lightcone_sdk::websocket::*;

// Authenticated connection for user streams
let client = LightconeWebSocketClient::connect_authenticated(&signing_key).await?;
client.subscribe_user("user_pubkey".to_string()).await?;

// Access maintained state
if let Some(state) = client.get_user_state("user_pubkey") {
    println!("Open orders: {}", state.orders.len());
}
```

### Shared Module

Common utilities used across modules:

- **Resolution**: Candle intervals (1m, 5m, 15m, 1h, 4h, 1d)
- **Decimal Helpers**: parse_decimal, format_decimal for string price handling

```rust
use lightcone_sdk::shared::{Resolution, parse_decimal};

let res = Resolution::OneHour;  // "1h"
let price = parse_decimal("0.500000")?;  // 0.5
```

## Features

- `default` - Standard features
- `live_tests` - Enable live integration tests

## Program ID

**Mainnet/Devnet**: `Aumw7EC9nnxDjQFzr1fhvXvnG3Rn3Bb5E3kbcbLrBdEk`

## License

MIT
