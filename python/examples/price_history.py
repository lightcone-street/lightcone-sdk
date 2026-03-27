"""Historical orderbook and deposit-token price history."""

import asyncio
import json
from dataclasses import asdict

from common import rest_client, market_and_orderbook, unix_timestamp_ms
from lightcone_sdk.shared.types import Resolution


async def main():
    client = rest_client()
    market, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id
    if not market.deposit_assets:
        raise RuntimeError("selected market has no deposit assets")
    deposit_asset = market.deposit_assets[0].deposit_asset

    to_ts = unix_timestamp_ms()
    from_ts = to_ts - 7 * 24 * 60 * 60 * 1000

    orderbook_history = await client.price_history().get(
        orderbook_id,
        Resolution.ONE_HOUR,
        from_ts,
        to_ts,
        limit=10,
        include_ohlcv=True,
    )
    print("orderbook:")
    print(json.dumps(asdict(orderbook_history), indent=2))

    try:
        deposit_history = await client.price_history().get_deposit_asset(
            deposit_asset,
            Resolution.ONE_HOUR,
            from_ts,
            to_ts,
            limit=10,
        )
        print("deposit asset:")
        print(json.dumps(asdict(deposit_history), indent=2))
    except Exception as err:
        print(f"deposit asset price history not available: {err}")

    await client.close()


asyncio.run(main())
