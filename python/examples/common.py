"""Shared helpers for the Lightcone Python SDK examples."""

import json
import os
import sys
import time
from pathlib import Path

from dotenv import load_dotenv
from solana.rpc.async_api import AsyncClient
from solders.keypair import Keypair
from solders.pubkey import Pubkey

# Allow imports from the SDK source
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))

from lightcone_sdk.client import LightconeClientBuilder, LightconeClient
from lightcone_sdk.auth.client import sign_login_message
from lightcone_sdk.auth import User
from lightcone_sdk.domain.market import Market, OrderBookPair
from lightcone_sdk.program.client import LightconePinocchioClient
from lightcone_sdk.shared.scaling import OrderbookDecimals

load_dotenv(Path(__file__).resolve().parent.parent / ".env")


def rest_client() -> LightconeClient:
    return LightconeClientBuilder().build()


def rpc_client() -> LightconePinocchioClient:
    url = os.environ.get("SOLANA_RPC_URL", "https://api.devnet.solana.com")
    return LightconePinocchioClient(AsyncClient(url))


def wallet() -> Keypair:
    raw = os.environ.get("LIGHTCONE_WALLET_PATH")
    if not raw:
        raise RuntimeError("set LIGHTCONE_WALLET_PATH in .env or the environment")
    path = Path(raw).expanduser()
    with path.open() as f:
        secret = json.load(f)
    return Keypair.from_bytes(bytes(secret))


async def login(
    client: LightconeClient,
    keypair: Keypair,
    use_embedded_wallet: bool = False,
) -> User:
    nonce = await client.auth().get_nonce()
    message, signature_bs58, pubkey_bytes = sign_login_message(keypair, nonce)
    return await client.auth().login_with_message(
        message,
        signature_bs58,
        pubkey_bytes,
        True if use_embedded_wallet else None,
    )


async def market(client: LightconeClient) -> Market:
    result = await client.markets().get(None, 1)
    markets = result.markets
    if not markets:
        raise RuntimeError("no markets returned by the API")
    return markets[0]


async def market_and_orderbook(
    client: LightconeClient,
) -> tuple[Market, OrderBookPair]:
    m = await market(client)
    ob = next((p for p in m.orderbook_pairs if p.active), None) or (
        m.orderbook_pairs[0] if m.orderbook_pairs else None
    )
    if ob is None:
        raise RuntimeError("selected market has no orderbooks")
    return m, ob


async def scaling_decimals(
    client: LightconeClient, orderbook: OrderBookPair
) -> OrderbookDecimals:
    decimals = await client.orderbooks().decimals(orderbook.orderbook_id)
    return OrderbookDecimals(
        orderbook_id=orderbook.orderbook_id,
        base_decimals=decimals.base_decimals,
        quote_decimals=decimals.quote_decimals,
        price_decimals=decimals.price_decimals,
        tick_size=max(orderbook.tick_size, 0),
    )


def deposit_mint(m: Market) -> Pubkey:
    if not m.deposit_assets:
        raise RuntimeError("selected market has no deposit assets")
    return Pubkey.from_string(m.deposit_assets[0].deposit_asset)


def num_outcomes(m: Market) -> int:
    return len(m.outcomes)


async def fresh_order_nonce(
    rpc: LightconePinocchioClient, user: Pubkey
) -> int:
    return await rpc.get_current_nonce(user)


def unix_timestamp() -> int:
    return int(time.time())


def unix_timestamp_ms() -> int:
    return int(time.time() * 1000)
