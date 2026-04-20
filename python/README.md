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

    # 3. Build, sign, and submit a limit order (scaling is automatic)
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

## Authentication
Authentication is only required for user-specific endpoints. Authentication is session-based using ED25519 signed messages. The flow is: request a nonce, sign it with your wallet, and exchange it for a session token.

## Examples
All examples are runnable with `python examples/<name>.py`. Examples default to the production environment and read the wallet keypair from `~/.config/solana/id.json`. Set `LIGHTCONE_ENV=local|staging|prod` or `LIGHTCONE_WALLET_PATH=/path/to/keypair.json` to override.

### Setup & Authentication

| Example | Description |
|---------|-------------|
| [`login`](examples/login.py) | Full auth lifecycle: sign message, login, check session, logout |

### Market Discovery & Data

| Example | Description |
|---------|-------------|
| [`markets`](examples/markets.py) | Featured markets, paginated listing, fetch by pubkey, search, platform deposit assets via `global_deposit_assets()` |
| [`orderbook`](examples/orderbook.py) | Fetch orderbook depth (bids/asks) and derive decimal precision metadata |
| [`trades`](examples/trades.py) | Recent trade history with cursor-based pagination |
| [`price_history`](examples/price_history.py) | Historical price history line data at various resolutions |
| [`positions`](examples/positions.py) | User positions across all markets and per-market |

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.py) | Limit order via `client.orders().limit_order()` with human-readable price/size, auto-scaling, and fill tracking |

### Cancelling Orders

| Example | Description |
|---------|-------------|
| [`cancel_order`](examples/cancel_order.py) | Cancel a single order by hash and cancel all orders in an orderbook |
| [`user_orders`](examples/user_orders.py) | Fetch open orders for an authenticated user |

### On-Chain Operations

| Example | Description |
|---------|-------------|
| [`global_deposit_withdrawal`](examples/global_deposit_withdrawal.py) | Init position tokens, deposit to global pool, move capital into a market, extend an existing ALT, and withdraw from global |
| [`read_onchain`](examples/read_onchain.py) | Read exchange state, market state, user nonce, and PDA derivations via RPC |
| [`onchain_transactions`](examples/onchain_transactions.py) | Build, sign, and submit mint/merge complete set and increment nonce on-chain |

### WebSocket Streaming

| Example | Description |
|---------|-------------|
| [`ws_book_and_trades`](examples/ws_book_and_trades.py) | Live orderbook depth with `OrderbookState` state + rolling `TradeHistory` buffer |
| [`ws_ticker_and_prices`](examples/ws_ticker_and_prices.py) | Best bid/ask ticker + price history line data with `PriceHistoryState` |
| [`ws_user_and_market`](examples/ws_user_and_market.py) | Authenticated user stream (orders, balances) + market lifecycle events |

## Error Handling

All SDK operations raise `SdkError` or one of its subclasses:

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

### API Rejections

When the backend rejects a request, the SDK raises `ApiRejected(details)` where `details` is an `ApiRejectedDetails` instance containing:

| Field | Type | Description |
|-------|------|-------------|
| `reason` | `str` | Human-readable rejection message |
| `rejection_code` | `RejectionCode \| None` | Machine-readable rejection code |
| `error_code` | `str \| None` | API-level error code such as `"NOT_FOUND"` |
| `error_log_id` | `str \| None` | Backend support correlation ID (`LCERR_*`) |
| `request_id` | `str \| None` | SDK-generated `x-request-id` header for tracing |

Known rejection codes include `INSUFFICIENT_BALANCE`, `EXPIRED`, `NONCE_MISMATCH`, `SELF_TRADE`, `MARKET_INACTIVE`, `BELOW_MIN_ORDER_SIZE`, `INVALID_NONCE`, `BROADCAST_FAILURE`, `ORDER_NOT_FOUND`, `NOT_ORDER_MAKER`, `ORDER_ALREADY_FILLED`, and `ORDER_ALREADY_CANCELLED`. Unknown codes are preserved verbatim for forward compatibility.

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

- **GET requests**: `RetryPolicy.IDEMPOTENT` - retries on transport failures and 429/502/503/504 with exponential backoff + jitter.
- **POST requests** (order submit, cancel, auth): `RetryPolicy.NONE` - no automatic retry. Non-idempotent actions are never retried to prevent duplicate side effects.
- Customizable per-call with `RetryPolicy.custom(RetryConfig(...))`. If you use `LightconeHttp` directly, pass a `RetryPolicy` per request.
