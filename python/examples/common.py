"""Shared helpers for the Lightcone Python SDK examples."""

import json
import os
import sys
import time
from pathlib import Path

from solders.keypair import Keypair
from solders.pubkey import Pubkey

# Allow imports from the SDK source
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))

from lightcone_sdk.client import LightconeClientBuilder, LightconeClient
from lightcone_sdk.auth.client import sign_login_message
from lightcone_sdk.auth import User
from lightcone_sdk.domain.market import Market, OrderBookPair

DEFAULT_WALLET_PATH = "~/.config/solana/id.json"


def client() -> LightconeClient:
    """Build a LightconeClient.

    Defaults to production. Set ``LIGHTCONE_ENV`` to override
    (options: local, staging, prod).
    """
    from lightcone_sdk.env import LightconeEnv

    builder = LightconeClientBuilder()
    env_str = os.environ.get("LIGHTCONE_ENV")
    if env_str:
        try:
            builder = builder.env(LightconeEnv(env_str.lower()))
        except ValueError as exc:
            raise RuntimeError(
                f"invalid LIGHTCONE_ENV '{env_str}'. Options: local, staging, prod"
            ) from exc
    return builder.build()


# Backward-compat alias
rest_client = client


def get_keypair() -> Keypair:
    """Load a keypair from disk.

    Defaults to ``~/.config/solana/id.json``. Set ``LIGHTCONE_WALLET_PATH``
    to override.
    """
    raw = os.environ.get("LIGHTCONE_WALLET_PATH", DEFAULT_WALLET_PATH)
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


async def wait_for_global_balance(
    client: LightconeClient,
    mint: Pubkey,
    minimum_amount: int,
    timeout_seconds: float = 30.0,
    interval_seconds: float = 2.0,
) -> None:
    import asyncio

    mint_str = str(mint)
    deadline = time.monotonic() + timeout_seconds
    attempt = 0
    print(f"waiting for global balance: mint={mint_str} required={minimum_amount}")
    while time.monotonic() < deadline:
        attempt += 1
        balances = await client.positions().deposit_token_balances()
        entry = next(
            (balance for balance in balances.values() if balance.mint == mint_str),
            None,
        )
        current_idle = int(entry.idle) if entry is not None else 0
        symbol = entry.symbol if entry is not None else "unknown"
        if current_idle >= minimum_amount:
            print(f"global balance ready: {symbol} idle={current_idle} (attempt {attempt})")
            return
        remaining = deadline - time.monotonic()
        print(
            f"global balance not ready: {symbol} idle={current_idle}/{minimum_amount} "
            f"(attempt {attempt}, {remaining:.0f}s remaining)"
        )
        await asyncio.sleep(interval_seconds)
    raise RuntimeError(
        f"global balance for {mint_str} did not reach {minimum_amount} within {timeout_seconds}s"
    )


def unix_timestamp() -> int:
    return int(time.time())


def unix_timestamp_ms() -> int:
    return int(time.time() * 1000)
