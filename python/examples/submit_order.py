"""LimitOrderEnvelope with human-readable price/size, auto-scaling, and fill tracking."""

import asyncio

from solders.pubkey import Pubkey

from common import (
    client as make_client,
    wallet,
    login,
    market_and_orderbook,
    scaling_decimals,
)
from lightcone_sdk.program.envelope import LimitOrderEnvelope


async def main():
    client = make_client()
    keypair = wallet()
    await login(client, keypair)

    # 1. Fetch market, orderbook, and decimals
    m, orderbook = await market_and_orderbook(client)
    decimals = await scaling_decimals(client, orderbook)
    base_mint = Pubkey.from_string(orderbook.base.mint)
    quote_mint = Pubkey.from_string(orderbook.quote.mint)

    # 2. Get a fresh nonce from on-chain
    nonce = await client.orders().current_nonce(keypair.pubkey())

    # 3. Build, scale, sign a limit order
    request = (
        LimitOrderEnvelope()
        .maker(keypair.pubkey())
        .market(Pubkey.from_string(m.pubkey))
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .price("0.55")
        .size("1")
        .nonce(nonce)
        .apply_scaling(decimals=decimals)
        .sign(keypair, orderbook.orderbook_id)
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
