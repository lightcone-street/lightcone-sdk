"""Limit order via client.orders().limit_order() with human-readable price/size, auto-scaling, and fill tracking."""

import asyncio

from common import (
    client as make_client,
    wallet,
    login,
    market_and_orderbook,
)
from lightcone_sdk.program.orders import generate_salt
from lightcone_sdk.shared.signing import SigningStrategy


async def main():
    keypair = wallet()
    client = make_client()
    client.set_signing_strategy(SigningStrategy.native(keypair))
    await login(client, keypair)

    # 1. Fetch market and orderbook
    _market, orderbook = await market_and_orderbook(client)

    # 2. Fetch and cache the on-chain nonce once. Subsequent orders that omit
    #    .nonce() will automatically use this cached value.
    nonce = await client.orders().current_nonce(keypair.pubkey())
    client.set_order_nonce(nonce)

    # 3. submit() auto-populates nonce from cache when .nonce() is not called.
    response = await (
        client.orders().limit_order()
        .maker(keypair.pubkey())
        .bid()
        .price("0.55")
        .size("1")
        .salt(generate_salt())
        .submit(client, orderbook)
    )
    print(
        f"submitted: {response.order_hash} "
        f"filled={response.filled} remaining={response.remaining} "
        f"fills={len(response.fills)}"
    )

    await client.close()


asyncio.run(main())
