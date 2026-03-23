"""Limit order via client.orders().limit_order() with human-readable price/size, auto-scaling, and fill tracking."""

import asyncio

from common import (
    client as make_client,
    wallet,
    login,
    market_and_orderbook,
)


async def main():
    client = make_client()
    keypair = wallet()
    await login(client, keypair)

    # 1. Fetch market and orderbook
    _market, orderbook = await market_and_orderbook(client)

    # 2. Build, sign a limit order (market, mints, salt are auto-filled from orderbook)
    request = (
        client.orders().limit_order()
        .maker(keypair.pubkey())
        .bid()
        .price("0.55")
        .size("1")
        .sign(keypair, orderbook)
    )

    # 3. Submit
    response = await client.orders().submit(request)
    print(
        f"submitted: {response.order_hash} "
        f"filled={response.filled} remaining={response.remaining} "
        f"fills={len(response.fills)}"
    )

    await client.close()


asyncio.run(main())
