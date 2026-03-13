"""Historical price history line data at various resolutions."""

import asyncio
import json

from common import rest_client, market_and_orderbook, unix_timestamp
from src.shared.types import Resolution


async def main():
    client = rest_client()
    _, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id

    # Fetch 7-day history at 1-hour resolution
    to_ts = unix_timestamp()
    from_ts = to_ts - 7 * 24 * 60 * 60

    history = await client.price_history().get(
        orderbook_id, Resolution.ONE_HOUR.as_str(), from_ts, to_ts
    )

    print(json.dumps([{"time": p.time, "value": p.value} for p in history], indent=2))

    await client.close()


asyncio.run(main())
