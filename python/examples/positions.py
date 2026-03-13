"""User positions across all markets and per-market."""

import asyncio

from common import rest_client, wallet, login, market


async def main():
    client = rest_client()
    keypair = wallet()
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
    for pos in per_market.positions:
        print(f"  market: {pos.market_pubkey}")
        for o in pos.outcomes:
            print(f"    outcome {o.outcome_index}: idle={o.balance_idle} on_book={o.balance_on_book} total={o.balance}")

    await client.close()


asyncio.run(main())
