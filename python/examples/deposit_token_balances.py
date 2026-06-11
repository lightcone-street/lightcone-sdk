"""Authenticated user's SPL deposit-token balances."""

import asyncio

from common import rest_client, get_keypair, login


async def main():
    client = rest_client()
    keypair = get_keypair()
    session = await login(client, keypair)
    wallet = session.user.trading_wallet(session.auth_method)

    balances = await client.positions().deposit_token_balances()

    print(f"wallet: {wallet}")
    print(f"tracked balances: {len(balances)}")

    entries = sorted(balances.values(), key=lambda b: b.symbol)
    for balance in entries:
        print(f"  {balance.symbol:>8}  {balance.mint:<42}  idle={balance.idle}")

    await client.close()


asyncio.run(main())
