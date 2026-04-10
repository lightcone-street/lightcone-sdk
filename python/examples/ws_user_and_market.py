"""Authenticated user stream (orders, balances) + market lifecycle events."""

import asyncio

from common import rest_client, get_keypair, login, market
from lightcone_sdk.ws import WsEventType, MessageInType
from lightcone_sdk.ws.subscriptions import UserParams, MarketParams


async def main():
    client = rest_client()
    keypair = get_keypair()

    # Login required for user stream
    await login(client, keypair)
    pubkey = str(keypair.pubkey())
    print("logged in:", pubkey)

    m = await market(client)

    # Connect WebSocket (auth token passed automatically)
    ws = client.ws()
    await ws.connect()
    print("connected")

    # Subscribe to user events and market events
    await ws.subscribe(UserParams(wallet_address=pubkey))
    await ws.subscribe(MarketParams(market_pubkey=m.pubkey))

    saw_auth = False
    saw_user = False
    saw_market = False
    done = asyncio.Event()

    def on_event(event):
        nonlocal saw_auth, saw_user, saw_market
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message

            if msg.type == MessageInType.AUTH.value:
                print(f"auth: {msg.data}")
                saw_auth = True

            elif msg.type == MessageInType.USER.value:
                print(f"user: {msg.data}")
                saw_user = True

            elif msg.type == MessageInType.MARKET.value:
                print(f"market: {msg.data}")
                saw_market = True

        elif event.type == WsEventType.ERROR:
            print(f"ws error: {event.error}")
        elif event.type == WsEventType.DISCONNECTED:
            print("disconnected:", event.reason)

        if saw_auth and saw_user:
            done.set()

    ws.on(on_event)

    try:
        await asyncio.wait_for(done.wait(), timeout=30)
    except asyncio.TimeoutError:
        print("no more websocket data (timeout)")

    await ws.disconnect()
    if not saw_auth and not saw_user:
        raise RuntimeError("received no websocket events — connection may be broken")
    print(f"market event received: {saw_market}")
    await client.close()


asyncio.run(main())
