"""Fetch orderbook depth (bids/asks) and decimal precision metadata."""

import asyncio

from common import rest_client, market_and_orderbook


async def main():
    client = rest_client()
    m, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id

    # 1. Fetch orderbook depth
    depth = await client.orderbooks().get(orderbook_id, 10)
    print("market:", m.slug)
    print("orderbook:", orderbook_id)
    print(f"best bid: {depth.best_bid}, best ask: {depth.best_ask}")
    print(f"levels: {len(depth.bids)} bids / {len(depth.asks)} asks")

    # 2. Fetch decimal precision metadata
    decimals = await client.orderbooks().decimals(orderbook_id)
    print(f"decimals: base={decimals.base_decimals}, quote={decimals.quote_decimals}")

    await client.close()


asyncio.run(main())
