"""Recent trade history with cursor-based pagination."""

import asyncio

from common import rest_client, market_and_orderbook


def print_trades(page_label: str, trades: list) -> None:
    print(f"{page_label}: {len(trades)} trade(s)")
    for trade in trades:
        print(
            f"  {trade.trade_id} {trade.timestamp} "
            f"{trade.size} {trade.side} @ {trade.price}"
        )


async def main():
    client = rest_client()
    _, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id

    # 1. First page of trades
    page1 = await client.trades().get(orderbook_id, 10)
    print_trades("page 1", page1.trades)
    if page1.trades:
        t = page1.trades[0]
        print(f"latest: {t.size} {t.side} @ {t.price}")

    # 2. Next page using cursor
    if page1.next_cursor is not None:
        page2 = await client.trades().get(orderbook_id, 10, page1.next_cursor)
        print_trades("page 2", page2.trades)
    else:
        print("no more pages")

    await client.close()


asyncio.run(main())
