# Lightcone SDK

Python SDK for the Lightcone impact market protocol on Solana.

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

## Installation

```bash
pip install git+https://github.com/lightcone-street/lightcone-sdk.git@prod#subdirectory=python
```


## Quick Start

```python
import asyncio
import json
from pathlib import Path

from solders.keypair import Keypair

from lightcone_sdk import LightconeClientBuilder, LightconeEnv
from lightcone_sdk.auth.client import sign_login_message
from lightcone_sdk.ws.subscriptions import BookUpdateParams


async def main():
    # Defaults to Prod. Use .env(LightconeEnv.STAGING) for staging.
    client = (
        LightconeClientBuilder()
        .build()
    )
    with Path("~/.config/solana/id.json").expanduser().open() as f:
        secret = json.load(f)
    keypair = Keypair.from_bytes(bytes(secret))

    # 1. Authenticate
    nonce = await client.auth().get_nonce()
    message, signature_bs58, pubkey_bytes = sign_login_message(keypair, nonce)
    await client.auth().login_with_message(
        message,
        signature_bs58,
        pubkey_bytes,
    )

    # 2. Find a market
    market = await client.markets().get_by_slug("some-market")
    orderbook = market.orderbook_pairs[0]

    # 3. Fetch/cache trading rules, validate exactly, sign, and submit
    #    market, base_mint, quote_mint, and nonce are auto-filled from the orderbook.
    response = await (
        client.orders().limit_order()
        .maker(keypair.pubkey())
        .bid()
        .price("0.55")
        .size("100")
        .submit(client, orderbook)
    )
    print("Order submitted:", response)

    # 4. Stream real-time updates
    ws = client.ws()
    await ws.connect()
    await ws.subscribe(BookUpdateParams(orderbook_ids=[orderbook.orderbook_id]))

    await ws.disconnect()
    await client.close()


asyncio.run(main())
```

## Start Trading

```python
import json
from pathlib import Path

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk import LightconeClientBuilder

with Path("~/.config/solana/id.json").expanduser().open() as f:
    secret = json.load(f)
keypair = Keypair.from_bytes(bytes(secret))

client = (
    LightconeClientBuilder()
    .native_signer(keypair)
    .build()
)
```

## Environment Configuration

The SDK defaults to the **production** environment. Use `LightconeEnv` to target a different deployment:

```python
from lightcone_sdk import LightconeClientBuilder, LightconeEnv

# Production (default)
prod_client = LightconeClientBuilder().build()

# Staging
staging_client = (
    LightconeClientBuilder()
    .env(LightconeEnv.STAGING)
    .build()
)

# Local development
local_client = (
    LightconeClientBuilder()
    .env(LightconeEnv.LOCAL)
    .build()
)
```

Each environment configures the API URL, WebSocket URL, Solana RPC URL, and on-chain program ID automatically. Individual overrides such as `.base_url()`, `.ws_url()`, and `.rpc_url()` still take precedence when called after `.env()`.

The Solana RPC URL can also be overridden via the `SDK_RPC_URL` environment variable, which takes precedence over the environment default. This is useful for pointing all examples at a private RPC to avoid public devnet rate limits.

### Step 1: Find a Market

```python
market = await client.markets().get_by_slug("some-market")
orderbook = next(
    (pair for pair in market.orderbook_pairs if pair.active),
    market.orderbook_pairs[0],
)
```

### Step 2: Deposit Collateral

```python
deposit_mint = Pubkey.from_string(market.deposit_assets[0].deposit_asset)
deposit_ix = (client.positions().deposit()
    .user(keypair.pubkey())
    .mint(deposit_mint)
    .amount(1_000_000)
    .market(market)
    .build_ix())
```

### Step 3: Place an Order

```python
order = await (
    client.orders().limit_order()
    .maker(keypair.pubkey())
    .bid()
    .price("0.55")
    .size("2")
    .submit(client, orderbook)
)
```

### Step 4: Monitor

```python
from lightcone_sdk.ws.subscriptions import BookUpdateParams, UserParams

open_orders = await client.orders().get_user_orders(str(keypair.pubkey()), 50)
ws = client.ws()
await ws.connect()
await ws.subscribe(BookUpdateParams(orderbook_ids=[orderbook.orderbook_id]))
await ws.subscribe(UserParams(wallet_address=str(keypair.pubkey())))
```

Book streams are snapshot-only: every accepted `book_update` replaces the full
top-20 view. `OrderbookState` discards equal/older `seq` values within a
subscription generation and accepts forward gaps. Call `begin_generation()` on
reconnect/resubscribe; `resync: true` requires unsubscribe/resubscribe with the
same aggregation. Each `(orderbook, aggregation)` pair needs its own state.
Truncation flags are preserved and mean that side is not exhaustive. See
[`examples/ws_book_and_trades.py`](examples/ws_book_and_trades.py).
Each decoded bid and ask level includes exact decimal-string `quote_notional`.
For grouped books, `price` is a display bucket boundary, so quote liquidity
and totals must use `quote_notional` rather than `price * size`. The
price-to-base-size `OrderbookState` dictionaries do not retain quote notional;
read it from the decoded `WsOrderBook` levels.
Ticker consumers should use the supplied `mid_price`; it is
engine-authoritative and may use one-sided-book or last-trade fallback.
REST depth is a coherent projection that may briefly lag a mutation. Use its
`revision` and `captured_at_ms` metadata, and expect revision gaps.

Order submission accepts strings or exact `Decimal` values and uses Python
integer arithmetic. `submit()` fetches and caches
`/api/orderbooks/{id}/decimals` before invoking a signer. Direct
`sign()`/`finalize()` calls require the returned `OrderbookRules`. Raw amounts,
explicit salts, and derived prices are checked against the same signed-64-bit
rules; no tick or size normalization is implicit.

```python
await ws.subscribe(BookUpdateParams(orderbook_ids=[orderbook.orderbook_id], n_sig_figs=5, mantissa=2))
```

### Step 5: Cancel an Order

```python
from lightcone_sdk import sign_cancel_order
from lightcone_sdk.domain.order import CancelBody

signature = sign_cancel_order(order.order_hash, keypair)
await client.orders().cancel(
    CancelBody(
        order_hash=order.order_hash,
        maker=str(keypair.pubkey()),
        signature=signature,
    )
)
```

### Step 6: Exit a Position

```python
# sign_and_submit builds the tx, signs it using the client's signing strategy, and submits
tx_hash = await (client.positions().merge()
    .user(keypair.pubkey())
    .market(market)
    .mint(deposit_mint)
    .amount(1_000_000)
    .sign_and_submit())
```

`market.num_outcomes` is the validated protocol outcome count. Market deposit, merge, and unified withdrawal use it instead of the length of display outcome metadata. The pubkey-only `withdraw_from_position()` builder requires `.num_outcomes(market.num_outcomes)` before building.

## Authentication
Authentication is only required for user-specific endpoints. Authentication is session-based using ED25519 signed messages. The flow is: request a nonce, sign it with your wallet, and exchange it for a session token.

Privy hosts can also authenticate with passwordless Email, Google, X, or Wallet. After every interactive success, call `await client.auth().register_privy(RegisterPrivyRequest(...))`. The backend validates the exact selector against Privy's verified methods, creates or synchronizes the Account, and changes the Primary Login Identity only for a new Account. `session.user.identity` is that stable primary; `session.user.linked_identities` contains every connected method with primary first.

Use `session.user.wallet_display_name(session.auth_method)` to show a shortened
label for the wallet the session trades with, regardless of login identity.

`session.user.max_slippage_preference` is an exact decimal string strictly below
`10`, or `None` until one is stored. Persist a value greater than zero and less
than 10 using `await client.auth().update_max_slippage_preference(value)`; the
method returns the canonical exact decimal string. Values at or above 10%
remain valid order protection but are not remembered through this API.

### Cookie handling

After login succeeds, the SDK stores the session token internally and attaches it as `Cookie: lightcone-token=…` on every authenticated request. The token lives on the `LightconeHttp` instance and is added per request.

### Server-side cookie forwarding (`*_with_auth` variants)

> **Naming note.** The `_with_auth` suffix does **not** mean other methods are unauthed — most SDK methods that talk to authed endpoints (e.g. `positions().positions()`, `metrics().user()`) read auth from the SDK's process-wide token store automatically; that's the typical client-side path. The `*_with_auth(auth_token)` siblings exist for **server-side rendering (SSR) and route-handler callers** where the per-request browser cookie can't propagate to the shared client. Those callers extract the token from the incoming request and pass it explicitly. Same wire contract, different credentials path.

When the SDK runs on a server (FastAPI, Starlette, etc.) and the *user's* `auth_token` cookie arrives on an incoming HTTP request, the SDK's process-wide token store is the wrong place to route it through — the store is shared across all users of that server process.

For these cases, authed methods that need per-call forwarding ship a `*_with_auth(auth_token)` sibling that injects the cookie just for that one call. The token is used only for that call and is **not** written back to the shared store, even if the backend rotates the cookie via `Set-Cookie`:

```python
# Inside a server route handler, after extracting the auth_token cookie
# from the incoming request:
snapshot = await client.positions().deposit_token_balances_with_cookies(
    None,
    auth_token,
)
print(f"snapshot slot {snapshot.context_slot}: {len(snapshot.balances)} balances")

positions = await client.positions().positions_with_auth(
    auth_token=auth_token,
)
```

### External wallet SOL balances

`deposit_token_balances()` returns a complete external-wallet snapshot. Native
SOL is the required exact nine-decimal `native_sol_balance` string and remains
separate from the mint-keyed SPL `balances` mapping. Use
`WalletDepositBalancesState` to apply REST snapshots and nested
`wallet_deposit_balances` WebSocket events, and to derive exact native plus
canonical WSOL without floating-point arithmetic:

The nested `wallet_deposit_balance_snapshot` replaces all stored SPL and native
state even after a higher component slot. `wallet_deposit_balance_update`
replaces one absolute SPL balance and removes explicit zero, while
`wallet_native_sol_balance_update` replaces the absolute native value rather
than applying a delta. Pre-baseline updates, wrong-wallet updates, and
`wallet_deposit_balance_status` do not mutate state. `context_slot` records the
latest accepted component observation rather than enforcing global monotonicity.
Matching SPL updates with invalid or negative idle balances return `REJECTED`
without changing balances or the context slot.

```python
import asyncio

from lightcone_sdk import WalletDepositBalancesState, WsEventType
from lightcone_sdk.ws.subscriptions import WalletDepositBalancesParams

snapshot = await client.positions().deposit_token_balances()
state = WalletDepositBalancesState()
state.apply_rest_snapshot(wallet_address, snapshot)
print(state.combined_sol_balance())

ws = client.ws()
params = WalletDepositBalancesParams(wallet_address=wallet_address)
updated = asyncio.Event()

def apply_wallet_event(event):
    if (
        event.type is WsEventType.MESSAGE
        and event.message is not None
        and event.message.type == "wallet_deposit_balances"
    ):
        state.apply_event(event.message.data)
        updated.set()

remove_listener = ws.on(apply_wallet_event)
try:
    await ws.connect()
    await ws.subscribe(params)
    await asyncio.wait_for(updated.wait(), timeout=10)
finally:
    remove_listener()
    await ws.disconnect()
```

`plan_sol_split`, `plan_sol_merge`, `plan_sol_redeem`, and
`plan_native_sol_withdrawal` return unsigned action plans with live fee/rent
costs, the action-specific reserve and spendable balance, and separate expected
native and canonical WSOL deltas. Each planner requires complete matching-wallet
state and fails closed when an account check, estimate, or native reserve is
unavailable. Unsponsored actions reserve the greater of live costs and the
applicable 0.001 SOL or 0.0035 SOL floor. Sponsored planning is rejected until a
concrete sponsor owns transaction fees and account rent.
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
account implicitly; an explicit self-custody unwrap-all/close operation is
outside the current contract. Rebuild immediately before signing,
submit with `sign_and_submit_prepared_tx_confirmed_with_slot` so the wallet
cannot replace the fee-estimated message, and refresh a complete snapshot
covering its slot before restoring action authority. Prepared submission is
unavailable for Privy because its final signed bytes cannot be verified by the
SDK. Atomic execution does not resolve uncertain submission or confirmation
errors; inspect authoritative balances before retrying. See the
[persistent canonical WSOL ADR](../docs/adr/0001-persistent-canonical-wsol.md).

WebSocket clients are owned independently from `Auth`; logout does not clean them
up. For each retained client, `clear_authed_subscriptions()` purges User/wallet
reconnect tracking and queued authenticated messages. It does not stop an
already-open server stream; send the matching unsubscribe or disconnect for live
teardown.

The `deposit_token_balances` example is manual-only and runs with
`LIGHTCONE_ENV=local` or `staging` only when `SDK_API_URL`, `SDK_WS_URL`,
`SDK_RPC_URL`, and `SDK_PROGRAM_ID` are all unset. It sends 0.001 SOL to the Rust
SDK wallet configured by `LIGHTCONE_WALLET_PATH` from the distinct sender
configured by `LIGHTCONE_WALLET_PATH_PYTHON`, confirms with a slot, and refreshes
a complete snapshot at that slot. Running it moves funds. If it fails after
submission, inspect authoritative balances before retrying because funds may
already have moved.

## Examples
All examples are runnable with `python examples/<name>.py`. Examples default to the production environment and read the wallet keypair from `~/.config/solana/id.json`. Set `LIGHTCONE_ENV=local|staging|prod` or `LIGHTCONE_WALLET_PATH=/path/to/keypair.json` to override.

The authenticated markets client provides paginated `favorite_markets(limit=None, cursor=None)`, `add_favorite_market(market_pubkey)`, and `remove_favorite_market(market_pubkey)`, plus `_with_cookies` variants for server-side cookie forwarding. Favorite pages include `next_cursor` and `has_more`; the backend defaults to 100 items and clamps limits to 1000. Add and remove are idempotent set operations, so the SDK may safely replay them after supported credential restoration or transient transport failures. Cookie-forwarding variants retry transient failures with the supplied cookie but never invoke the process-wide credential restorer. [`with_cookies`](examples/with_cookies.py) exercises these methods and restores the original favorite state.

### Setup & Authentication

| Example | Description |
|---------|-------------|
| [`login`](examples/login.py) | Full auth lifecycle: sign message, login, check session, logout |
| [`with_auth`](examples/with_auth.py) | Per-call auth-token forwarding for SSR / route-handler consumers — logs in, captures the token via `client.auth_token`, clears the SDK's internal store, and exercises every `*_with_auth` variant |

### Market Discovery & Data

| Example | Description |
|---------|-------------|
| [`markets`](examples/markets.py) | Featured markets, paginated listing, fetch by pubkey, search, platform deposit assets via `global_deposit_assets()` |
| [`orderbook`](examples/orderbook.py) | Fetch orderbook depth (bids/asks) and derive decimal precision metadata |
| [`trades`](examples/trades.py) | Recent trade history with cursor-based pagination (per-orderbook and market-wide) |
| [`price_history`](examples/price_history.py) | Historical price history line data at various resolutions |
| [`positions`](examples/positions.py) | User positions across all markets and per-market |
| [`deposit_token_balances`](examples/deposit_token_balances.py) | WebSocket-backed exact SOL balances and slot-confirmed 0.001 SOL native withdrawal without closing canonical WSOL in non-production |
| [`metrics_all`](examples/metrics_all.py) | Exercise every endpoint on `client.metrics()` - platform, markets, categories, orderbook, leaderboard, history |

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.py) | Deposit collateral, then place an exactly validated limit order using cached trading rules. Companion `cancel_order` cancels it and withdraws to stay net-neutral |

### Cancelling Orders

| Example | Description |
|---------|-------------|
| [`cancel_order`](examples/cancel_order.py) | Cancel a single order by hash, cancel all orders in an orderbook, and withdraw the released collateral from the global pool |
| [`user_orders`](examples/user_orders.py) | Fetch open orders for an authenticated user |

### On-Chain Operations

| Example | Description |
|---------|-------------|
| [`global_deposit_withdrawal`](examples/global_deposit_withdrawal.py) | Init position tokens, deposit to global pool, move capital into a market, extend an existing ALT, withdraw from global, and merge back to keep the run net-neutral |
| [`read_onchain`](examples/read_onchain.py) | Read exchange state, market state, user nonce, and PDA derivations via RPC |
| [`onchain_transactions`](examples/onchain_transactions.py) | Build, sign, and submit mint/merge complete set and increment nonce on-chain |

### WebSocket Streaming

| Example | Description |
|---------|-------------|
| [`ws_book_and_trades`](examples/ws_book_and_trades.py) | Live orderbook depth with `OrderbookState` state + rolling `TradeHistory` buffer |
| [`ws_ticker_and_prices`](examples/ws_ticker_and_prices.py) | Best bid/ask ticker + price history line data with `PriceHistoryState` |
| [`ws_user_and_market`](examples/ws_user_and_market.py) | Authenticated user stream (orders, balances) + market lifecycle events |

## Error Handling

Transport, authentication, signing, and transaction operations raise `SdkError`
or one of its subclasses:

| Variant | When |
|---------|------|
| `ApiRejected` | Backend rejected the request with structured details |
| `HttpError` | REST request failures |
| `WsError` | WebSocket connection/protocol errors |
| `AuthError` | Authentication failures |
| `DeserializationError` | Required fields are missing while decoding REST or WS payloads |
| `MissingMarketContext` | Market context not provided for operation requiring `DepositSource.MARKET` |
| `SigningError` | Signing operation failures |
| `UserCancelled` | User cancelled wallet signing prompt |
| `SdkError` | Catch-all for other SDK failures |

Strict wallet-balance REST and nested WebSocket decoders raise `TypeError` for
malformed payloads. Exact state arithmetic can raise `ScalingError` or `ValueError`.
`WsClient` logs and drops malformed inbound frames instead of emitting them as a
normal message or `WsEventType.ERROR`.

### API Rejections

When the backend rejects a request, the SDK raises `ApiRejected(details)` where `details` is an `ApiRejectedDetails` instance containing:

| Field | Type | Description |
|-------|------|-------------|
| `reason` | `str` | Human-readable rejection message |
| `rejection_code` | `RejectionCode \| None` | Machine-readable rejection code |
| `error_code` | `str \| None` | API-level error code such as `"NOT_FOUND"` |
| `error_log_id` | `str \| None` | Backend support correlation ID (`LCERR_*`) |
| `request_id` | `str \| None` | SDK-generated `x-request-id` header for tracing |
| `existing_method` | `str \| None` | Primary method of the conflicting Account when identity ownership has one deterministic owner |

Known rejection codes include `INSUFFICIENT_BALANCE`, `EXPIRED`, `NONCE_MISMATCH`, `SELF_TRADE`, `MARKET_INACTIVE`, `BELOW_MIN_ORDER_SIZE`, `INVALID_NONCE`, `BROADCAST_FAILURE`, `ORDER_NOT_FOUND`, `NOT_ORDER_MAKER`, `ORDER_ALREADY_FILLED`, `ORDER_ALREADY_CANCELLED`, `DUPLICATE_ORDER`, `POST_ONLY_WOULD_CROSS`, `FOK_NO_FILL`, `IOC_NO_FILL`, `WOULD_CROSS_UNAVAILABLE_LIQUIDITY`, `WOULD_CROSS_BOOK`, `MARKET_NOT_FOUND`, `ORDERBOOK_NOT_FOUND`, `TOKEN_PAIR_MISMATCH`, `INSUFFICIENT_MARKET_FEE_BUFFER`, and `SIGNATURE_EXPIRED`. Unknown codes are preserved verbatim for forward compatibility.

```python
from lightcone_sdk import ApiRejected

try:
    await client.orders().submit(request)
except ApiRejected as err:
    print(err.details.reason)
    if err.details.rejection_code is not None:
        print(err.details.rejection_code.label())
    if err.details.request_id is not None:
        print(err.details.request_id)
```

`HttpErrorKind` variants:

| Variant | Meaning |
|---------|---------|
| `REQUEST` | Network/transport failure |
| `SERVER_ERROR` | Non-2xx 5xx response from the backend |
| `RATE_LIMITED` | 429 - back off and retry |
| `UNAUTHORIZED` | 401 - session expired or missing |
| `BAD_REQUEST` | Other 4xx response from the backend |
| `NOT_FOUND` | 404 - resource does not exist |
| `TIMEOUT` | Request timed out |
| `MAX_RETRIES_EXCEEDED` | All retry attempts exhausted |

## Retry Strategy

- **Replay-safe requests**: GET and DELETE helpers default to `RetryPolicy.IDEMPOTENT`, and idempotent set operations such as favorite-market POSTs opt into it explicitly. This policy retries transport failures and 429/502/503/504 with exponential backoff + jitter.
- **Non-idempotent requests** (order submit, cancel, auth): `RetryPolicy.NONE` - no automatic retry, which prevents duplicate side effects.
- Customizable per-call with `RetryPolicy.custom(RetryConfig(...))`. If you use `LightconeHttp` directly, pass a `RetryPolicy` per request.

### Credential restoration (401 recovery)

Sessions built on short-lived tokens expire mid-run: the backend starts answering 401 even though the app could mint a fresh token (e.g. by re-running login). Rather than every caller hand-rolling "detect 401 → refresh → retry", the transport accepts a host-supplied async hook:

```python
async def restore_credentials() -> bool:
    # e.g. re-run the login flow so the auth cookie is valid again
    return await refresh_session()

client.set_credential_restorer(restore_credentials)
```

When a request to the API origin fails with HTTP 401 and a restorer is registered, the transport consults it **at most once per logical request**, with concurrent 401s sharing one restoration (bounded by a 30-second timeout). A successful restoration replays the request once **only if it declared itself retry-safe** (an idempotent/custom retry policy); `RetryPolicy.NONE` requests — mutations like orders and cancels — are never auto-replayed: the restoration still heals the session for the caller's next attempt, but the original 401 propagates. Restoration is skipped for credential-management endpoints (login, logout) and for cookie-override/custom-session requests, redirects are never followed on the API transport, and without a registered restorer 401s propagate unchanged. A timed-out restoration is cancelled outright (asyncio task cancellation), so it can never keep running alongside the next one. The transport also disables aiohttp's ambient cookie jar (`DummyCookieJar`): cookies are managed explicitly, so a response's `Set-Cookie` can never silently ride a later request.

The SDK stays credential-agnostic: what "restore" means belongs to the host. For classifying auth failures in your own code, use `lightcone_sdk.error.is_unauthorized(error)` — it covers both bare 401s and 401s carrying a structured rejection envelope (`ApiRejectedDetails.http_status`).

## Trigger Orders

Trigger orders (stop-limit, take-profit-limit) are under development and not yet available. Internal types exist in the source for internal use only.
