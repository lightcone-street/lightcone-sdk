"""Committed funding accounts (global deposits and conditional tokens) across
all markets and per market."""

import asyncio

from common import rest_client, get_keypair, login, market


async def main():
    client = rest_client()
    keypair = get_keypair()
    session = await login(client, keypair)
    wallet = session.user.trading_wallet(session.auth_method)
    m = await market(client)

    # 1. First page of funding accounts across markets (public route)
    all_positions = await client.positions().get(wallet)
    print(f"wallet: {wallet} (revision {all_positions.committed_revision})")
    for account in all_positions.funding_accounts:
        print(
            f"  {account.source.value} {account.mint}: observed={account.raw_observed} "
            f"order_reserved={account.order_reserved}"
        )

    # 2. Global accounts plus the conditional accounts of one market
    per_market = await client.positions().get_for_market(wallet, m.pubkey)
    print(f"funding accounts for {m.slug}: {len(per_market.funding_accounts)}")

    # 3. Further pages, as the authenticated owner
    cursor = all_positions.next_cursor
    while all_positions.has_more and cursor:
        page = await client.positions().positions_page(cursor=cursor)
        print(f"next page: {len(page.funding_accounts)} account(s)")
        if not page.has_more:
            break
        cursor = page.next_cursor

    await client.close()


asyncio.run(main())
