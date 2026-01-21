# Lightcone TypeScript SDK

TypeScript SDK for the Lightcone protocol on Solana.

## Installation

```bash
npm install @lightcone/sdk
```

## Modules

| Module | Description |
|--------|-------------|
| [`program`](src/program/README.md) | On-chain Solana program interaction (accounts, transactions, orders, Ed25519 verification) |
| [`api`](src/api/README.md) | REST API client for market data and order management |
| [`websocket`](src/websocket/README.md) | Real-time data streaming via WebSocket |
| [`shared`](src/shared/README.md) | Shared utilities (Resolution, constants, decimal helpers) |

## Quick Start

```typescript
import { LightconePinocchioClient, PROGRAM_ID, api, websocket } from "@lightcone/sdk";
```

### REST API

Query markets, submit orders, and manage positions via the REST API.

```typescript
import { api } from "@lightcone/sdk";

const client = new api.LightconeApiClient();

// Get markets
const markets = await client.getMarkets();
console.log(`Found ${markets.total} markets`);

// Get orderbook depth
const orderbook = await client.getOrderbook("orderbook_id", 10);
console.log(`Best bid: ${orderbook.best_bid}`);
console.log(`Best ask: ${orderbook.best_ask}`);

// Get user positions
const positions = await client.getUserPositions("user_pubkey");
```

### On-Chain Program

Build transactions for on-chain operations: minting positions, matching orders, redeeming winnings.

```typescript
import { Connection, Keypair } from "@solana/web3.js";
import { LightconePinocchioClient, BidOrderParams } from "@lightcone/sdk";

const connection = new Connection("https://api.devnet.solana.com");
const client = new LightconePinocchioClient(connection);

// Fetch account state
const exchange = await client.getExchange();
const market = await client.getMarket(0n);
console.log(`Market status: ${market.status}`);

// Create a signed order
const keypair = Keypair.generate();
const nonce = await client.getNextNonce(keypair.publicKey);

const order = client.signFullOrder(
  client.createBidOrder({
    nonce,
    maker: keypair.publicKey,
    market: client.pda.getMarketPda(0n, client.programId)[0],
    baseMint: conditionalTokenMint,
    quoteMint: usdcMint,
    makerAmount: 1_000_000n, // 1 USDC
    takerAmount: 500_000n,   // 0.5 outcome tokens
    expiration: 0n,
  }),
  keypair
);
```

### WebSocket

Stream real-time orderbook updates, trades, and user events.

```typescript
import { websocket } from "@lightcone/sdk";

const client = await websocket.LightconeWebSocketClient.connectDefault();

// Subscribe to orderbook updates
client.subscribeBookUpdates(["market:orderbook"]);

// Register event handler
client.on((event) => {
  if (event.type === "BookUpdate") {
    const book = client.getOrderbook(event.orderbookId);
    if (book) {
      console.log(`Best bid: ${book.bestBid()}`);
      console.log(`Best ask: ${book.bestAsk()}`);
    }
  } else if (event.type === "Trade") {
    console.log(`Trade: ${event.trade.size} @ ${event.trade.price}`);
  } else if (event.type === "Disconnected") {
    console.log(`Disconnected: ${event.reason}`);
  }
});
```

## Module Overview

### Program Module

Direct interaction with the Lightcone Solana program:

- **Account Types**: Exchange, Market, Position, OrderStatus, UserNonce
- **Transaction Builders**: All 14 instructions (mint, merge, match, settle, etc.)
- **PDA Derivation**: 8 PDA functions with seeds
- **Order Types**: FullOrder (225 bytes), CompactOrder (65 bytes)
- **Ed25519 Verification**: Three strategies (individual, batch, cross-reference)

```typescript
import {
  LightconePinocchioClient,
  createBidOrder,
  signOrderFull,
  hashOrder,
} from "@lightcone/sdk";

// Create and sign an order
const order = signOrderFull(
  createBidOrder({
    nonce: 1n,
    maker: pubkey,
    market: marketPda,
    baseMint: yesToken,
    quoteMint: noToken,
    makerAmount: 100_000n,
    takerAmount: 50_000n,
    expiration: 0n,
  }),
  keypair
);
const orderHash = hashOrder(order);
```

### API Module

REST API client with typed requests and responses:

- **Markets**: getMarkets, getMarket, getMarketBySlug
- **Orderbooks**: getOrderbook with depth parameter
- **Orders**: submitOrder, cancelOrder, cancelAllOrders
- **Positions**: getUserPositions, getUserMarketPositions
- **Trades**: getTrades with filters
- **Price History**: getPriceHistory with OHLCV

```typescript
import { api } from "@lightcone/sdk";

const client = new api.LightconeApiClient();
const response = await client.submitOrder({
  maker: "pubkey",
  nonce: 1,
  market_pubkey: "market",
  base_token: "base",
  quote_token: "quote",
  side: 0, // BID
  maker_amount: 1000000,
  taker_amount: 500000,
  expiration: 0,
  signature: "hex_signature",
  orderbook_id: "orderbook",
});
```

### WebSocket Module

Real-time streaming with automatic state management:

- **Subscriptions**: book_updates, trades, user, price_history, market
- **State Management**: LocalOrderbook, UserState, PriceHistory
- **Authentication**: Ed25519 sign-in for user streams
- **Auto-Reconnect**: Configurable reconnection with exponential backoff and jitter

```typescript
import { websocket } from "@lightcone/sdk";
import { Keypair } from "@solana/web3.js";

// Authenticated connection for user streams
const keypair = Keypair.generate();
const client = await websocket.LightconeWebSocketClient.connectAuthenticated(keypair);
client.subscribeUser(keypair.publicKey.toBase58());

// Access maintained state
const state = client.getUserState(keypair.publicKey.toBase58());
if (state) {
  console.log(`Open orders: ${state.orderCount()}`);
}
```

### Shared Module

Common utilities used across modules:

- **Resolution**: Candle intervals (1m, 5m, 15m, 1h, 4h, 1d)
- **Constants**: Program IDs, seeds, discriminators, sizes
- **Types**: MarketStatus, OrderSide, account data interfaces

```typescript
import { Resolution, PROGRAM_ID } from "@lightcone/sdk";

const res = Resolution.OneHour; // "1h"
console.log(res); // "1h"
```

## Features

- **Full TypeScript support**: Comprehensive types for all APIs
- **Zero-copy serialization**: Efficient binary order formats
- **Cross-instruction Ed25519**: Optimized transaction sizes
- **Automatic reconnection**: Jittered exponential backoff
- **State management**: Local orderbook and user state tracking

## Program ID

**Mainnet/Devnet**: `Aumw7EC9nnxDjQFzr1fhvXvnG3Rn3Bb5E3kbcbLrBdEk`

## License

MIT
