"""Authenticated user's SPL deposit-token balances."""

import asyncio

from common import rest_client, get_keypair, login


async def main():
    client = rest_client()
    keypair = get_keypair()
    session = await login(client, keypair)
    wallet = session.user.trading_wallet(session.auth_method)

    snapshot = await client.positions().deposit_token_balances()

    print(f"wallet: {wallet}")
    print(f"context slot: {snapshot.context_slot}")
    print(f"tracked balances: {len(snapshot.balances)}")

    entries = sorted(snapshot.balances.values(), key=lambda b: b.symbol)
    for balance in entries:
        print(f"  {balance.symbol:>8}  {balance.mint:<42}  idle={balance.idle}")

    await client.close()


asyncio.run(main())
