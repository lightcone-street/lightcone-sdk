"""User positions across all markets and per-market."""

import asyncio
import json

from common import rest_client, wallet, login, market


async def main():
    client = rest_client()
    keypair = wallet()
    user = await login(client, keypair)
    m = await market(client)

    # 1. All positions across markets
    all_positions = await client.positions().get(user.wallet_address)
    print(f"wallet: {user.wallet_address}")
    print(f"total positions: {len(all_positions)}")

    # 2. Positions for a specific market
    per_market = await client.positions().get_for_market(
        user.wallet_address, m.pubkey
    )
    print(f"positions in {m.slug}: {len(per_market)}")

    await client.close()


asyncio.run(main())
