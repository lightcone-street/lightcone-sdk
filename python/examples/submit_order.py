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

    # 2. Get a fresh nonce and cache it on the client.
    #    Subsequent orders that omit .nonce() will auto-populate from this cache.
    nonce = await client.orders().current_nonce(keypair.pubkey())
    client.set_order_nonce(nonce)

    # 3. Build and submit a limit order via the unified submit() flow.
    #    submit() dispatches based on the client's signing strategy (native/wallet/privy).
    #    Nonce is auto-populated from the client cache since we don't call .nonce() here.
    response = await (
        client.orders().limit_order()
        .maker(keypair.pubkey())
        .market(Pubkey.from_string(m.pubkey))
        .base_mint(base_mint)
        .quote_mint(quote_mint)
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
