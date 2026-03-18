"""Shared helpers for the Lightcone Python SDK examples."""

import json
import os
import sys
import time
from pathlib import Path

from dotenv import load_dotenv
from solders.keypair import Keypair
from solders.pubkey import Pubkey

# Allow imports from the SDK source
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))

from lightcone_sdk.client import LightconeClientBuilder, LightconeClient
from lightcone_sdk.auth.client import sign_login_message
from lightcone_sdk.auth import User
from lightcone_sdk.domain.market import Market, OrderBookPair

load_dotenv(Path(__file__).resolve().parent.parent / ".env")


def client() -> LightconeClient:
    """Build a LightconeClient with optional Solana RPC support."""
    builder = LightconeClientBuilder()
    rpc_url = os.environ.get("SOLANA_RPC_URL")
    if rpc_url:
        builder = builder.rpc_url(rpc_url)
    return builder.build()


# Backward-compat alias
rest_client = client


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


def deposit_mint(m: Market) -> Pubkey:
    if not m.deposit_assets:
        raise RuntimeError("selected market has no deposit assets")
    return Pubkey.from_string(m.deposit_assets[0].deposit_asset)


def num_outcomes(m: Market) -> int:
    return len(m.outcomes)


def unix_timestamp() -> int:
    return int(time.time())


def unix_timestamp_ms() -> int:
    return int(time.time() * 1000)
