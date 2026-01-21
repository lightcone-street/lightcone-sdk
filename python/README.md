# Lightcone SDK for Python

Python SDK for the Lightcone protocol on Solana.

## Installation

```bash
pip install lightcone-sdk
```

Or install from source:

```bash
pip install -e .
```

## Quick Start

```python
import asyncio
from solana.rpc.async_api import AsyncClient
from solders.keypair import Keypair
from lightcone_sdk import (
    LightconePinocchioClient,
    BidOrderParams,
    OrderSide,
)

async def main():
    # Connect to Solana
    connection = AsyncClient("https://api.mainnet-beta.solana.com")
    client = LightconePinocchioClient(connection)

    # Fetch exchange info
    exchange = await client.get_exchange()
    print(f"Exchange has {exchange.market_count} markets")

    # Fetch a market
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
            expiration=int(time.time()) + 3600,  # 1 hour
        ),
        keypair,
    )

    await connection.close()

asyncio.run(main())
```

## Features

- **Account fetching**: Deserialize Exchange, Market, Position, OrderStatus, and UserNonce accounts
- **Order management**: Create, sign, hash, and serialize orders
- **Transaction building**: Build all 14 Lightcone instructions
- **PDA derivation**: Derive all program-derived addresses
- **Ed25519 verification**: Build Ed25519 signature verification instructions

## API Reference

### Client

```python
from lightcone_sdk import LightconePinocchioClient

client = LightconePinocchioClient(connection)

# Account fetchers
exchange = await client.get_exchange()
market = await client.get_market(market_id)
position = await client.get_position(owner, market)
order_status = await client.get_order_status(order_hash)
nonce = await client.get_user_nonce(user)

# Transaction builders
tx = await client.initialize(authority)
tx = await client.create_market(authority, num_outcomes, oracle, question_id)
tx = await client.mint_complete_set(params, num_outcomes)
tx = await client.match_orders_multi_with_verify(operator, market, base_mint, quote_mint, taker, makers)

# Order helpers
order = client.create_signed_bid_order(params, keypair)
order_hash = client.hash_order(order)
```

### PDA Derivation

```python
from lightcone_sdk import (
    get_exchange_pda,
    get_market_pda,
    get_position_pda,
    get_order_status_pda,
    get_conditional_mint_pda,
)

exchange, bump = get_exchange_pda()
market, bump = get_market_pda(market_id)
position, bump = get_position_pda(owner, market)
```

### Order Operations

```python
from lightcone_sdk import (
    create_bid_order,
    create_ask_order,
    hash_order,
    sign_order,
    verify_order_signature,
)

# Create and sign an order
order = create_bid_order(params)
sign_order(order, keypair)

# Verify signature
is_valid = verify_order_signature(order)

# Get order hash
order_hash = hash_order(order)
```

## Types

### Enums

- `MarketStatus`: PENDING, ACTIVE, RESOLVED, CANCELLED
- `OrderSide`: BID, ASK

### Data Classes

- `Exchange`: Exchange account data
- `Market`: Market account data
- `Position`: Position account data
- `OrderStatus`: Order status account data
- `UserNonce`: User nonce account data
- `FullOrder`: Complete order with signature (225 bytes)
- `CompactOrder`: Order without market/mints (65 bytes)

## Development

```bash
# Install dev dependencies (using uv)
uv venv && uv pip install -e ".[dev]"

# Or with pip
pip install -e ".[dev]"
```

### Running Tests

```bash
# Run unit tests only (default, runs offline)
pytest -v

# Run devnet integration tests (requires funded wallet)
DEVNET_TESTS=1 pytest tests/test_devnet.py -v -s

# Run all tests including devnet
DEVNET_TESTS=1 pytest -v -s
```

### Devnet Test Configuration

Set up your authority keypair for devnet tests:

```bash
# Option 1: Create keypairs/devnet-authority.json
mkdir -p keypairs
solana-keygen new -o keypairs/devnet-authority.json

# Option 2: Use environment variable
export AUTHORITY_KEYPAIR=/path/to/your/keypair.json

# Option 3: Use default Solana CLI keypair
# (~/.config/solana/id.json is used automatically)
```

Fund your wallet on devnet:
```bash
solana airdrop 2 --url devnet
```

### Code Quality

```bash
# Format code
black lightcone_sdk tests
ruff check lightcone_sdk tests

# Type check
mypy lightcone_sdk
```

## License

MIT
