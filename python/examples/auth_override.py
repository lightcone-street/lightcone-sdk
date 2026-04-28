"""Per-call cookie override for server-side cookie forwarding.

Demonstrates the ``*_with_auth_override`` variants on ``Positions``,
``Notifications``, ``Referrals``, and ``Orders``. These bypass the SDK's
process-wide ``auth_token`` store and pass the supplied token as a
``Cookie: auth_token=…`` header for that single call only.

In a real SSR / route handler the token would be extracted from the
incoming request's cookie jar. Here we mimic that by:
  1. Logging in once (the SDK captures the token internally).
  2. Reading the token off the client via ``client.auth_token``.
  3. Clearing the SDK's internal token to prove the override path doesn't
     depend on it.
  4. Calling each ``*_with_auth_override`` method with the captured token.
"""

import asyncio

from common import rest_client, get_keypair, login


async def main():
    client = rest_client()
    keypair = get_keypair()
    user = await login(client, keypair)

    auth_token = client.auth_token
    if not auth_token:
        raise RuntimeError(
            "auth_token not set after login — SDK should have captured it"
        )
    client.clear_auth_token()

    positions = await client.positions().positions_with_auth_override(auth_token)
    print(f"markets with positions: {positions.total_markets}")

    balances = await client.positions().deposit_token_balances_with_auth_override(
        auth_token,
    )
    print(f"tracked deposit balances: {len(balances)}")

    notifications = await client.notifications().fetch_with_auth_override(auth_token)
    print(f"notifications: {len(notifications)}")

    status = await client.referrals().get_status_with_auth_override(auth_token)
    print(f"referral codes: {len(status.referral_codes)}")

    orders = await client.orders().get_user_orders_with_auth_override(
        user.wallet_address, 50, None, auth_token,
    )
    print(f"open orders: {len(orders.orders)}")

    fills = await client.orders().get_user_order_fills_with_auth_override(
        user.wallet_address, None, 50, None, auth_token,
    )
    print(f"order fills: {len(fills.orders)}")

    await client.close()


asyncio.run(main())
