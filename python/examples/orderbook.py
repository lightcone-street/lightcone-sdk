"""Fetch orderbook depth (bids/asks) and decimal precision metadata."""

import asyncio

from common import market_and_orderbook, rest_client

from lightcone_sdk.domain.orderbook.aggregation import BookAggregation


async def main():
    client = rest_client()
    m, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id

    # 1. Fetch orderbook depth (capped server-side at 20 levels per side)
    depth = await client.orderbooks().get(orderbook_id, 10)
    print("market:", m.slug)
    print("orderbook:", orderbook_id)
    print(f"best bid: {depth.best_bid}, best ask: {depth.best_ask}")
    print(f"levels: {len(depth.bids)} bids / {len(depth.asks)} asks")
    print(f"depth decimals: price={depth.decimals.price}, size={depth.decimals.size}")

    # 2. Hyperliquid-style aggregation: 5 significant figures, 1/2/5 mantissa
    # sub-steps. Bids bucket by flooring, asks by ceiling.
    grouped_aggregation = BookAggregation.validate(n_sig_figs=5, mantissa=2)
    grouped = await client.orderbooks().get(
        orderbook_id,
        n_sig_figs=grouped_aggregation.n_sig_figs,
        mantissa=grouped_aggregation.mantissa,
    )
    print(
        f"grouped ({grouped_aggregation.key_suffix()}): "
        f"{len(grouped.bids)} bids / {len(grouped.asks)} asks"
    )

    # 3. Fetch and cache the exact admission rules for this orderbook.
    decimals = await client.orderbooks().decimals(orderbook_id)
    print(
        "decimals: "
        f"price={decimals.price_decimals}, "
        f"base={decimals.base_decimals}, "
        f"quote={decimals.quote_decimals}"
    )

    await client.close()


asyncio.run(main())
