"""Fetch open orders for an authenticated user."""

import asyncio

from common import rest_client, get_keypair, login


async def main():
    client = rest_client()
    keypair = get_keypair()
    await login(client, keypair)
    pubkey = str(keypair.pubkey())
    print(f"logged in: {pubkey}")

    # 1. First page of open and pending orders, with a first page of funding
    #    accounts. Quantities are committed at `committed_revision`.
    page = await client.orders().get_user_orders(50)

    print(f"orders: {len(page.orders)} (revision {page.committed_revision})")
    print(f"funding accounts: {len(page.funding_accounts)}")
    print(f"has more: {page.has_more}")

    if page.orders:
        first = page.orders[0]
        print(
            f"first order: {first.order_hash} {first.side.as_wire()} "
            f"@ {first.price} tif={first.tif.as_wire() if first.tif else 'unknown'}"
        )
        if first.state is not None:
            print(
                f"  open={first.state.open_base} pending={first.state.pending_base} "
                f"filled={first.state.confirmed_base}"
            )

    # 2. Pagination. A page can be empty while `has_more` is true.
    if page.has_more and page.next_cursor:
        page2 = await client.orders().get_user_orders(50, page.next_cursor)
        print(f"next page: {len(page2.orders)} order(s)")
    else:
        print("no more pages")

    await client.close()


asyncio.run(main())
