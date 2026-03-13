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
from solana.rpc.async_api import AsyncClient
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk import (
    LightconeClientBuilder,
    LimitOrderEnvelope,
    Resolution,
    sign_login_message,
    LightconePinocchioClient,
)

async def main():
    client = LightconeClientBuilder().build()
    rpc = LightconePinocchioClient(AsyncClient("https://api.devnet.solana.com"))
    keypair = Keypair()

    # 1. Authenticate
    nonce = await client.auth().get_nonce()
    message, signature_bs58, pubkey_bytes = sign_login_message(keypair, nonce)
    user = await client.auth().login_with_message(
        message, signature_bs58, pubkey_bytes
    )

    # 2. Find a market
    markets, _ = await client.markets().get(None, 1)
    market = markets[0]
    orderbook = market.orderbook_pairs[0]

    # 3. Get orderbook decimals for price scaling
    decimals_resp = await client.orderbooks().decimals(orderbook.orderbook_id)
    from lightcone_sdk import OrderbookDecimals
    decimals = OrderbookDecimals(
        base_decimals=decimals_resp.base_decimals,
        quote_decimals=decimals_resp.quote_decimals,
    )

    # 4. Build, sign, and submit a limit order
    nonce_val = await rpc.get_current_nonce(keypair.pubkey())
    request = (
        LimitOrderEnvelope()
        .maker(keypair.pubkey())
        .market(Pubkey.from_string(market.pubkey))
        .base_mint(Pubkey.from_string(orderbook.base_token))
        .quote_mint(Pubkey.from_string(orderbook.quote_token))
        .bid()
        .price("0.55")
        .size("100")
        .nonce(nonce_val)
        .apply_scaling(decimals=decimals)
        .sign(keypair, orderbook.orderbook_id)
    )

    response = await client.orders().submit(request)
    print("Order submitted:", response)

    # 5. Stream real-time updates
    from lightcone_sdk.ws.subscriptions import BookUpdateParams
    ws = client.ws()
    await ws.connect()
    await ws.subscribe(BookUpdateParams(orderbook_ids=[orderbook.orderbook_id]))

    await client.close()

asyncio.run(main())
```

## Start Trading

```python
import json
import asyncio
from solana.rpc.async_api import AsyncClient
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk import LightconeClientBuilder, LightconePinocchioClient

client = LightconeClientBuilder().build()
rpc = LightconePinocchioClient(AsyncClient("https://api.devnet.solana.com"))
with open("~/.config/solana/id.json") as f:
    secret = json.load(f)
keypair = Keypair.from_bytes(bytes(secret))
```

### Step 1: Find a Market

```python
markets, _ = await client.markets().get(None, 1)
market = markets[0]
orderbook = next(
    (p for p in market.orderbook_pairs if p.active),
    market.orderbook_pairs[0] if market.orderbook_pairs else None,
)
```

### Step 2: Deposit Collateral

```python
from lightcone_sdk import MintCompleteSetParams

market_pubkey = Pubkey.from_string(market.pubkey)
deposit_mint = Pubkey.from_string(market.deposit_assets[0].deposit_asset)
num_outcomes = len(market.outcomes)
tx = await rpc.mint_complete_set(
    MintCompleteSetParams(
        user=keypair.pubkey(),
        market=market_pubkey,
        deposit_mint=deposit_mint,
        amount=1_000_000,
    ),
    num_outcomes,
)
blockhash = await rpc.get_latest_blockhash()
tx.sign([keypair], blockhash)
```

### Step 3: Place an Order

```python
from lightcone_sdk import LimitOrderEnvelope, OrderbookDecimals

decimals_resp = await client.orderbooks().decimals(orderbook.orderbook_id)
decimals = OrderbookDecimals(
    base_decimals=decimals_resp.base_decimals,
    quote_decimals=decimals_resp.quote_decimals,
)
request = (
    LimitOrderEnvelope()
    .maker(keypair.pubkey())
    .market(Pubkey.from_string(market.pubkey))
    .base_mint(Pubkey.from_string(orderbook.base_token))
    .quote_mint(Pubkey.from_string(orderbook.quote_token))
    .bid()
    .price("0.55")
    .size("1")
    .nonce(await rpc.get_current_nonce(keypair.pubkey()))
    .apply_scaling(decimals=decimals)
    .sign(keypair, orderbook.orderbook_id)
)
order = await client.orders().submit(request)
```

### Step 4: Monitor

```python
from lightcone_sdk.ws.subscriptions import BookUpdateParams, UserParams

open_orders = await client.orders().get_user_orders(
    str(keypair.pubkey()), 50
)
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
from lightcone_sdk import MergeCompleteSetParams

tx = await rpc.merge_complete_set(
    MergeCompleteSetParams(
        user=keypair.pubkey(),
        market=Pubkey.from_string(market.pubkey),
        deposit_mint=deposit_mint,
        amount=1_000_000,
    ),
    num_outcomes,
)
blockhash = await rpc.get_latest_blockhash()
tx.sign([keypair], blockhash)
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
| [`orderbook`](examples/orderbook.py) | Fetch orderbook depth (bids/asks) and decimal precision metadata |
| [`trades`](examples/trades.py) | Recent trade history with cursor-based pagination |
| [`price_history`](examples/price_history.py) | Historical candlestick data (OHLCV) at various resolutions |
| [`positions`](examples/positions.py) | User positions across all markets and per-market |

### Placing Orders

| Example | Description |
|---------|-------------|
| [`submit_order`](examples/submit_order.py) | `LimitOrderEnvelope` with human-readable price/size, auto-scaling, and fill tracking |

### Cancelling Orders

| Example | Description |
|---------|-------------|
| [`cancel_order`](examples/cancel_order.py) | Cancel a single order by hash and cancel all orders in an orderbook |
| [`user_orders`](examples/user_orders.py) | Fetch open orders for an authenticated user |

### On-Chain Operations

| Example | Description |
|---------|-------------|
| [`read_onchain`](examples/read_onchain.py) | Read exchange state, market state, user nonce, and PDA derivations via RPC |
| [`onchain_transactions`](examples/onchain_transactions.py) | Build, sign, and submit mint/merge complete set and increment nonce on-chain |

### WebSocket Streaming

| Example | Description |
|---------|-------------|
| [`ws_book_and_trades`](examples/ws_book_and_trades.py) | Live orderbook depth with `OrderbookSnapshot` state + rolling `TradeHistory` buffer |
| [`ws_ticker_and_prices`](examples/ws_ticker_and_prices.py) | Best bid/ask ticker + price history candles with `PriceHistoryState` |
| [`ws_user_and_market`](examples/ws_user_and_market.py) | Authenticated user stream (orders, balances) + market lifecycle events |

## Error Handling

All SDK operations raise `SdkError` on failure:

| Variant | When |
|---------|------|
| `HttpError` | REST request failures |
| `WsError` | WebSocket connection/protocol errors |
| `AuthError` | Authentication failures |
| `SdkError` | Catch-all |

Notable `HttpError` variants (`HttpErrorKind`):

| Variant | Meaning |
|---------|---------|
| `SERVER_ERROR` | Non-2xx response from the backend |
| `RATE_LIMITED` | 429 - back off and retry |
| `UNAUTHORIZED` | 401 - session expired or missing |
| `MAX_RETRIES_EXCEEDED` | All retry attempts exhausted |

## Retry Strategy

- **GET requests**: `RetryPolicy.IDEMPOTENT` - retries on transport failures and 502/503/504, backs off on 429 with exponential backoff + jitter.
- **POST requests** (order submit, cancel, auth): `RetryPolicy.NONE` - no automatic retry. Non-idempotent actions are never retried to prevent duplicate side effects.
- Customizable per-call with `RetryPolicy.custom(RetryConfig(...))`.
