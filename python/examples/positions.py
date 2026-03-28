"""User positions across all markets and per-market."""

import asyncio

from common import rest_client, get_keypair, login, market


async def main():
    client = rest_client()
    keypair = get_keypair()
    user = await login(client, keypair)
    m = await market(client)

    # 1. All positions across markets
    all_positions = await client.positions().get(user.wallet_address)
    print(f"wallet: {user.wallet_address}")
    print(f"markets with positions: {all_positions.total_markets}")

    # 2. Positions for a specific market
    per_market = await client.positions().get_for_market(
        user.wallet_address, m.pubkey
    )
    print(f"positions in {m.slug}: {len(per_market.positions)}")

    await client.close()


asyncio.run(main())
