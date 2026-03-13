"""Best bid/ask ticker + price history line data with PriceHistoryState."""

import asyncio

from common import rest_client, market_and_orderbook
from lightcone_sdk.ws import WsEventType, MessageInType
from lightcone_sdk.ws.subscriptions import TickerParams, PriceHistoryParams
from lightcone_sdk.domain.price_history.state import PriceHistoryState
from lightcone_sdk.domain.price_history import LineData
from lightcone_sdk.domain.price_history.wire import (
    PriceHistorySnapshot,
    PriceHistoryUpdate,
    PriceHistoryHeartbeat,
)
from lightcone_sdk.shared.types import Resolution


async def main():
    client = rest_client()
    _, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id
    resolution = Resolution.ONE_MINUTE
    resolution_str = resolution.as_str()

    # State tracker
    history = PriceHistoryState()

    # Connect WebSocket
    ws = client.ws()
    await ws.connect()
    print("connected")

    # Subscribe to ticker and price history
    await ws.subscribe(TickerParams(orderbook_ids=[orderbook_id]))
    await ws.subscribe(PriceHistoryParams(
        orderbook_id=orderbook_id,
        resolution=resolution.as_str(),
        include_ohlcv=False,
    ))

    event_count = 0
    max_events = 4
    done = asyncio.Event()

    def on_event(event):
        nonlocal event_count
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message

            if msg.type == MessageInType.TICKER.value:
                ticker = msg.data
                print(
                    f"ticker: bid={ticker.best_bid} "
                    f"ask={ticker.best_ask} "
                    f"mid={ticker.mid_price}"
                )
                event_count += 1

            elif msg.type == MessageInType.PRICE_HISTORY.value:
                data = msg.data

                if isinstance(data, PriceHistorySnapshot):
                    prices = [
                        LineData(time=c.t, value=c.m or "0")
                        for c in data.candles
                    ]
                    history.apply_snapshot(
                        data.orderbook_id, data.resolution, prices
                    )
                    candles = history.get(orderbook_id, resolution_str)
                    print(f"price snapshot: {len(candles)} candle(s)")
                    event_count += 1

                elif isinstance(data, PriceHistoryUpdate):
                    if data.candle:
                        point = LineData(
                            time=data.candle.t, value=data.candle.m or "0"
                        )
                        history.apply_update(
                            data.orderbook_id, data.resolution, point
                        )
                        candles = history.get(orderbook_id, resolution_str)
                        latest = candles[-1] if candles else None
                        print(f"latest candle: {latest}")
                        event_count += 1

                elif isinstance(data, PriceHistoryHeartbeat):
                    print(f"heartbeat: {data.server_time}")

        elif event.type == WsEventType.ERROR:
            print(f"ws error: {event.error}")

        if event_count >= max_events:
            done.set()

    ws.on(on_event)

    try:
        await asyncio.wait_for(done.wait(), timeout=30)
    except asyncio.TimeoutError:
        pass

    await ws.disconnect()
    await client.close()


asyncio.run(main())
