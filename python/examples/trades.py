"""Recent trade history with cursor-based pagination."""

import asyncio

from common import rest_client, market_and_orderbook


async def main():
    client = rest_client()
    _, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id

    # 1. First page of trades
    page1 = await client.trades().get(orderbook_id, 10)
    print(f"page 1: {len(page1.trades)} trade(s)")
    if page1.trades:
        t = page1.trades[0]
        print(f"  latest: {t.size} {t.side} @ {t.price}")

    # 2. Next page using cursor
    if page1.next_cursor:
        page2 = await client.trades().get(orderbook_id, 10, page1.next_cursor)
        print(f"page 2: {len(page2.trades)} trade(s)")
    else:
        print("no more pages")

    await client.close()


asyncio.run(main())
