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
- [Authentication](#authentication)
- [Environment Configuration](#environment-configuration)
- [Examples](#examples)
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
    .depositSource(DepositSource.Market)
    .build();
  const keypair = Keypair.generate();

  // 1. Authenticate
  const nonce = await client.auth().getNonce();
  const signed = auth.signLoginMessage(keypair, nonce);
  await client.auth().loginWithMessage(
    signed.message,
    signed.signature_bs58,
    signed.pubkey_bytes
  );

  // 2. Find a market
  const market = (await client.markets().get(undefined, 1)).markets[0];
  if (!market) {
    throw new Error("No markets returned by the API");
  }
  const orderbook =
    market.orderbookPairs.find((pair) => pair.active) ?? market.orderbookPairs[0];
  if (!orderbook) {
    throw new Error("Selected market has no orderbooks");
  }

  // 3. Deposit collateral to the global pool
  const depositMint = new PublicKey(market.depositAssets[0].pubkey);
  const depositIx = client.positions().deposit()
    .user(keypair.publicKey)
    .mint(depositMint)
    .amount(1_000_000n)
    .buildIx();

  // 4. Build, sign, and submit a limit order
  const response = await client.orders().limitOrder()
    .maker(keypair.publicKey)
    .bid()
    .price("0.55")
    .size("100")
    .submit(client, orderbook);
  console.log("Order submitted:", response);

  // 5. Withdraw from the global pool
  const withdrawIx = client.positions().withdraw()
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
import * as os from "os";
import * as path from "path";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  LightconeClient,
  DepositSource,
} from "@lightconexyz/lightcone-sdk";

function readKeypairFile(filePath: string): Keypair {
  const resolved = filePath.startsWith("~/")
    ? path.join(os.homedir(), filePath.slice(2))
    : filePath;
  const secret = JSON.parse(fs.readFileSync(resolved, "utf-8"));
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

const keypair = readKeypairFile("~/.config/solana/id.json");

// Defaults to Prod. Use .env(LightconeEnv.Staging) for staging.
const client = LightconeClient.builder()
  .nativeSigner(keypair)
  .depositSource(DepositSource.Market)
  .build();
```

### Step 1: Find a Market

```typescript
const market = (await client.markets().get(undefined, 1)).markets[0];
if (!market) {
  throw new Error("No markets returned by the API");
}

const orderbook =
  market.orderbookPairs.find((pair) => pair.active) ?? market.orderbookPairs[0];
if (!orderbook) {
  throw new Error("Selected market has no orderbooks");
}
```

### Step 2: Deposit Collateral

```typescript
const depositMint = new PublicKey(market.depositAssets[0].pubkey);
const depositIx = client.positions().deposit()
  .user(keypair.publicKey)
  .mint(depositMint)
  .amount(1_000_000n)
  .buildIx();
```

### Step 3: Place an Order

```typescript
const order = await client.orders().limitOrder()
  .maker(keypair.publicKey)
  .bid()
  .price("0.55")
  .size("1")
  .submit(client, orderbook);
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
const txHash = await client.positions().merge()
  .user(keypair.publicKey)
  .market(market)
  .mint(depositMint)
  .amount(1_000_000n)
  .signAndSubmit();
```

### Step 7: Withdraw

```typescript
const withdrawIx = client.positions().withdraw()
  .user(keypair.publicKey)
  .mint(depositMint)
  .amount(1_000_000n)
  .buildIx();
```

## Authentication

Authentication is only required for user-specific endpoints. Authentication is session-based using ED25519 signed messages. The flow is: request a nonce, sign it with your wallet, and exchange it for a session cookie.

## Environment Configuration

The SDK defaults to the **production** environment. Use `LightconeEnv` to target a different deployment:

```typescript
import {
  LightconeClient,
  LightconeEnv,
} from "@lightconexyz/lightcone-sdk";

// Production (default — no .env() call needed)
const prodClient = LightconeClient.builder().build();

// Staging
const stagingClient = LightconeClient.builder()
  .env(LightconeEnv.Staging)
  .build();

// Local development
const localClient = LightconeClient.builder()
  .env(LightconeEnv.Local)
  .build();
```

Each environment configures the API URL, WebSocket URL, Solana RPC URL, and on-chain program ID automatically. Individual URL overrides (`.baseUrl()`, `.wsUrl()`, `.rpcUrl()`) take precedence when called after `.env()`.

## Examples

All examples are runnable with `npx tsx examples/<name>.ts`. Examples default to the production environment and read the wallet keypair from `~/.config/solana/id.json`.

### Setup & Authentication

| Example | Description |
|---------|-------------|
| [`login`](examples/login.ts) | Full auth lifecycle: sign message, login, check session, logout |

### Market Discovery & Data

| Example | Description |
|---------|-------------|
| [`markets`](examples/markets.ts) | Featured markets, paginated listing, fetch by pubkey, search, platform deposit assets via `globalDepositAssets()` |
| [`orderbook`](examples/orderbook.ts) | Fetch orderbook depth (bids/asks) and decimal precision metadata |
| [`trades`](examples/trades.ts) | Recent trade history with cursor-based pagination |
| [`price_history`](examples/price_history.ts) | Historical candlestick data (OHLCV) at various resolutions |
| [`positions`](examples/positions.ts) | User positions across all markets and per-market |

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.ts) | Deposit the quote amount into the global pool, then place a limit order via `client.orders().limitOrder()` with human-readable price/size, auto-scaling, and fill tracking. Companion `cancel_order` cancels it and withdraws to stay net-neutral |

### Cancelling Orders

| Example | Description |
|---------|-------------|
| [`cancel_order`](examples/cancel_order.ts) | Cancel a single order by hash, cancel all orders in an orderbook, and withdraw the released collateral from the global pool |
| [`user_orders`](examples/user_orders.ts) | Fetch open orders for an authenticated user |

### On-Chain Operations

| Example | Description |
|---------|-------------|
| [`read_onchain`](examples/read_onchain.ts) | Read exchange state, market state, user nonce, and PDA derivations via RPC |
| [`onchain_transactions`](examples/onchain_transactions.ts) | Build, sign, and submit mint/merge complete set and increment nonce on-chain |
| [`global_deposit_withdrawal`](examples/global_deposit_withdrawal.ts) | Init position tokens, deposit to global pool, move capital into a market, extend an existing ALT, withdraw from global, and merge back to keep the run net-neutral |

### WebSocket Streaming

| Example | Description |
|---------|-------------|
| [`ws_book_and_trades`](examples/ws_book_and_trades.ts) | Live orderbook depth with `OrderbookState` state + rolling `TradeHistory` buffer |
| [`ws_ticker_and_prices`](examples/ws_ticker_and_prices.ts) | Best bid/ask ticker + price history candles with `PriceHistoryState` |
| [`ws_user_and_market`](examples/ws_user_and_market.ts) | Authenticated user stream (orders, balances) + market lifecycle events |

## Error Handling

All SDK operations reject with `SdkError`:

| Variant | When |
|---------|------|
| `Http` | REST request failures |
| `Ws` | WebSocket connection/protocol errors |
| `Auth` | Authentication failures |
| `Validation` | Domain type conversion failures |
| `Serde` | Serialization errors |
| `MissingMarketContext` | Market context not provided for an operation requiring `DepositSource.Market` |
| `Signing` | Signing operation failures |
| `UserCancelled` | User cancelled wallet signing prompt |
| `ApiRejected` | Backend rejected the request (see [API Rejections](#api-rejections)) |
| `Program` | On-chain program errors (RPC, account parsing) |
| `Other` | Catch-all |

### API Rejections

When the backend rejects a request (insufficient balance, expired order, etc.), the SDK throws `SdkError` with `variant === "ApiRejected"`. The structured details are available on `error.apiRejectedDetails`:

| Field | Type | Description |
|-------|------|-------------|
| `reason` | `string` | Human-readable error message |
| `rejectionCode` | `RejectionCode \| undefined` | Machine-readable rejection code (see below) |
| `errorCode` | `string \| undefined` | API-level error code (for example `"NOT_FOUND"` or `"INVALID_ARGUMENT"`) |
| `errorLogId` | `string \| undefined` | Backend support correlation ID (`LCERR_*`) |
| `requestId` | `string \| undefined` | SDK-generated `x-request-id` for cross-service tracing |

`ApiRejectedDetails.toString()` formats all present fields as a multi-line report for logs or support tickets.

#### `RejectionCode`

Machine-readable rejection codes expose a human-readable `.label()` method. Unrecognized codes from the backend are preserved as-is for forward compatibility.

| Wire Code | Label | When |
|-----------|-------|------|
| `INSUFFICIENT_BALANCE` | "Insufficient Balance" | Not enough funds to fill the order |
| `EXPIRED` | "Expired" | Order expiration time has passed |
| `NONCE_MISMATCH` | "Nonce Mismatch" | Order nonce does not match the current user nonce |
| `SELF_TRADE` | "Self Trade" | Order would match against the maker's own order |
| `MARKET_INACTIVE` | "Market Inactive" | Market is not accepting orders |
| `BELOW_MIN_ORDER_SIZE` | "Below Min Order Size" | Order size is below the minimum |
| `INVALID_NONCE` | "Invalid Nonce" | Nonce is invalid |
| `BROADCAST_FAILURE` | "Broadcast Failure" | Failed to broadcast to the network |
| `ORDER_NOT_FOUND` | "Order Not Found" | Order does not exist |
| `NOT_ORDER_MAKER` | "Not Order Maker" | Caller is not the order maker |
| `ORDER_ALREADY_FILLED` | "Order Already Filled" | Order has already been fully filled |
| `ORDER_ALREADY_CANCELLED` | "Order Already Cancelled" | Order was already cancelled |

```typescript
import { SdkError } from "@lightconexyz/lightcone-sdk";

try {
  const response = await client.orders().submit(request);
  console.log("Order placed:", response.order_hash);
} catch (error) {
  if (error instanceof SdkError && error.variant === "ApiRejected") {
    const details = error.apiRejectedDetails;
    if (details?.rejectionCode) {
      console.log(
        `Rejected (${details.rejectionCode.label()}): ${details.reason}`
      );
    }
    if (details?.errorLogId) {
      console.log(`Support code: ${details.errorLogId}`);
    }
  } else {
    console.error(error);
  }
}
```

### Request Correlation

The SDK generates a UUID v4 `x-request-id` header on every HTTP request. On rejection, this ID is attached to `ApiRejectedDetails.requestId` for cross-service tracing. The same ID is sent to the backend for correlation in logs and error events.

`HttpError` variants:

| Variant | Meaning |
|---------|---------|
| `Request` | Network or transport failure |
| `ServerError` | Non-2xx response from the backend |
| `RateLimited` | 429 - back off and retry |
| `Unauthorized` | 401 - session expired or missing |
| `NotFound` | 404 - resource not found |
| `BadRequest` | 400 - invalid request |
| `Timeout` | Request timed out |
| `MaxRetriesExceeded` | All retry attempts exhausted |

## Retry Strategy

- **GET requests**: `RetryPolicy.Idempotent` - retries on transport failures and 502/503/504, backs off on 429 with exponential backoff + jitter.
- **POST requests** (order submit, cancel, auth): `RetryPolicy.None` - no automatic retry. Non-idempotent actions are never retried to prevent duplicate side effects.
- Customizable per-call with `RetryPolicy.custom(config)`.
