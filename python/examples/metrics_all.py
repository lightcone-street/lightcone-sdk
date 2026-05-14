"""Exercise every metrics endpoint end-to-end.

Useful as a wire-type smoke test: if any field parsing is wrong, this example
will fail.

Usage:
    LIGHTCONE_ENV=local python examples/metrics_all.py
"""

import asyncio

from common import get_keypair, login, rest_client
from lightcone_sdk.domain.metrics import MetricsHistoryQuery


async def main():
    client = rest_client()
    keypair = get_keypair()
    await login(client, keypair)

    metrics = client.metrics()

    # Platform
    platform = await metrics.platform()
    print(
        "platform: "
        f"24h=${platform.volume_24h_usd}, "
        f"7d=${platform.volume_7d_usd}, "
        f"open_interest=${platform.open_interest_usd}, "
        f"active_markets={platform.active_markets}, "
        f"active_orderbooks={platform.active_orderbooks}"
    )
    print(f"  deposit token volumes: {len(platform.deposit_token_volumes)}")

    # Markets list
    markets = await metrics.markets()
    print(f"markets: {len(markets.markets)} entries (total={markets.total})")
    for entry in markets.markets[:3]:
        print(
            f"  - {entry.market_name or '?'} "
            f"-- 24h=${entry.volume_24h_usd} "
            f"(share={entry.platform_volume_share_24h_pct}%)"
        )

    # Market detail and orderbook detail
    if markets.markets:
        top_market = markets.markets[0]
        detail = await metrics.market(top_market.market_pubkey)
        print(
            f"market detail {detail.market_pubkey}: "
            f"outcomes={len(detail.outcome_volumes)}, "
            f"orderbooks={len(detail.orderbook_volumes)}"
        )

        if detail.orderbook_volumes:
            first_orderbook = detail.orderbook_volumes[0]
            orderbook_metrics = await metrics.orderbook(first_orderbook.orderbook_id)
            print(
                f"orderbook {orderbook_metrics.orderbook_id}: "
                f"24h_usd=${orderbook_metrics.volume_24h_usd} "
                f"24h_base={orderbook_metrics.volume_24h_base}"
            )

    # Categories
    categories = await metrics.categories()
    print(f"categories: {len(categories.categories)}")
    if categories.categories:
        first_category = categories.categories[0]
        detail = await metrics.category(first_category.category)
        print(
            f"category '{detail.category}': "
            f"24h=${detail.volume_24h_usd}, "
            f"traders_24h={detail.unique_traders_24h}"
        )

    # Deposit tokens
    deposit_tokens = await metrics.deposit_tokens()
    print(f"deposit tokens: {len(deposit_tokens.deposit_tokens)}")

    deposit_token_history = await metrics.deposit_tokens_volume_history()
    print(
        f"deposit token volume history @ {deposit_token_history.resolution}: "
        f"{len(deposit_token_history.points)} days, "
        f"total=${deposit_token_history.volume_total_usd}"
    )

    open_interest_history = await metrics.open_interest_history()
    print(
        f"open interest history @ {open_interest_history.resolution}: "
        f"{len(open_interest_history.points)} days, "
        f"latest=${open_interest_history.latest_open_interest_usd}"
    )

    unique_traders_history = await metrics.unique_traders_history()
    print(
        f"unique traders history @ {unique_traders_history.resolution}: "
        f"{len(unique_traders_history.points)} days, "
        f"latest={unique_traders_history.latest_unique_traders}"
    )

    # Leaderboard
    board = await metrics.leaderboard(5)
    print(f"leaderboard ({board.period}): {len(board.entries)} entries")
    for entry in board.entries:
        name = entry.market_name or entry.market_pubkey
        print(f"  #{entry.rank} {name} -- 24h=${entry.volume_24h_usd}")

    # History
    history = await metrics.history(
        "platform",
        "platform",
        MetricsHistoryQuery(),
    )
    print(
        f"history platform/platform @ {history.resolution}: "
        f"{len(history.points)} buckets"
    )

    # Per-user metrics using the SDK's captured auth token
    user_metrics = await metrics.user()
    print(
        f"user (jwt) {user_metrics.wallet_address}: "
        f"outcomes_traded={user_metrics.total_outcomes_traded} "
        f"volume=${user_metrics.total_volume_usd} "
        f"referrals_used={user_metrics.total_referrals_used}"
    )

    # Public path-based variant, no auth required.
    by_wallet = await metrics.user_by_wallet(str(keypair.pubkey()))
    print(
        f"user (by-wallet) {by_wallet.wallet_address}: "
        f"outcomes_traded={by_wallet.total_outcomes_traded} "
        f"volume=${by_wallet.total_volume_usd}"
    )

    await client.close()


asyncio.run(main())
