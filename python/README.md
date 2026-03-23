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
pip install lightcone-sdk
```


## Quick Start

```python
import asyncio

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk import LightconeClientBuilder
from lightcone_sdk.auth.client import sign_login_message
from lightcone_sdk.ws.subscriptions import BookUpdateParams


async def main():
    client = (
        LightconeClientBuilder()
        .rpc_url("https://api.devnet.solana.com")
        .build()
    )
    keypair = Keypair()

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
    request = (
        client.orders().limit_order()
        .maker(keypair.pubkey())
        .market(Pubkey.from_string(market.pubkey))
        .base_mint(Pubkey.from_string(orderbook.base.mint))
        .quote_mint(Pubkey.from_string(orderbook.quote.mint))
        .bid()
        .price("0.55")
        .size("100")
        .nonce(await client.orders().current_nonce(keypair.pubkey()))
        .sign(keypair, orderbook)
    )

    response = await client.orders().submit(request)
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
    .rpc_url("https://api.devnet.solana.com")
    .native_signer(keypair)
    .build()
)
```

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
request = (
    client.orders().limit_order()
    .maker(keypair.pubkey())
    .market(Pubkey.from_string(market.pubkey))
    .base_mint(Pubkey.from_string(orderbook.base.mint))
    .quote_mint(Pubkey.from_string(orderbook.quote.mint))
    .bid()
    .price("0.55")
    .size("1")
    .nonce(await client.orders().current_nonce(keypair.pubkey()))
    .sign(keypair, orderbook)
)
order = await client.orders().submit(request)
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
tx_hash = await (client.markets().merge_complete_set()
    .user(keypair.pubkey())
    .market(Pubkey.from_string(market.pubkey))
    .mint(deposit_mint)
    .amount(1_000_000)
    .num_outcomes(num_outcomes)
    .sign_and_submit())
```

## Authentication
Authentication is only required for user-specific endpoints. Authentication is session-based using ED25519 signed messages. The flow is: request a nonce, sign it with your wallet, and exchange it for a session token.

## Examples
All examples are runnable with `python examples/<name>.py`. Set environment variables in a `.env` file - see [`.env.example`](.env.example) for the template.

### Setup & Authentication

| Example | Description |
|---------|-------------|
| [`login`](examples/login.py) | Full auth lifecycle: sign message, login, check session, logout |

### Market Discovery & Data

| Example | Description |
|---------|-------------|
| [`markets`](examples/markets.py) | Featured markets, paginated listing, fetch by pubkey, search |
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
| [`ws_book_and_trades`](examples/ws_book_and_trades.py) | Live orderbook depth with `OrderbookSnapshot` state + rolling `TradeHistory` buffer |
| [`ws_ticker_and_prices`](examples/ws_ticker_and_prices.py) | Best bid/ask ticker + price history line data with `PriceHistoryState` |
| [`ws_user_and_market`](examples/ws_user_and_market.py) | Authenticated user stream (orders, balances) + market lifecycle events |

## Error Handling

All SDK operations raise `SdkError` or one of its subclasses:

| Variant | When |
|---------|------|
| `HttpError` | REST request failures |
| `WsError` | WebSocket connection/protocol errors |
| `AuthError` | Authentication failures |
| `DeserializationError` | Required fields are missing while decoding REST or WS payloads |
| `MissingMarketContext` | Market context not provided for operation requiring `DepositSource.MARKET` |
| `SigningError` | Signing operation failures |
| `UserCancelled` | User cancelled wallet signing prompt |
| `SdkError` | Catch-all, including rejected order responses |

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
