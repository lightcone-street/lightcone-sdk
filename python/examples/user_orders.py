"""Fetch open orders for an authenticated user."""

import asyncio

from common import rest_client, wallet, login


async def main():
    client = rest_client()
    keypair = wallet()
    await login(client, keypair)
    pubkey = str(keypair.pubkey())
    print(f"logged in: {pubkey}")

    # 1. First page of user orders
    snapshot = await client.orders().get_user_orders(pubkey, 50)

    limit_orders = [o for o in snapshot.orders if o.order_type == "limit"]
    trigger_orders = [o for o in snapshot.orders if o.order_type == "trigger"]

    print(f"orders: {len(limit_orders)} limit / {len(trigger_orders)} trigger")
    print(f"balances: {len(snapshot.balances)} market / {len(snapshot.global_deposits)} global")
    print(f"has more: {snapshot.has_more}")

    for order in snapshot.orders:
        side = "BID" if order.side == 0 else "ASK"
        if order.order_type == "limit":
            print(
                f"  [limit] {order.order_hash} {side} @ {order.price} "
                f"size={order.size} remaining={order.remaining} filled={order.filled}"
            )
        else:
            print(
                f"  [trigger] {order.trigger_order_id} {side} @ {order.price} "
                f"trigger={order.trigger_price} size={order.size}"
            )

    for bal in snapshot.balances:
        print(f"  balance: market={bal.market_pubkey} orderbook={bal.orderbook_id}")
        for out in bal.outcomes:
            print(f"    outcome {out.outcome_index}: idle={out.idle} on_book={out.on_book}")

    for gd in snapshot.global_deposits:
        print(f"  global deposit: {gd.mint} = {gd.balance}")

    # 2. Pagination
    if snapshot.has_more and snapshot.next_cursor:
        page2 = await client.orders().get_user_orders(
            pubkey, 50, snapshot.next_cursor
        )
        print(f"next page: {len(page2.orders)} order(s)")
    else:
        print("no more pages")

    await client.close()


asyncio.run(main())
