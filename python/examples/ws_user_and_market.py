"""Authenticated user stream (orders, balances) + market lifecycle events."""

import asyncio
import json

from common import rest_client, wallet, login, market_and_orderbook
from src.ws import WsEventType, MessageInType
from src.ws.subscriptions import UserParams, MarketParams


async def main():
    client = rest_client()
    keypair = wallet()

    # Login required for user stream
    await login(client, keypair)
    pubkey = str(keypair.pubkey())
    print("logged in:", pubkey)

    m, _ = await market_and_orderbook(client)

    # Connect WebSocket (auth token passed automatically)
    ws = client.ws()
    await ws.connect()
    print("connected")

    # Subscribe to user events and market events
    await ws.subscribe(UserParams(wallet_address=pubkey))
    await ws.subscribe(MarketParams(market_pubkey=m.pubkey))

    event_count = 0
    max_events = 15
    done = asyncio.Event()

    def on_event(event):
        nonlocal event_count
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message

            if msg.type == MessageInType.AUTH.value:
                print("[auth]", msg.data)
            elif msg.type == MessageInType.USER.value:
                text = str(msg.data)
                print("[user]", text[:200])
            elif msg.type == MessageInType.MARKET.value:
                text = str(msg.data)
                print("[market]", text[:200])

        elif event.type == WsEventType.ERROR:
            print("ws error:", event.error)
        elif event.type == WsEventType.DISCONNECTED:
            print("disconnected:", event.reason)

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
