# Lightcone SDK

TypeScript SDK for the Lightcone impact market protocol on Solana.

## Table of Contents
- [Installation](#installation)
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

```bash
npm install @lightconexyz/lightcone-sdk
```

## Quick Start

```typescript
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  LightconeClient,
  DepositSource,
  auth,
} from "@lightconexyz/lightcone-sdk";

async function main() {
  const client = LightconeClient.builder()
    .rpcUrl("https://api.devnet.solana.com")
    .depositSource(DepositSource.Market)
    .build();
  const keypair = Keypair.generate();

  // 1. Authenticate
  {
    const nonce = await client.auth().getNonce();
    const signed = auth.signLoginMessage(keypair, nonce);
    const user = await client.auth().loginWithMessage(
      signed.message,
      signed.signature_bs58,
      signed.pubkey_bytes
    );
  }

  // 2. Find a market
  const market = await client.markets().getBySlug("some-market");
  const orderbook = market.orderbookPairs[0];

  // 3. Deposit collateral to the global pool
  const depositMint = new PublicKey(market.depositAssets[0].pubkey);
  const depositIx = client.positions().depositToGlobal()
    .user(keypair.publicKey)
    .mint(depositMint)
    .amount(1_000_000n)
    .buildIx();

  // 4. Build, sign, and submit a limit order
  const request = client.orders().limitOrder()
    .maker(keypair.publicKey)
    .bid()
    .price("0.55")
    .size("100")
    .sign(keypair, orderbook);

  const response = await client.orders().submit(request);
  console.log("Order submitted:", response);

  // 5. Withdraw from the global pool
  const withdrawIx = client.positions().withdrawFromGlobal()
    .user(keypair.publicKey)
    .mint(depositMint)
    .amount(1_000_000n)
    .buildIx();

  // 6. Stream real-time updates
  const ws = client.ws();
  await ws.connect();
  ws.subscribe({ type: "book_update", orderbook_ids: [orderbook.orderbookId] });
}

main().catch(console.error);
```

## Start Trading

```typescript
import * as fs from "fs";
import * as path from "path";
import { Keypair, PublicKey } from "@solana/web3.js";
import { LightconeClient, DepositSource } from "@lightconexyz/lightcone-sdk";

function readKeypairFile(filePath: string): Keypair {
  const resolved = filePath.startsWith("~")
    ? path.join(process.env.HOME ?? "", filePath.slice(1))
    : filePath;
  const secret = JSON.parse(fs.readFileSync(resolved, "utf-8"));
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

const client = LightconeClient.builder()
  .rpcUrl("https://api.devnet.solana.com")
  .depositSource(DepositSource.Market)
  .build();
const keypair = readKeypairFile("~/.config/solana/id.json");
```

### Step 1: Find a Market

```typescript
const market = await client.markets().getBySlug("some-market");
const orderbook =
  market.orderbookPairs.find((pair) => pair.active) ?? market.orderbookPairs[0];
```

### Step 2: Deposit Collateral

```typescript
const depositMint = new PublicKey(market.depositAssets[0].pubkey);
const depositIx = client.positions().depositToGlobal()
  .user(keypair.publicKey)
  .mint(depositMint)
  .amount(1_000_000n)
  .buildIx();
```

### Step 3: Place an Order

```typescript
const request = client.orders().limitOrder()
  .maker(keypair.publicKey)
  .bid()
  .price("0.55")
  .size("1")
  .sign(keypair, orderbook);
const order = await client.orders().submit(request);
```

### Step 4: Monitor

```typescript
import { asPubkeyStr } from "@lightconexyz/lightcone-sdk";

const open = await client
  .orders()
  .getUserOrders(keypair.publicKey.toBase58(), 50);
const ws = client.ws();
await ws.connect();
ws.subscribe({ type: "book_update", orderbook_ids: [orderbook.orderbookId] });
ws.subscribe({
  type: "user",
  wallet_address: asPubkeyStr(keypair.publicKey.toBase58()),
});
```

### Step 5: Cancel an Order

```typescript
import { program, asPubkeyStr } from "@lightconexyz/lightcone-sdk";

const signature = program.signCancelOrder(order.order_hash, keypair);
await client.orders().cancel({
  order_hash: order.order_hash,
  maker: asPubkeyStr(keypair.publicKey.toBase58()),
  signature,
});
```

### Step 6: Exit a Position

```typescript
// signAndSubmit builds the tx, signs it using the client's signing strategy, and submits
const txHash = await client.markets().mergeCompleteSet()
  .user(keypair.publicKey)
  .market(new PublicKey(market.pubkey))
  .mint(depositMint)
  .amount(1_000_000n)
  .numOutcomes(numOutcomes)
  .signAndSubmit();
```

### Step 7: Withdraw

```typescript
const withdrawIx = client.positions().withdrawFromGlobal()
  .user(keypair.publicKey)
  .mint(depositMint)
  .amount(1_000_000n)
  .buildIx();
```

## Authentication
Authentication is only required for user-specific endpoints. Authentication is session-based using ED25519 signed messages. The flow is: request a nonce, sign it with your wallet, and exchange it for a session token.

## Examples
All examples are runnable with `npx tsx examples/<name>.ts`. Set environment variables in a `.env` file - see [`.env.example`](.env.example) for the template.

### Setup & Authentication

| Example | Description |
|---------|-------------|
| [`login`](examples/login.ts) | Full auth lifecycle: sign message, login, check session, logout |

### Market Discovery & Data

| Example | Description |
|---------|-------------|
| [`markets`](examples/markets.ts) | Featured markets, paginated listing, fetch by pubkey, search |
| [`orderbook`](examples/orderbook.ts) | Fetch orderbook depth (bids/asks) and decimal precision metadata |
| [`trades`](examples/trades.ts) | Recent trade history with cursor-based pagination |
| [`price_history`](examples/price_history.ts) | Historical candlestick data (OHLCV) at various resolutions |
| [`positions`](examples/positions.ts) | User positions across all markets and per-market |

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.ts) | Limit order via `client.orders().limitOrder()` with human-readable price/size, auto-scaling, and fill tracking |

### Cancelling Orders

| Example | Description |
|---------|-------------|
| [`cancel_order`](examples/cancel_order.ts) | Cancel a single order by hash and cancel all orders in an orderbook |
| [`user_orders`](examples/user_orders.ts) | Fetch open orders for an authenticated user |

### On-Chain Operations

| Example | Description |
|---------|-------------|
| [`read_onchain`](examples/read_onchain.ts) | Read exchange state, market state, user nonce, and PDA derivations via RPC |
| [`onchain_transactions`](examples/onchain_transactions.ts) | Build, sign, and submit mint/merge complete set and increment nonce on-chain |
| [`global_deposit_withdrawal`](examples/global_deposit_withdrawal.ts) | Init position tokens, deposit to global pool, move capital into a market, extend an existing ALT, and withdraw from global |

### WebSocket Streaming

| Example | Description |
|---------|-------------|
| [`ws_book_and_trades`](examples/ws_book_and_trades.ts) | Live orderbook depth with `OrderbookSnapshot` state + rolling `TradeHistory` buffer |
| [`ws_ticker_and_prices`](examples/ws_ticker_and_prices.ts) | Best bid/ask ticker + price history candles with `PriceHistoryState` |
| [`ws_user_and_market`](examples/ws_user_and_market.ts) | Authenticated user stream (orders, balances) + market lifecycle events |

## Error Handling

All SDK operations return promises that reject with `SdkError`:

| Variant | When |
|---------|------|
| `SdkError.Http(HttpError)` | REST request failures |
| `SdkError.Ws(WsError)` | WebSocket connection/protocol errors |
| `SdkError.Auth(AuthError)` | Authentication failures |
| `SdkError.Validation(string)` | Domain type conversion failures |
| `SdkError.Serde(Error)` | Serialization errors |
| `SdkError.Other(string)` | Catch-all |

Notable `HttpError` variants:

| Variant | Meaning |
|---------|---------|
| `ServerError { status, body }` | Non-2xx response from the backend |
| `RateLimited { retryAfterMs }` | 429 - back off and retry |
| `Unauthorized` | 401 - session expired or missing |
| `MaxRetriesExceeded { attempts, lastError }` | All retry attempts exhausted |

## Retry Strategy

- **GET requests**: `RetryPolicy.Idempotent` - retries on transport failures and 502/503/504, backs off on 429 with exponential backoff + jitter.
- **POST requests** (order submit, cancel, auth): `RetryPolicy.None` - no automatic retry. Non-idempotent actions are never retried to prevent duplicate side effects.
- Customizable per-call with `RetryPolicy.custom(config)`.
