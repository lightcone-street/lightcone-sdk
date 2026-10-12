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
- [API Key](#api-key)
- [Environment Configuration](#environment-configuration)
- [Examples](#examples)
- [Error Handling](#error-handling)
- [Solana v1 transactions](#solana-v1-transactions)
- [Transaction Fee Funding](#transaction-fee-funding)
- [Retry Strategy](#retry-strategy)

## Installation

```bash
npm install @lightconexyz/lightcone-sdk
```

## Solana v1 transactions

All transaction builders and submission methods now use `V1Transaction`. Legacy
and v0 envelopes are rejected. Instruction builders still return web3.js
`TransactionInstruction`, and addresses remain web3.js `PublicKey` values.
The official `@solana/kit` 8.0.0 codec owns v1 wire serialization.

Configure explicit resources before using fluent submission or SOL planners:

```typescript
const client = LightconeClient.builder()
  .nativeSigner(keypair)
  .transactionResources({
    computeUnitLimit: 200_000,               // choose the limit for this workload
    loadedAccountsDataSizeLimit: 65_536,     // bytes
    priorityFeeLamports: 0n,                 // total lamports, not a CU price
  })
  .build();
const context = await client.transactionContext();
const tx = client.positions().depositToGlobal()
  .user(keypair.publicKey).mint(mint).amount(1_000_000n)
  .buildTx(context);
const confirmed = await client.signAndSubmitTxConfirmedWithSlot(tx);
```

`client.transactionContextWithResources(resources)` supplies limits for one
context. Direct `build*Tx` instruction helpers take a required context before the
optional program ID; domain `*Tx` methods and fluent `buildTx` take it last.
Use `V1Transaction.compile(instructions, payer, context)` to compose instructions.
Resources have no SDK defaults: compute units must be 1–1,400,000, loaded account
data 1–67,108,864 bytes, optional heap 32–256 KiB in 1-KiB increments, and priority
fees unsigned 64-bit `bigint` total lamports. Context expiry heights are exact
positive safe integers returned alongside their blockhash.

The transaction owns its immutable message and includes every required signature
slot when enforcing the 4,096-byte limit. It supports at most 64 distinct addresses,
12 signers, and 64 instructions. ComputeBudget instructions and address lookup
tables are unsupported. Split large account-initialization requests into batches;
an instruction valid by itself can exceed a transaction's account or byte limit.
`toWireBytes()` and `messageBytes()` return copies. `sign([keypair, ...])` returns a
new transaction; `fromWireBytes(bytes, context)` validates canonical imports.

Wallet adapters must implement `ExternalSigner.signTransaction` with v1 byte
support, return every required valid Ed25519 signature, and preserve the exact
message, including resources, fee payer, accounts, instructions, and blockhash.
Unsponsored external signers must report their `walletAddress`. Both shared and
raw Privy transaction methods fail before network activity because the current
Privy sign-and-send endpoint cannot expose its signed bytes for validation; use
an `ExternalSigner` that returns signed v1 bytes. Privy order signing is not
supported either: order submission and cancellation require a native signer or
an `ExternalSigner`.

Before prompting a wallet the SDK checks the cluster's v1 feature activation.
Submission verifies all signatures, checks activation again, simulates the exact
signed bytes with `sigVerify: true` and `replaceRecentBlockhash: false`, then sends
once with preflight enabled and `maxRetries: 0`. There is no send retry or RPC
failover. Definite request or preflight rejections return `SubmissionRejected`
with `signature` and `rpcCode`. `AlreadyProcessed` and uncertain sends return
`SubmissionUnknown`, preserving `signature` and `lastValidBlockHeight` for
reconciliation. Confirmation uses the original expiry for ordinary and prepared
transactions. Rebuild and obtain new signatures when the message or budget changes;
a timeout or expired status does not authorize automatic resubmission.

Canonical v1 RPC uses a direct `fetch` transport to avoid web3.js's internal
rate-limit retry. Fee estimation, feature checks, and simulation use read-only
retry and failover with their exact parameters. Sends use one HTTP attempt.
Use `.rpcFetch(customFetch)` on the builder when RPC access needs custom headers or
a transport override. The supplied transport must also avoid resending requests.
Existing account reads and confirmation retain their read-only failover behavior.

Legacy `Connection`
constructor headers/middleware are not copied into the v1 transport; configure
v1 authentication in `rpcFetch`, including when supplying your own Connection.
SDK-managed connections (including clones and backups) share that fetch for
blockhash acquisition and other reads.

See the [cross-language transaction ADR](../docs/adr/0004-solana-v1.md).

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
for wallet-adapter transaction signing. The default is false, each transaction
captures its signer and capability before asynchronous RPC work, `clone()` copies
the current value, and local-keypair submission rejects an enabled capability. Off-chain order-message signing is outside this contract. Privy transaction
submission is unavailable until signed v1 bytes can be verified.

## Quick Start

This example authenticates and subscribes to public market data. For funded order submission, use the [trading example](examples/submit_order.ts).

```typescript
import { Keypair } from "@solana/web3.js";
import {
  LightconeClient,
  auth,
} from "@lightconexyz/lightcone-sdk";

async function main() {
  const client = LightconeClient.builder().build();
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

  // 3. Subscribe to real-time updates.
  const ws = client.ws();
  await ws.connect();
  ws.subscribe({ type: "book_update", orderbook_ids: [orderbook.orderbookId] });
}

main().catch(console.error);
```

## Start Trading

These excerpts use top-level `await` in an ES module. Use a funded keypair for the selected environment. The [submit example](examples/submit_order.ts) confirms the deposit and waits for the API balance before ordering.

```typescript
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  LightconeClient,
  DepositSource,
  auth,
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
  .depositSource(DepositSource.Global)
  .transactionResources({
    computeUnitLimit: 200_000,
    loadedAccountsDataSizeLimit: 1_048_576,
    priorityFeeLamports: 0n,
  })
  .build();
const nonce = await client.auth().getNonce();
const signed = auth.signLoginMessage(keypair, nonce);
await client.auth().loginWithMessage(
  signed.message, signed.signature_bs58, signed.pubkey_bytes
);
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

The amount is in the deposit mint's smallest units. Instruction construction alone does not transfer collateral. Submit and confirm the transaction, then wait for the API balance to cover the order. Select the deposit mint backing the orderbook's quote token, as shown in the [submit example](examples/submit_order.ts).

Calculate the deposit from the same price and size used by the order. Poll at most 15 times, with two seconds between unsuccessful reads. A timeout or read failure stops before order submission. Check the confirmed deposit before repeating this step.

```typescript
import { shared } from "@lightconexyz/lightcone-sdk";
import { program } from "@lightconexyz/lightcone-sdk";

const rules = await client.orderbooks().decimals(orderbook.orderbookId);
const orderPrice = "0.55"; // Quote tokens per base token.
const orderSize = "1"; // Base tokens.
const quoteAtoms = shared.scalePriceSize(
  orderPrice, orderSize, program.OrderSide.BID, rules
).quoteAtoms;
const depositMint = new PublicKey(orderbook.quote.depositAsset);
const depositContext = await client.transactionContext();
const depositTx = client.positions().deposit()
  .user(keypair.publicKey)
  .mint(depositMint)
  .amount(quoteAtoms)
  .buildTx(depositContext);
await client.signAndSubmitTxConfirmedWithSlot(depositTx);

// Confirmation can precede indexing. Check the available global collateral.
for (let attempt = 0; attempt < 15; attempt++) {
  const snapshot = await client.positions().depositTokenBalances();
  const entry = Object.values(snapshot.balances).find(
    (balance) => balance.mint === depositMint.toBase58()
  );
  const idleAtoms = shared.exactScaledInteger(entry?.idle ?? "0", rules.quoteDecimals);
  if (idleAtoms >= quoteAtoms) break;
  if (attempt === 14) {
    throw new Error("Global collateral is not indexed yet; order was not submitted");
  }
  await new Promise((resolve) => setTimeout(resolve, 2_000));
}
```

### Step 3: Place an Order

```typescript
const envelope = client.orders().limitOrder()
  .maker(keypair.publicKey)
  .bid()
  .price(orderPrice)
  .size(orderSize);
const order = await envelope.submit(client, orderbook);
console.log(order.status, order.state?.open_base);
```

An order's identity is its signed fields; the `u64` salt is the only
distinguishing value, so the envelope draws one salt on first use and keeps it.
Submission is never retried. If the outcome is unknown (`ENGINE_UNAVAILABLE`, a
timeout, or a lost connection), call `envelope.submit(client, orderbook)` again
on the same envelope, which resubmits the identical order: a `DUPLICATE_ORDER`
rejection or an `ALREADY_EXISTS` error proves the first submission was accepted,
and any other result is the outcome of this one. Never sign a replacement with a
new salt, which is a different order. Order reads list only open and pending
orders, so they cannot prove an order was never accepted.

The response reports the committed order: `status` is `accepted`,
`accepted_pending`, or `filled`; `state` holds the `original`, `confirmed`,
`pending`, `open`, `executable`, and `cancelled` base quantities (decimal
strings in base-token units) with the `committed_revision`; and `fills` lists up
to 16 authenticated fills with signed fee estimates in raw fee-mint atoms.
`state` and `initial_cohort` are `undefined` when the acceptance is durable but
its committed view is not yet readable.

### Step 4: Monitor

```typescript
import { asPubkeyStr } from "@lightconexyz/lightcone-sdk";

const open = await client
  .orders()
  .getUserOrders(50);
const ws = client.ws();
await ws.connect();
ws.subscribe({ type: "book_update", orderbook_ids: [orderbook.orderbookId] });
ws.subscribe({
  type: "user",
  wallet_address: asPubkeyStr(keypair.publicKey.toBase58()),
});
```

The `user` channel sends one snapshot, then committed facts tagged by
`event_type`: `order` (an order's complete committed state), `funding` (funding
account balances, reservations, and live `available`/`ready`), `fill` (the
`trades` payload for the wallet's fills), `closure` (an accepted-order cutoff),
`recovery_completed`, and `notification`. Retired and future event types decode
as `{ event_type: "unknown" }`. Live facts carry `commit` (`effect_id`,
`committed_revision`, `projection_generation`, `actionable`); revisions and
sequences are `bigint`.

Seed `UserOpenLimitOrders` with `convertSnapshotOrders(snapshot.orders)` (merge
later REST pages with `limitOrderFromUserOrder` and `applyOrder`), then call
`apply(update)` for every `order` fact and `applyClosure(closure)` for every
`closure`. Application is revision-guarded, so older state cannot reopen a closed
order. `applyClosure` returns `undefined` for scopes it cannot evaluate from order
data (deposit token, account, settlement profile): refetch the orders then.
`recovery_completed` means the engine rebuilt committed state and cached user
state is stale, but no snapshot follows it: `clear()` the container, then
unsubscribe and resubscribe the `user` channel for a new snapshot, or reseed from
REST pages. After a reconnect the client resubscribes automatically; `clear()` the
container and reseed from the snapshot that follows.

Book streams are snapshot-only: every accepted `book_update` replaces the full
top-20 view. `OrderbookState` discards equal/older `seq` values within a
subscription generation and accepts forward gaps. Call `beginGeneration()` on
reconnect/resubscribe; `resync: true` (a frame without `seq`) requires
unsubscribe/resubscribe with the same aggregation. Each frame's `ready` flag
reports whether the engine's committed book is live for matching, and
`OrderbookState.ready` tracks the last accepted frame (REST depth carries the
same `ready`). Each `(orderbook, aggregation)` pair needs its own state.
Truncation flags are preserved and mean that side is not exhaustive. See
[`examples/ws_book_and_trades.ts`](examples/ws_book_and_trades.ts).
Each decoded bid and ask level includes exact decimal-string `quote_notional`.
For grouped books, `price` is a display bucket boundary, so quote liquidity
and totals must use `quote_notional` rather than `price * size`. The
price-to-base-size `OrderbookState` maps do not retain quote notional; read it
from the decoded `OrderBook` levels.
Ticker consumers should use the supplied `mid`; it is engine-authoritative and
may use one-sided-book or last-trade fallback. The committed backend sends only
the initial ticker snapshot, and the market channel has no producer.
REST depth is a coherent projection that may briefly lag a mutation. Use its
`revision` and `captured_at_ms` metadata, and expect revision gaps.

```typescript
ws.subscribe({ type: "book_update", orderbook_ids: [orderbook.orderbookId], nSigFigs: 5, mantissa: 2 });
```

Order submission accepts decimal strings and uses exact `bigint` construction.
`submit()` fetches and caches `/api/orderbooks/{id}/decimals` before invoking a
signer. Direct `sign()`/`finalize()` calls require the returned `OrderbookRules`.
Raw amounts, explicit salts, and derived prices are preflighted against the same
admission rules and the nonzero `u64` atom range; no tick or size normalization
is implicit.

The envelope generates a salt when omitted. The price is quote tokens per base token, and the size is base tokens. Example values must satisfy the selected orderbook's trading rules and available collateral.

### Wallet Balances and SOL Action Planning

`depositTokenBalances()` returns a required exact nine-decimal
`native_sol_balance` alongside the separate mint-keyed SPL map. Initialize
`WalletDepositBalancesState` with `applyRestSnapshot(wallet, snapshot)` and feed
the outer `wallet_deposit_balances` channel's nested snapshot, absolute SPL,
absolute native-SOL, and status events to `applyEvent()`. Both methods accept an
optional minimum snapshot slot; only a complete snapshot below that floor is
ignored without mutation, while an equal slot and every balance/status event
keep their existing behavior. Without a floor, complete snapshots replace state
even after a higher incremental-update slot. Status and wrong-wallet events do not mutate
it, pre-baseline balance updates are ignored, and explicit-zero SPL updates
remove their mint. Matching SPL updates with invalid or negative
idle balances return `rejected` without mutation. `contextSlot` records the latest accepted
balance observation rather than enforcing global monotonic ordering.
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
`unwrapAllSolBalanceAvailability(breakdown, costs)` validates that complete
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
// program and asPubkeyStr were imported in the preceding steps.

const signature = program.signCancelOrder(order.order_hash, keypair);
await client.orders().cancel({
  order_hash: order.order_hash,
  maker: asPubkeyStr(keypair.publicKey.toBase58()),
  signature,
});
```

### Step 6: Exit a Position

Merge only when the wallet holds a complete set of outcome tokens for this deposit mint. Cancelling an unfilled order does not create that set. To return unused global collateral, proceed to withdrawal.

```typescript
// Uses the explicit transaction resources configured on the client.
// The signature means RPC acceptance; confirm before dependent account operations.
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

Native clients running in Node follow the [native client recovery guide](../docs/auth-session-recovery.md) for session teardown and WebSocket recovery. It explains per-token logout, incomplete teardown, anonymous public continuity, and finite reconnect budgets. Browser clients use Privy authentication.

Authentication is required for user-specific endpoints. Native clients using Lightcone sessions fetch `/api/auth/nonce`. Sign the exact message `Sign in to Lightcone\nNonce: {nonce}` with ED25519. Use `auth.signLoginMessage` to construct this challenge, then exchange the signed message for a session. Do not sign a timestamp or the nonce alone.

Authenticate before connecting a private WebSocket. Native clients running in Node send the Lightcone session cookie during the upgrade. Browser clients supply their Privy cookie automatically. Private user subscriptions require `wallet_address`. Derive the Trading Wallet with `tradingWallet(session.user, session.auth_method)`. Refer to the [authenticated streaming example](examples/ws_user_and_market.ts).

Privy hosts can also authenticate with passwordless Email, Google, X, or Wallet. After every interactive success, call `client.auth().registerPrivy({ attempted_identity })`. The backend validates the exact selector against Privy's verified methods, creates or synchronizes the Account, and changes the Primary Login Identity only for a new Account. `session.user.identity` is that stable primary; `session.user.linked_identities` contains every connected method with primary first.

Use `walletDisplayName(session.user, session.auth_method)` to show a shortened
label for the wallet the session trades with, regardless of login identity.

`session.user.max_slippage_preference` is an exact decimal string strictly below
`10`, or `null` until one is stored. Persist a value greater than zero and less
than 10 using `client.auth().updateMaxSlippagePreference(value)`; the method
returns the canonical exact decimal string. Values at or above 10% remain valid
order protection but are not remembered through this API.

`session.user.telegram_invite_url` is the single-use Telegram group invite URL that the backend assigned to the Account. It is `null` when the Account holds no invite or when an older backend omits the field. The URL is a secret of the Account. Do not log it and do not send it to analytics.

### Cookie handling

After login succeeds, the SDK stores the session token internally and attaches it as `Cookie: lightcone-token=…` on every authenticated request. Behaviour depends on the runtime:

- **Node / non-browser**: token is stored on the `LightconeHttp` instance and added as a `Cookie` header per request.
- **Browser**: requests use `credentials: "include"` and the runtime supplies the cookie automatically — the SDK's internal store is unused.

### Server-side cookie forwarding (`*WithCookies` variants)

Ordinary authenticated methods use the client's session or the browser cookie. Server-side rendering and route handlers must pass the incoming raw `Cookie` header to the corresponding `*WithCookies` method.

The cookie name is `lightcone-token`. Do not install a user's token on a client shared between requests from different users.

> **Behavior change.** `getWithCookies` responses no longer capture `Set-Cookie` into the shared token slot (they previously did): a forwarded per-user request rotating its token must not leak that token to every later request from a shared server client. These requests also never consult the credential restorer.

These methods forward the supplied header for one call:

```typescript
// cookieHeader is the incoming raw Cookie header, including lightcone-token=...
const snapshot = await client
  .positions()
  .depositTokenBalancesWithCookies(undefined, cookieHeader);
console.log(`snapshot slot ${snapshot.context_slot}: ${Object.keys(snapshot.balances).length} balances`);

const positions = await client
  .positions()
  .positionsWithCookies(cookieHeader);
```

In browsers, use the ordinary methods. The browser supplies the cookie through `credentials: "include"`.

## API Key

Configure an API key for native or server-side submit, cancel, and cancel-all requests. Public reads and authentication need no API key. The key identifies an API Consumer; user authentication, wallet ownership, and signed-order validation still apply.

```ts
const client = LightconeClient.builder()
  .env(LightconeEnv.Staging)
  .apiKey(process.env.LIGHTCONE_API_KEY!)
  .build();
```

The SDK attaches `x-lightcone-api-key` only to POST `/api/orders/submit`, `/api/orders/cancel`, and `/api/orders/cancel-all` on the configured API origin. It never forwards the key to another origin or logs it. HTTPS is required except for loopback development. Keep keys out of source control and browsers. Browser builds reject API-key configuration; web users trade with their existing Privy session.

Cloudflare limits these trading attempts by public source IP, not by API key: direct API callers share 500 attempts per 10-second window per IP. A 429 response does not revoke credentials. Public reads and WebSocket messages are outside this trading limit. Revocation reaches all backend process caches within 24 hours; new keys work immediately through a database lookup on cache miss.

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
| [`positions`](examples/positions.ts) | Committed funding accounts across all markets and per-market |
| [`metrics_all`](examples/metrics_all.ts) | Exercise every endpoint on `client.metrics()` - platform, markets, categories, orderbook, leaderboard, history |

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.ts) | Deposit collateral, then place an exactly validated limit order using cached trading rules. Companion `cancel_order` cancels it and withdraws to stay net-neutral |

### Cancelling Orders

| Example | Description |
|---------|-------------|
| [`cancel_order`](examples/cancel_order.ts) | Cancel a single order by hash, cancel all orders in an orderbook, and withdraw the released collateral from the global pool |
| [`user_orders`](examples/user_orders.ts) | Fetch open and pending orders and funding accounts for an authenticated user |

### On-Chain Operations

| Example | Description |
|---------|-------------|
| [`read_onchain`](examples/read_onchain.ts) | Read exchange, market, orderbook, and position state and PDA derivations via RPC |
| [`onchain_transactions`](examples/onchain_transactions.ts) | Build, sign, and submit mint/merge complete set on-chain |
| [`global_deposit_withdrawal`](examples/global_deposit_withdrawal.ts) | Deposit to the global pool, move collateral into a market, withdraw from global, and merge the complete set back |

Every instruction builder appends the program's event transport trailer (the
event-authority PDA, then the program account) automatically, and public
instructions require transaction-level invocation except for the governance CPI allowlist. See the
[program module README](src/program/README.md#event-transport-trailer).

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
| [`ws_user_and_market`](examples/ws_user_and_market.ts) | Read-only authenticated user/market subscriptions plus one reduced Trading Wallet balance snapshot |

## Error Handling

All SDK operations reject with `SdkError`:

| Variant | When |
|---------|------|
| `Http` | REST request failures |
| `Ws` | WebSocket connection/protocol errors |
| `Auth` | Authentication failures |
| `Validation` | Invalid v1 resources/messages, unsupported cluster or wallet capabilities, and domain conversion failures |
| `InsufficientSolForTransactionFees` | Proven fee-payer shortfall, with exact available and required lamports |
| `Serde` | Serialization errors, and a 2xx body that does not match the expected shape (the message names the field) |
| `MissingMarketContext` | Market context not provided for an operation requiring `DepositSource.Market` |
| `Signing` | Signing operation failures |
| `UserCancelled` | User cancelled wallet signing prompt |
| `SubmissionRejected` | RPC rejected the request or preflight before queuing; inspect `signature`, `rpcCode`, and the error message |
| `SubmissionUnknown` | Send outcome is uncertain; inspect `signature` and `lastValidBlockHeight` before rebuilding |
| `TransactionFailed` | Transaction confirmed with an on-chain error |
| `TransactionExpired` | Expiry is past and final history evidence has no signature; reconcile before rebuilding |
| `ConfirmationTimeout` | Confirmation could not establish the outcome |
| `ApiRejected` | Backend rejected the request (see [API Rejections](#api-rejections)) |
| `Program` | On-chain program errors (RPC, account parsing) |
| `Other` | Catch-all |

### API Rejections

When the backend rejects a request (insufficient balance, expired order, etc.), the SDK throws `SdkError` with `variant === "ApiRejected"`. The structured details are available on `error.apiRejectedDetails`:

| Field | Type | Description |
|-------|------|-------------|
| `reason` | `string` | Human-readable error message |
| `rejectionCode` | `RejectionCode \| undefined` | Machine-readable rejection code (see below) |
| `errorCode` | `string \| undefined` | API-level error code (for example `"NOT_FOUND"` or `"INVALID_ARGUMENT"`); `errorCodeKind()` returns it as a typed `ErrorCode` |
| `errorLogId` | `string \| undefined` | Backend support correlation ID (`LCERR_*`) |
| `requestId` | `string \| undefined` | SDK-generated `x-request-id` for cross-service tracing |
| `existingMethod` | `string \| undefined` | Primary method of the conflicting Account when identity ownership has one deterministic owner |

`ApiRejectedDetails.toString()` formats all present fields as a multi-line report for logs or support tickets.

#### `RejectionCode`

Business outcomes of a trading mutation, delivered with HTTP 200 and a `rejection_code`. Each has a human-readable `.label()`; `.known()` returns the code (or `undefined` for one this SDK version does not know, which keeps its raw text). `.isTransient()` is true for `STALE_STATE`, `REQUEST_EXPIRED`, `TRADING_CAPACITY_UNAVAILABLE`, and `TRADING_NOT_READY`, where the same signed order may be submitted again.

| Wire code | Label | When |
|-----------|-------|------|
| `DUPLICATE_ORDER` | "Duplicate Order" | This signed order identity was already accepted |
| `INSUFFICIENT_BALANCE` | "Insufficient Balance" | Requested funding cannot cover the complete order |
| `FOK_INSUFFICIENT_LIQUIDITY` | "FOK Insufficient Liquidity" | A fill-or-kill order's full base target is not executable |
| `SELF_TRADE` | "Self Trade" | The order would cross the same wallet's liquidity |
| `INVALID_ORDER` | "Invalid Order" | Price/size precision, tick, or minimum-size rule violated |
| `INVALID_REQUEST` | "Invalid Request" | The request exceeds supported limits |
| `ORDER_EXPIRED` | "Order Expired" | The signed order had already expired |
| `TRADING_PAUSED` | "Trading Paused" | Trading is paused for the market, exchange, or deposit token |
| `INVALID_SIGNATURE` | "Invalid Signature" | The mutation signature did not verify |
| `CANCEL_ALL_REPLAY` | "Cancel-All Replay" | A cancel-all salt was already consumed |
| `STALE_STATE` | "Stale State" | Trading state changed while processing |
| `REQUEST_EXPIRED` | "Request Expired" | The request expired before acceptance |
| `TRADING_CAPACITY_UNAVAILABLE` | "Trading Capacity Unavailable" | Capacity unavailable, including the 250-open-order wallet cap |
| `TRADING_NOT_READY` | "Trading Not Ready" | Committed trading state is not ready |
| `INTERNAL_ERROR` | "Internal Error" | Committed order state could not be read |
| `ORDER_NOT_FOUND` | "Order Not Found" | Cancel of an order hash that was never accepted |
| `ORDERBOOK_NOT_FOUND` | "Orderbook Not Found" | The engine does not know the orderbook |

#### `ErrorCode`

Transport, validation, authorization, and availability failures carry an `error_code` on a 4xx/5xx envelope. `ApiRejectedDetails.errorCodeKind()` returns it as a typed `ErrorCode` (`.known()` is `undefined` for a code this SDK version does not know); `.isRetryable()` marks the conditions that may clear on their own.

| Code | HTTP | When |
|------|------|------|
| `INVALID_ARGUMENT` | 400 | Malformed field, bad side, zero amount, non-canonical key, bad cancel-all salt or timestamp |
| `FAILED_PRECONDITION` | 400 | Engine precondition failed outside the business-rejection path |
| `FORBIDDEN` | 403 | Mutation signature rejected (a stale cancel-all timestamp is `INVALID_ARGUMENT`) |
| `AUTH_REQUIRED` | 401 | Order mutation without a session |
| `AUTH_WALLET_MISMATCH` | 403 | Session wallet differs from the order maker / cancellation wallet |
| `ALREADY_EXISTS` | 409 | Duplicate order identity or reused cancel-all salt |
| `ABORTED` | 409 | Trading state changed; retry |
| `RESOURCE_EXHAUSTED` | 429 | Engine admission queue or capacity exhausted |
| `ENGINE_UNAVAILABLE` | 503 | Engine unavailable. For submission the outcome is unknown: resubmit the identical signed request, where `DUPLICATE_ORDER` or `ALREADY_EXISTS` proves the first was accepted; never re-sign with a new salt |
| `TRADING_UNAVAILABLE` | 503 | Committed trading state or orderbook metadata unavailable |
| `ENGINE_INTERNAL_ERROR` | 500 | Engine failure without a public reason |
| `INVALID_TIF`, `INVALID_DEPOSIT_SOURCE`, `INVALID_SIGNATURE`, `INVALID_PUBKEY`, `INVALID_CURSOR`, `INVALID_ORDER_HASH`, `UNEXPECTED_QUERY`, `INVALID_LIMIT` | 400 | Request validation |
| `NOT_FOUND` | 404 | Resource not found |
| `ORDERBOOK_DECIMALS_NOT_FOUND` | 404 | No decimals or trading rules for the orderbook. `Orders.submit` fetches these first, so an unknown orderbook fails here before reaching the submit route |
| `ORDERBOOK_CONFIGURATION_ERROR` | 500 | The orderbook's stored trading rules are invalid |
| `NON_CANONICAL_ENCODING` | 400 | Percent-encoded unreserved characters on a route that rejects them |
| `RATE_LIMITED` | 429 | Request rate limit |

Some failures carry no envelope at all: request-body extractor errors (for example an unknown field such as `nonce`) answer `400 text/plain`, and authenticated GETs without a session answer `401 text/plain`. These surface as `HttpError` `BadRequest` (with the backend text) and `Unauthorized` (`isUnauthorized(error)` covers both 401 forms). A 2xx body that does not match the expected response shape surfaces as `SdkError` `Serde`, naming the offending field.

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

## Supported order responses

Refer to the [shared order-response contract](../README.md#supported-order-responses). REST user orders (`UserOrder`, with its recorded quantities in `state`), WebSocket snapshot orders (`UserSnapshotOrder`), and live `order` facts (`OrderUpdate`) all carry an order's committed state and convert to `LimitOrder` with `limitOrderFromUserOrder` (`undefined` when the recorded quantities are absent), `limitOrderFromSnapshot`, and `limitOrderFromUpdate`. Their pages also carry funding accounts (`FundingAccount`) and the committed revision; there is no account nonce. `convertSnapshotOrders` returns `UserOpenLimitOrders`.

Unit conventions: quantities named `*_base` and order sizes are decimal strings in base-token units, except `CancelQuantities`, whose `*_base` fields are raw base-token atoms (`bigint`); `amount_in`/`amount_out` and `maker_amount`/`taker_amount` are in the units of the token each side gives or receives; prices are quote units per base unit; fields ending in `_atoms` are raw integer atoms (`bigint`); order and trade revisions, sequences, generations, and slots are `bigint`. Wallet deposit-balance `context_slot` values remain safe-integer `number`s. Funding reservations are `DecimalText` because they can exceed 96-bit decimal range; `decimalTextToDecimal` returns `undefined` beyond it, as Rust's `to_decimal()` does.
