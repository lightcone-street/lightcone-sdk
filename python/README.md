# Lightcone Python SDK

Python SDK for the Lightcone protocol on Solana.

## Installation

```bash
pip install lightcone-sdk
```

## Modules

| Module | Description |
|--------|-------------|
| [`program`](src/program/README.md) | On-chain Solana program interaction (accounts, transactions, orders, Ed25519 verification) |
| [`api`](src/api/README.md) | REST API client for market data and order management |
| [`websocket`](src/websocket/README.md) | Real-time data streaming via WebSocket |
| [`shared`](src/shared/README.md) | Shared utilities (Resolution, constants, decimal helpers) |

## Quick Start

```python
from lightcone_sdk import *
```

### REST API

Query markets, submit orders, and manage positions via the REST API.

```python
import asyncio
from lightcone_sdk.api import LightconeApiClient

async def main():
    async with LightconeApiClient("https://api.lightcone.xyz") as client:
        # Get markets
        markets = await client.get_markets()
        print(f"Found {markets.total} markets")

        # Get orderbook depth
        orderbook = await client.get_orderbook("orderbook_id", depth=10)
        print(f"Best bid: {orderbook.best_bid}")
        print(f"Best ask: {orderbook.best_ask}")

        # Get user positions
        positions = await client.get_user_positions("user_pubkey")

asyncio.run(main())
```

### On-Chain Program

Build transactions for on-chain operations: minting positions, matching orders, redeeming winnings.

```python
import asyncio
from solana.rpc.async_api import AsyncClient
from solders.keypair import Keypair
from lightcone_sdk.program import LightconePinocchioClient
from lightcone_sdk import BidOrderParams, MintCompleteSetParams

async def main():
    connection = AsyncClient("https://api.devnet.solana.com")
    client = LightconePinocchioClient(connection)

    # Fetch account state
    exchange = await client.get_exchange()
    market = await client.get_market(0)
    print(f"Market status: {market.status.name}")

    # Create a signed order
    keypair = Keypair()
    nonce = await client.get_next_nonce(keypair.pubkey())

    order = client.create_signed_bid_order(
        BidOrderParams(
            nonce=nonce,
            maker=keypair.pubkey(),
            market=client.get_market_address(0),
            base_mint=...,  # conditional token mint
            quote_mint=...,  # USDC mint
            maker_amount=1_000_000,  # 1 USDC
            taker_amount=500_000,    # 0.5 outcome tokens
            expiration=0,
        ),
        keypair,
    )

    await connection.close()

asyncio.run(main())
```

### WebSocket

Stream real-time orderbook updates, trades, and user events.

```python
import asyncio
from lightcone_sdk.websocket import LightconeWebSocketClient, WsEventType

async def main():
    client = await LightconeWebSocketClient.connect("wss://ws.lightcone.xyz/ws")

    # Subscribe to orderbook updates
    await client.subscribe_book_updates(["market:orderbook"])

    # Process events
    async for event in client:
        if event.type == WsEventType.BOOK_UPDATE:
            book = client.get_orderbook(event.orderbook_id)
            if book:
                print(f"Best bid: {book.best_bid()}")
                print(f"Best ask: {book.best_ask()}")
        elif event.type == WsEventType.TRADE:
            print(f"Trade: {event.trade.size} @ {event.trade.price}")
        elif event.type == WsEventType.DISCONNECTED:
            break

    await client.disconnect()

asyncio.run(main())
```

## Module Overview

### Program Module

Direct interaction with the Lightcone Solana program:

- **Account Types**: Exchange, Market, Position, OrderStatus, UserNonce
- **Transaction Builders**: All 14 instructions (mint, merge, match, settle, etc.)
- **PDA Derivation**: 8 PDA functions with seeds
- **Order Types**: FullOrder (225 bytes), CompactOrder (65 bytes)
- **Ed25519 Verification**: Three strategies (individual, batch, cross-reference)

```python
from lightcone_sdk.program import *
from lightcone_sdk import BidOrderParams

# Create and sign an order
order = create_signed_bid_order(
    BidOrderParams(
        nonce=1,
        maker=pubkey,
        market=market_pda,
        base_mint=yes_token,
        quote_mint=no_token,
        maker_amount=100_000,
        taker_amount=50_000,
        expiration=0,
    ),
    keypair,
)
order_hash = hash_order(order)
```

### API Module

REST API client with typed requests and responses:

- **Markets**: get_markets, get_market, get_market_by_slug
- **Orderbooks**: get_orderbook with depth parameter
- **Orders**: submit_order, cancel_order, cancel_all_orders
- **Positions**: get_user_positions, get_user_market_positions
- **Trades**: get_trades with filters
- **Price History**: get_price_history with OHLCV

```python
from lightcone_sdk.api import LightconeApiClient, SubmitOrderRequest

async with LightconeApiClient("https://api.lightcone.xyz") as client:
    response = await client.submit_order(SubmitOrderRequest(
        maker="pubkey",
        nonce=1,
        market_pubkey="market",
        base_token="base",
        quote_token="quote",
        side=0,  # BID
        maker_amount=1000000,
        taker_amount=500000,
        expiration=0,
        signature="hex_signature",
        orderbook_id="orderbook",
    ))
```

### WebSocket Module

Real-time streaming with automatic state management:

- **Subscriptions**: book_updates, trades, user, price_history, market
- **State Management**: LocalOrderbook, UserState, PriceHistory
- **Authentication**: Ed25519 sign-in for user streams
- **Auto-Reconnect**: Configurable reconnection with exponential backoff and jitter

```python
from lightcone_sdk.websocket import LightconeWebSocketClient, authenticate

# Authenticated connection for user streams
from nacl.signing import SigningKey
credentials = await authenticate(signing_key)
client = await LightconeWebSocketClient.connect(
    "wss://ws.lightcone.xyz/ws",
    auth_token=credentials.auth_token,
)
await client.subscribe_user("user_pubkey")

# Access maintained state
state = client.get_user_state("user_pubkey")
if state:
    print(f"Open orders: {state.order_count()}")
```

### Shared Module

Common utilities used across modules:

- **Resolution**: Candle intervals (1m, 5m, 15m, 1h, 4h, 1d)
- **Constants**: Program IDs, seeds, discriminators, sizes
- **Types**: MarketStatus, OrderSide, account data classes

```python
from lightcone_sdk.shared import Resolution, PROGRAM_ID

res = Resolution.ONE_HOUR  # "1h"
print(res.as_str())  # "1h"
```

## Program ID

**Mainnet/Devnet**: `Aumw7EC9nnxDjQFzr1fhvXvnG3Rn3Bb5E3kbcbLrBdEk`

## License

MIT
