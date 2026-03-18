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
- [Examples](#examples)
- [Authentication](#authentication)
- [Error Handling](#error-handling)
- [Retry Strategy](#retry-strategy)
- [Global Deposits](#global-deposits)

## Installation

```bash
npm install @lightconexyz/lightcone-sdk
```

## Quick Start

```typescript
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  LightconeClient,
  LimitOrderEnvelope,
  auth,
} from "@lightconexyz/lightcone-sdk";

async function main() {
  const client = LightconeClient.builder()
    .rpcUrl("https://api.devnet.solana.com")
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
  const { markets } = await client.markets().get();
  const market = markets[0];
  const orderbook = market.orderbookPairs[0];

  // 3. Build, sign, and submit a limit order
  //    Decimals are derived automatically from the orderbook's token metadata.
  const nonce = await client.orders().currentNonce(keypair.publicKey);
  const request = LimitOrderEnvelope.new()
    .maker(keypair.publicKey)
    .market(new PublicKey(market.pubkey))
    .baseMint(new PublicKey(orderbook.base.pubkey))
    .quoteMint(new PublicKey(orderbook.quote.pubkey))
    .bid()
    .price("0.55")
    .size("100")
    .nonce(nonce)
    .sign(keypair, orderbook);

  const response = await client.orders().submit(request);
  console.log("Order submitted:", response);

  // 5. Stream real-time updates
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
import { LightconeClient } from "@lightconexyz/lightcone-sdk";

function readKeypairFile(filePath: string): Keypair {
  const resolved = filePath.startsWith("~")
    ? path.join(process.env.HOME ?? "", filePath.slice(1))
    : filePath;
  const secret = JSON.parse(fs.readFileSync(resolved, "utf-8"));
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

const client = LightconeClient.builder()
  .rpcUrl("https://api.devnet.solana.com")
  .build();
const keypair = readKeypairFile("~/.config/solana/id.json");
```

### Step 1: Find a Market

```typescript
const { markets } = await client.markets().get();
const market = markets[0];
const orderbook =
  market.orderbookPairs.find((pair) => pair.active) ?? market.orderbookPairs[0];
```

### Step 2: Deposit Collateral

```typescript
import { Transaction } from "@solana/web3.js";

const marketPubkey = new PublicKey(market.pubkey);
const depositMint = new PublicKey(market.depositAssets[0].pubkey);
const numOutcomes = market.outcomes.length;
const mintIx = client.markets().mintCompleteSetIx(
  {
    user: keypair.publicKey,
    market: marketPubkey,
    depositMint,
    amount: 1_000_000n,
  },
  numOutcomes
);
const tx = new Transaction().add(mintIx);
tx.feePayer = keypair.publicKey;
tx.recentBlockhash = (await client.rpc().getLatestBlockhash()).blockhash;
tx.sign(keypair);
```

### Step 3: Place an Order

```typescript
import { LimitOrderEnvelope } from "@lightconexyz/lightcone-sdk";

const request = LimitOrderEnvelope.new()
  .maker(keypair.publicKey)
  .market(new PublicKey(market.pubkey))
  .baseMint(new PublicKey(orderbook.base.pubkey))
  .quoteMint(new PublicKey(orderbook.quote.pubkey))
  .bid()
  .price("0.55")
  .size("1")
  .nonce(await client.orders().currentNonce(keypair.publicKey))
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
const mergeIx = client.markets().mergeCompleteSetIx(
  {
    user: keypair.publicKey,
    market: new PublicKey(market.pubkey),
    depositMint,
    amount: 1_000_000n,
  },
  numOutcomes
);
const mergeTx = new Transaction().add(mergeIx);
mergeTx.feePayer = keypair.publicKey;
mergeTx.recentBlockhash = (await client.rpc().getLatestBlockhash()).blockhash;
mergeTx.sign(keypair);
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
| [`submit_order`](examples/submit_order.ts) | `LimitOrderEnvelope` with human-readable price/size, auto-scaling, and fill tracking |

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
| [`global_deposit`](examples/global_deposit.ts) | Deposit collateral to global balance, inspect `global_deposits`, and move funds into a market position |

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

## Global Deposits

The SDK supports the global deposit flow end to end:

- Admins whitelist deposit mints with `client.admin().whitelistDepositTokenIx(...)`.
- Users deposit collateral into their global balance with `client.positions().depositToGlobalIx(...)`.
- Users can move global collateral into a market position with `client.positions().globalToMarketDepositIx(...)`.
- Order submissions can opt into global collateral by setting `deposit_source` or calling `.depositSource(DepositSource.Global)`.

For a runnable script, see [`examples/global_deposit.ts`](examples/global_deposit.ts).
