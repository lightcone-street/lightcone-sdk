"""Fetch open orders for an authenticated user."""

import asyncio

from common import rest_client, get_keypair, login


async def main():
    client = rest_client()
    keypair = get_keypair()
    await login(client, keypair)
    pubkey = str(keypair.pubkey())
    print(f"logged in: {pubkey}")

    # 1. First page of user orders
    snapshot = await client.orders().get_user_orders(50)

    limit_orders = [o for o in snapshot.orders if o.order_type == "limit"]

    print(f"orders: {len(limit_orders)} limit")
    print(f"balances: {len(snapshot.balances)} market")
    print(f"has more: {snapshot.has_more}")

    if limit_orders:
        first = limit_orders[0]
        side = "BID" if first.side == 0 else "ASK"
        print(f"first limit: {first.order_hash} {side} @ {first.price}")

    # 2. Pagination
    if snapshot.next_cursor:
        page2 = await client.orders().get_user_orders(50, snapshot.next_cursor)
        print(f"next page: {len(page2.orders)} order(s)")
    else:
        print("no more pages")

    await client.close()


asyncio.run(main())
