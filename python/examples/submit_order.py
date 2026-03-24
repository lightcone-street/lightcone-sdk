"""Limit order via client.orders().limit_order() with human-readable price/size, auto-scaling, and fill tracking."""

import asyncio

from solders.pubkey import Pubkey

from common import (
    client as make_client,
    wallet,
    login,
    market_and_orderbook,
)
from lightcone_sdk.program.orders import generate_salt


async def main():
    client = make_client()
    keypair = wallet()
    await login(client, keypair)

    # 1. Fetch market and orderbook
    m, orderbook = await market_and_orderbook(client)
    base_mint = Pubkey.from_string(orderbook.base.pubkey)
    quote_mint = Pubkey.from_string(orderbook.quote.pubkey)

    # 2. Get a fresh nonce from on-chain
    nonce = await client.orders().current_nonce(keypair.pubkey())

    # 3. Build, sign a limit order (scaling is applied automatically)
    request = (
        client.orders().limit_order()
        .maker(keypair.pubkey())
        .market(Pubkey.from_string(m.pubkey))
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .price("0.55")
        .size("1")
        .nonce(nonce)
        .salt(generate_salt())
        .sign(keypair, orderbook)
    )

    # 4. Submit
    response = await client.orders().submit(request)
    print(
        f"submitted: {response.order_hash} "
        f"filled={response.filled} remaining={response.remaining} "
        f"fills={len(response.fills)}"
    )

    await client.close()


asyncio.run(main())
