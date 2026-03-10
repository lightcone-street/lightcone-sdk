"""Best bid/ask ticker + price history candles with PriceHistoryState."""

import asyncio

from common import rest_client, market_and_orderbook
from src.ws import WsEventType, MessageInType
from src.ws.subscriptions import TickerParams, PriceHistoryParams
from src.domain.price_history.state import PriceHistoryState
from src.domain.price_history import LineData
from src.shared.types import Resolution


async def main():
    client = rest_client()
    _, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id
    resolution = Resolution.ONE_MINUTE

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
    max_events = 20
    done = asyncio.Event()

    def on_event(event):
        nonlocal event_count
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message

            if msg.type == MessageInType.TICKER.value:
                ticker = msg.data
                print(
                    f"[ticker] bid={getattr(ticker, 'best_bid', '-')} "
                    f"ask={getattr(ticker, 'best_ask', '-')} "
                    f"mid={getattr(ticker, 'mid', '-')}"
                )

            elif msg.type == MessageInType.PRICE_HISTORY.value:
                data = msg.data

                # Snapshot
                if hasattr(data, "prices"):
                    prices = [
                        LineData(time=c.t, value=getattr(c, "m", "0") or "0")
                        for c in data.prices
                    ]
                    history.apply_snapshot(
                        data.orderbook_id, data.resolution, prices
                    )
                    candles = history.get(orderbook_id, resolution.as_str())
                    print(f"[price_history] snapshot: {len(candles)} candle(s)")

                # Update
                elif hasattr(data, "t") and hasattr(data, "m"):
                    point = LineData(time=data.t, value=data.m or "0")
                    history.apply_update(
                        data.orderbook_id, data.resolution, point
                    )
                    candles = history.get(orderbook_id, resolution.as_str())
                    print(f"[price_history] update: t={data.t} mid={data.m or '-'}")
                    if candles:
                        print(f"  total candles: {len(candles)}")

                # Heartbeat
                elif hasattr(data, "server_time"):
                    print(f"[price_history] heartbeat: server_time={data.server_time}")

        elif event.type == WsEventType.ERROR:
            print("ws error:", event.error)

        event_count += 1
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
