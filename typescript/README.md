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
- [Transaction Fee Funding](#transaction-fee-funding)
- [Retry Strategy](#retry-strategy)

## Installation

```bash
npm install @lightconexyz/lightcone-sdk
```

## Transaction Fee Funding

Shared on-chain submission checks the exact prepared message fee and declared
fee-payer Native SOL Balance before signing when both RPC facts are available.
A proven shortfall is an `SdkError` with variant
`InsufficientSolForTransactionFees`, bigint `availableLamports` and
`requiredLamports` fields, and the canonical deposit-SOL message. Fee or balance
lookup failure continues through the existing submission path; planner-owned SOL
actions retain fail-closed live fee, rent, and reserve checks.

`LightconeClient.builder().transactionSponsorship(true)` and
`client.setTransactionSponsorshipEnabled(true)` are trusted application assertions
for wallet-adapter and Privy signing. The default is false, each transaction
captures its signer and capability before asynchronous RPC work, `clone()` copies
the current value, and local-keypair submission rejects an enabled capability. Raw
`Privy.signAndSendTx` forwarding and off-chain order-message signing are outside
this contract.

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

  // 4. Fetch/cache immutable trading rules, validate exactly, sign, and submit
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

Book streams are snapshot-only: every accepted `book_update` replaces the full
top-20 view. `OrderbookState` discards equal/older `seq` values within a
subscription generation and accepts forward gaps. Call `beginGeneration()` on
reconnect/resubscribe; `resync: true` requires unsubscribe/resubscribe with the
same aggregation. Each `(orderbook, aggregation)` pair needs its own state.
Truncation flags are preserved and mean that side is not exhaustive. See
[`examples/ws_book_and_trades.ts`](examples/ws_book_and_trades.ts).
Each decoded bid and ask level includes exact decimal-string `quote_notional`.
For grouped books, `price` is a display bucket boundary, so quote liquidity
and totals must use `quote_notional` rather than `price * size`. The
price-to-base-size `OrderbookState` maps do not retain quote notional; read it
from the decoded `OrderBook` levels.
Ticker consumers should use the supplied `mid`; it is engine-authoritative and
may use one-sided-book or last-trade fallback.
REST depth is a coherent projection that may briefly lag a mutation. Use its
`revision` and `captured_at_ms` metadata, and expect revision gaps.

```typescript
ws.subscribe({ type: "book_update", orderbook_ids: [orderbook.orderbookId], nSigFigs: 5, mantissa: 2 });
```

Order submission accepts decimal strings and uses exact `bigint` construction.
`submit()` fetches and caches `/api/orderbooks/{id}/decimals` before invoking a
signer. Direct `sign()`/`finalize()` calls require the returned `OrderbookRules`.
Raw amounts, explicit salts, and derived prices are preflighted against the same
signed-64-bit admission rules; no tick or size normalization is implicit.

### Wallet Balances and SOL Action Planning

`depositTokenBalances()` returns a required exact nine-decimal
`native_sol_balance` alongside the separate mint-keyed SPL map. Initialize
`WalletDepositBalancesState` with `applyRestSnapshot(wallet, snapshot)` and feed
the outer `wallet_deposit_balances` channel's nested snapshot, absolute SPL,
absolute native-SOL, and status events to `applyEvent()`. Complete snapshots
replace state even after a higher component slot; status and wrong-wallet events
do not mutate it, pre-baseline component updates are ignored, and explicit-zero
SPL updates remove their mint. Matching SPL updates with invalid or negative
idle balances return `rejected` without mutation. `contextSlot` records the latest accepted
component observation rather than enforcing global monotonic ordering.
`combinedSolBalance()` sums native SOL and canonical WSOL with `bigint` precision
while retaining both stored values. REST response types are trusted rather than
runtime-decoded; WebSocket frames are validated strictly, while malformed REST
exact values fail when a state method scales them. The reducer owns its map
container but retains balance objects by reference, so treat applied payloads as
immutable.

`planSolSplit`, `planSolMerge`, `planSolRedeem`, and
`planNativeSolWithdrawal` return unsigned action plans with live fee/rent costs,
the action-specific reserve and spendable balance, and separate expected native
and canonical WSOL deltas. Each planner requires complete matching-wallet state,
checks the canonical account when needed, and fails closed when RPC estimates or
native reserve are unavailable. Unsponsored actions reserve the greater of live
costs and the applicable 0.001 SOL or 0.0035 SOL floor. Sponsored planning is
rejected until a concrete sponsor owns transaction fees and account rent.
An occupied canonical address is accepted only when it decodes as the wallet's
initialized, unfrozen Tokenkeg native-mint account.

Split plans consume canonical WSOL first and wrap only a shortfall in the same
transaction. Merge and redeem plans retain proceeds in the persistent canonical
account. Native withdrawal transfers directly when possible; otherwise it moves
only the shortfall through a bounded seeded temporary Tokenkeg account and closes
that temporary account before sending the exact native amount to the recipient.
The temporary account's create, initialize, WSOL transfer, close, and native
transfer instructions share one Solana transaction, so an instruction failure
rolls the entire conversion back atomically. No planner closes the canonical
account implicitly.

Native-keypair self-custody users can explicitly call `planWrapSol` with a
positive exact `bigint` lamport amount or call no-amount `planUnwrapWsolAll`.
These standalone planners require the authenticated Trading Wallet's local
native signing strategy; wallet-adapter and Privy strategies are rejected before
RPC planning. Wrap creates or reuses only that wallet's canonical Tokenkeg ATA,
transfers the exact amount, and runs `SyncNative`, the Token Program instruction
that recalculates the WSOL token amount from account lamports. Wrap retains the
ordinary 0.0035 SOL account-creation or 0.001 SOL existing-account reserve floor
above lower live costs. Live decoded canonical amounts must match the authoritative
wallet state. Wrap also requires an existing account's full lamports to equal
its decoded token amount plus native rent reserve; unsynchronized direct
donations reject before `SyncNative` can make the projected canonical delta
inexact. Unwrap-all still accepts such excess and returns it on close.

Unwrap-all accepts no partial amount and closes the entire positive canonical
account back to the same wallet. Its `SolActionCosts` fields contain the fresh fee, zero
upfront rent, no account creation, and no sponsorship. Availability reserves
only that fee rather than the ordinary persistent-account floor;
`unwrapAllSolBalanceAvailability(components, costs)` validates that complete
cost tuple before deriving fee-only fields. The
native delta credits every live account lamport, including refunded rent or
direct donations, minus the fee and removes the full canonical token amount.
Closing a pre-existing account returns all of its WSOL and means a future WSOL
action may need to fund account rent again. Ordinary split, merge, redeem,
claim, order, and native-withdraw paths never call these conversion planners or
close canonical WSOL.

For either explicit conversion, rebuild immediately before signing,
submit with `signAndSubmitPreparedTxConfirmedWithSlot` so the wallet cannot
replace the fee-estimated message, and refresh a complete snapshot covering its
slot before restoring action authority. Prepared submission is unavailable for
Privy because its final signed bytes cannot be verified by the SDK. Atomic
execution does not resolve uncertain submission or confirmation errors; inspect
authoritative balances before retrying. See the
[persistent canonical WSOL ADR](../docs/adr/0001-persistent-canonical-wsol.md).

WebSocket clients are owned independently from `Auth`; logout does not clean them
up. For each retained client, `clearAuthedSubscriptions()` purges User/wallet
reconnect tracking and queued authenticated messages. It does not stop an
already-open server stream; send the matching unsubscribe or disconnect the socket
for live teardown.

The [`deposit_token_balances`](examples/deposit_token_balances.ts) self-custody
example is manual-only and runs with `LIGHTCONE_ENV=local` or `staging` only
when `SDK_API_URL`, `SDK_WS_URL`, `SDK_RPC_URL`, and `SDK_PROGRAM_ID` are all
unset. It sends 0.001 SOL to the Python SDK wallet configured by
`LIGHTCONE_WALLET_PATH_PYTHON` from the distinct sender configured by
`LIGHTCONE_WALLET_PATH_TS`, confirms with a slot, and refreshes a complete
snapshot at that slot. Running it moves funds. If it fails after submission,
inspect authoritative balances before retrying because funds may already have moved.

The separate [`wsol_conversion`](examples/wsol_conversion.ts) example runs with
the TypeScript wallet in local aggregate runs and is included when the globally
gated stateful example workflow is enabled for staging CI; that workflow
currently disables all stateful CI jobs. Local runs may use a paid RPC while
retaining built-in API, WebSocket, and program identity; an enabled staging-CI
run may supply its managed endpoints. Direct staging runs remain override-free.
It permits a pre-existing canonical balance, wraps exactly 0.001 SOL, prints the
exact wallet, costs, full-account return, and future-rent warning, then unwraps
the complete canonical account without pausing. It retains each frozen
projection until a complete REST snapshot covers the confirmed slot. Any
planning, signing, submission, or uncertain-confirmation failure exits without
automatic retry.

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

`market.numOutcomes` is the validated protocol outcome count. Market deposit, merge, and unified withdrawal use it instead of the length of display outcome metadata. The pubkey-only `withdrawFromPosition()` builder requires `.numOutcomes(market.numOutcomes)` before building.

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

Privy hosts can also authenticate with passwordless Email, Google, X, or Wallet. After every interactive success, call `client.auth().registerPrivy({ attempted_identity })`. The backend validates the exact selector against Privy's verified methods, creates or synchronizes the Account, and changes the Primary Login Identity only for a new Account. `session.user.identity` is that stable primary; `session.user.linked_identities` contains every connected method with primary first.

Use `walletDisplayName(session.user, session.auth_method)` to show a shortened
label for the wallet the session trades with, regardless of login identity.

`session.user.max_slippage_preference` is an exact decimal string strictly below
`10`, or `null` until one is stored. Persist a value greater than zero and less
than 10 using `client.auth().updateMaxSlippagePreference(value)`; the method
returns the canonical exact decimal string. Values at or above 10% remain valid
order protection but are not remembered through this API.

### Cookie handling

After login succeeds, the SDK stores the session token internally and attaches it as `Cookie: lightcone-token=…` on every authenticated request. Behaviour depends on the runtime:

- **Node / non-browser**: token is stored on the `LightconeHttp` instance and added as a `Cookie` header per request.
- **Browser**: requests use `credentials: "include"` and the runtime supplies the cookie automatically — the SDK's internal store is unused.

### Server-side cookie forwarding (`*WithAuth` variants)

> **Naming note.** The `WithAuth` suffix does **not** mean other methods are unauthed — most SDK methods that talk to authed endpoints (e.g. `positions().positions()`, `metrics().user()`) read auth from the SDK's process-wide token store / browser cookie automatically; that's the typical client-side path. The `*WithAuth(authToken: string)` siblings exist for **server-side rendering (SSR) and route-handler callers** where the per-request browser cookie can't propagate to the shared client. Those callers extract the token from the incoming request and pass it explicitly. Same wire contract, different credentials path.

When the SDK runs on a server (SSR, an Express / Next.js route handler, etc.) and the *user's* `auth_token` cookie arrives on an incoming HTTP request, the SDK's process-wide token store is the wrong place to route it through — the store is shared across all users of that server process.

> **Behavior change.** `getWithCookies` responses no longer capture `Set-Cookie` into the shared token slot (they previously did): a forwarded per-user request rotating its token must not leak that token to every later request from a shared server client. These requests also never consult the credential restorer.

For these cases, authed methods that need per-call forwarding ship a `*WithAuth(authToken)` sibling that injects the cookie just for that one call:

```typescript
// Inside a server route, after extracting the auth_token cookie
// from the incoming request:
const snapshot = await client
  .positions()
  .depositTokenBalancesWithCookies(undefined, authToken);
console.log(`snapshot slot ${snapshot.context_slot}: ${Object.keys(snapshot.balances).length} balances`);

const positions = await client
  .positions()
  .positionsWithAuth(authToken);
```

In a browser context these methods are equivalent to their non-`WithAuth` counterparts because the runtime is already attaching the cookie via `credentials: "include"`.

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

The Solana RPC URL can also be overridden via the `SDK_RPC_URL` environment variable, which takes precedence over the environment default. This is useful for pointing all examples at a private RPC to avoid public devnet rate limits.

Favorite-market add and remove methods are idempotent set operations, so the SDK may safely replay them after supported credential restoration or transient transport failures. Per-call cookie variants used by SSR and route handlers retry transient failures with the supplied cookie but never invoke the process-wide credential restorer.

## Examples

All examples are runnable with `npx tsx examples/<name>.ts`. Examples default to the production environment and read the wallet keypair from `~/.config/solana/id.json`.

### Setup & Authentication

| Example | Description |
|---------|-------------|
| [`login`](examples/login.ts) | Full auth lifecycle: sign message, login, check session, logout |
| [`with_cookies`](examples/with_cookies.ts) | Per-call cookie forwarding for SSR / route-handler consumers, including paginated favorite-market list/add/remove while restoring the original state |

### Market Discovery & Data

| Example | Description |
|---------|-------------|
| [`markets`](examples/markets.ts) | Featured markets, paginated listing, fetch by pubkey, search, and platform deposit assets via `globalDepositAssets()`; authenticated favorite-market APIs are demonstrated by `with_cookies` |
| [`orderbook`](examples/orderbook.ts) | Fetch orderbook depth (bids/asks) and decimal precision metadata |
| [`trades`](examples/trades.ts) | Recent trade history with cursor-based pagination (per-orderbook and market-wide) |
| [`price_history`](examples/price_history.ts) | Historical candlestick data (OHLCV) at various resolutions |
| [`positions`](examples/positions.ts) | User positions across all markets and per-market |
| [`metrics_all`](examples/metrics_all.ts) | Exercise every endpoint on `client.metrics()` - platform, markets, categories, orderbook, leaderboard, history |

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.ts) | Deposit collateral, then place an exactly validated limit order using cached trading rules. Companion `cancel_order` cancels it and withdraws to stay net-neutral |

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

### Manual Fund-Moving Operations

These examples refuse production and endpoint overrides and are intentionally
excluded from routine example runs.

| Example | Description |
|---------|-------------|
| [`deposit_token_balances`](examples/deposit_token_balances.ts) | Confirm an exact native withdrawal without closing canonical WSOL, then refresh complete state past the confirmed slot |
| [`wsol_conversion`](examples/wsol_conversion.ts) | Wrap an exact native amount, warn, close and unwrap the complete canonical account, and hold frozen projections through covering refreshes |

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
| `existingMethod` | `string \| undefined` | Primary method of the conflicting Account when identity ownership has one deterministic owner |

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
| `DUPLICATE_ORDER` | "Duplicate Order" | Order already exists on this orderbook |
| `POST_ONLY_WOULD_CROSS` | "Post Only Would Cross" | Post-only order would cross resting liquidity |
| `FOK_NO_FILL` | "FOK No Fill" | Fill-or-kill order could not be fully filled |
| `IOC_NO_FILL` | "IOC No Fill" | Immediate-or-cancel order got no fill |
| `WOULD_CROSS_UNAVAILABLE_LIQUIDITY` | "Would Cross Unavailable Liquidity" | Would cross liquidity unavailable for matching |
| `WOULD_CROSS_BOOK` | "Would Cross Book" | Order remainder would leave orderbook crossed |
| `MARKET_NOT_FOUND` | "Market Not Found" | Market does not exist |
| `ORDERBOOK_NOT_FOUND` | "Orderbook Not Found" | Orderbook does not exist |
| `TOKEN_PAIR_MISMATCH` | "Token Pair Mismatch" | Token pair doesn't match orderbook |
| `INSUFFICIENT_MARKET_FEE_BUFFER` | "Insufficient Market Fee Buffer" | Not enough market fee buffer |
| `SIGNATURE_EXPIRED` | "Signature Expired" | Order signature has expired |

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
| `MaxRetriesExceeded` | Never produced by the SDK itself: the HTTP retry loop propagates the final underlying error on exhaustion (structured details intact — see the retry-exhaustion tests). Kept public for consumer-built retry loops |

## Retry Strategy

- **Replay-safe requests**: GETs and idempotent set operations such as favorite-market updates use `RetryPolicy.Idempotent`, which retries transport failures and 502/503/504 and backs off on 429 with exponential backoff + jitter.
- **Non-idempotent requests** (order submit, cancel, auth): `RetryPolicy.None` - no automatic retry, which prevents duplicate side effects.
- Customizable per-call with `RetryPolicy.custom(config)`.

### Credential restoration (401 recovery)

Sessions built on short-lived tokens expire mid-run: the backend starts answering 401 even though the app could mint a fresh token (e.g. a browser refreshing its Privy session). Rather than every caller hand-rolling "detect 401 → refresh → retry", the transport accepts a host-supplied hook:

```typescript
client.setCredentialRestorer(async () => {
  // e.g. ask the auth provider's SDK to refresh the session
  return await refreshSession();
});
```

When a request to the API origin fails with HTTP 401 and a restorer is registered, the transport consults it **at most once per logical request**, with concurrent 401s sharing one restoration (bounded by a 30-second timeout). A successful restoration replays the request once **only if it declared itself retry-safe** (an idempotent/custom retry policy); `RetryPolicy.None` requests — mutations like orders and cancels — are never auto-replayed: the restoration still heals the session for the caller's next attempt, but the original 401 propagates. Restoration is skipped for credential-management endpoints (login, logout) and for cookie-override/custom-session requests, redirects are never followed on the API transport, and without a registered restorer 401s propagate unchanged. A timed-out restoration has its `AbortSignal` fired — promises cannot be cancelled, so restorers whose work is non-idempotent (refresh-token rotation) must honor the signal or serialize internally.

The SDK stays credential-agnostic: what "restore" means belongs to the host. For classifying auth failures in your own code, use `isUnauthorized(error)` from the error module — it covers both bare 401s and 401s carrying a structured rejection envelope (`ApiRejectedDetails.httpStatus`).

## Trigger Orders

Trigger orders (stop-limit, take-profit-limit) are under development and not yet available. Internal types exist in the source for internal use only.
