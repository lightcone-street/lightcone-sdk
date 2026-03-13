"""LimitOrderEnvelope with human-readable price/size, auto-scaling, and fill tracking."""

import asyncio

from solders.pubkey import Pubkey

from common import (
    rest_client,
    rpc_client,
    wallet,
    login,
    market_and_orderbook,
    scaling_decimals,
    fresh_order_nonce,
)
from src.program.envelope import LimitOrderEnvelope


async def main():
    client = rest_client()
    rpc = rpc_client()
    keypair = wallet()
    await login(client, keypair)

    # 1. Fetch market, orderbook, and decimals
    m, orderbook = await market_and_orderbook(client)
    decimals = await scaling_decimals(client, orderbook)
    base_mint = Pubkey.from_string(orderbook.base_token)
    quote_mint = Pubkey.from_string(orderbook.quote_token)

    # 2. Get a fresh nonce from on-chain
    nonce = await fresh_order_nonce(rpc, keypair.pubkey())

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
    await rpc.connection.close()


asyncio.run(main())
