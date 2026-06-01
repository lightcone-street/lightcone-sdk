"""Per-call cookie forwarding for SSR / route handlers.

Demonstrates the ``*_with_cookies`` variants on ``Positions``, ``Notifications``,
``Referrals``, ``Orders``, and ``Metrics``. These bypass the SDK's process-wide
``auth_token`` store and forward the supplied raw ``Cookie`` header for that
single call only — so a server can relay whatever auth cookies the browser sent
(``privy-token`` and/or ``lightcone-token``).

In a real SSR / route handler the header would be built from the incoming
request's cookie jar. Here we mimic that by:
  1. Logging in once (the SDK captures the ``lightcone-token`` internally).
  2. Reading the token off the client via ``client.auth_token`` and wrapping it
     in a ``Cookie`` header.
  3. Clearing the SDK's internal token to prove the ``*_with_cookies`` path
     doesn't depend on it.
  4. Calling each ``*_with_cookies`` method with the captured header.
"""

import asyncio

from common import rest_client, get_keypair, login


async def main():
    client = rest_client()
    keypair = get_keypair()
    await login(client, keypair)

    auth_token = client.auth_token
    if not auth_token:
        raise RuntimeError(
            "auth_token not set after login — SDK should have captured it"
        )
    client.clear_auth_token()

    # Native consumers authenticate with a lightcone-token; build the Cookie
    # header the same way a browser would send it.
    cookie_header = f"lightcone-token={auth_token}"

    positions = await client.positions().positions_with_cookies(cookie_header)
    print(f"markets with positions: {positions.total_markets}")

    balances = await client.positions().deposit_token_balances_with_cookies(cookie_header)
    print(f"tracked deposit balances: {len(balances)}")

    notifications = await client.notifications().fetch_with_cookies(cookie_header)
    print(f"notifications: {len(notifications)}")

    status = await client.referrals().get_status_with_cookies(cookie_header)
    print(f"referral codes: {len(status.referral_codes)}")

    orders = await client.orders().get_user_orders_with_cookies(50, None, cookie_header)
    print(f"open orders: {len(orders.orders)}")

    fills = await client.orders().get_user_order_fills_with_cookies(
        None, 50, None, cookie_header,
    )
    print(f"order fills: {len(fills.orders)}")

    user_metrics = await client.metrics().user_with_cookies(cookie_header)
    print(
        f"user metrics: volume_usd={user_metrics.total_volume_usd} "
        f"outcomes_traded={user_metrics.total_outcomes_traded}"
    )

    await client.close()


asyncio.run(main())
