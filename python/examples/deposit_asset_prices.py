"""Snapshot of deposit-asset prices + per-asset live updates."""

import asyncio
import json
from dataclasses import asdict

from common import rest_client
from lightcone_sdk.domain.price_history.wire import (
    DepositAssetPriceSnapshot,
    DepositAssetPriceTick,
)
from lightcone_sdk.ws import MessageInType, WsEventType
from lightcone_sdk.ws.subscriptions import DepositAssetPriceParams


async def main():
    client = rest_client()

    # REST: snapshot of current prices for every active mint in `global_deposit_tokens`.
    snapshot = await client.price_history().get_deposit_asset_prices_snapshot()
    print(
        f"REST /api/deposit-asset-prices-snapshot ({len(snapshot.prices)} entries):"
    )
    print(json.dumps(asdict(snapshot), indent=2))

    if not snapshot.prices:
        print("snapshot has no entries — backend has no priced assets")
        await client.close()
        return

    # Pick the first asset and subscribe via WS for live updates.
    deposit_asset = next(iter(snapshot.prices))

    ws = client.ws()
    await ws.connect()
    print("connected")

    await ws.subscribe(DepositAssetPriceParams(deposit_asset=deposit_asset))

    event_count = 0
    max_events = 2
    done = asyncio.Event()

    def on_event(event):
        nonlocal event_count
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message
            if msg.type == MessageInType.DEPOSIT_ASSET_PRICE.value:
                data = msg.data
                if isinstance(data, DepositAssetPriceSnapshot):
                    print(f"WS snapshot: {data.deposit_asset} -> {data.price}")
                    event_count += 1
                elif isinstance(data, DepositAssetPriceTick):
                    print(
                        f"WS tick: {data.deposit_asset} -> {data.price} "
                        f"@ {data.event_time}"
                    )
                    event_count += 1
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
