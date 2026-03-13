"""Authenticated user stream (orders, balances) + market lifecycle events."""

import asyncio

from common import rest_client, wallet, login, market_and_orderbook
from src.ws import WsEventType, MessageInType
from src.ws.subscriptions import UserParams, MarketParams
from src.domain.order.wire import UserSnapshot, UserUpdate, OrderUpdate, TriggerOrderUpdate, UserBalanceUpdate


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

    saw_auth = False
    saw_user = False
    saw_market = False
    done = asyncio.Event()

    def on_event(event):
        nonlocal saw_auth, saw_user, saw_market
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message

            if msg.type == MessageInType.AUTH.value:
                auth = msg.data
                print(f"[auth] status={auth.status} wallet={auth.wallet_address}")
                saw_auth = True

            elif msg.type == MessageInType.USER.value:
                user_update = msg.data
                if isinstance(user_update, UserUpdate) and isinstance(user_update.data, UserSnapshot):
                    snap = user_update.data
                    print(f"[user] snapshot: {len(snap.orders)} orders, {len(snap.balances)} balances, {len(snap.global_deposits)} deposits")
                    for o in snap.orders[:5]:
                        side_str = "BID" if o.side == 0 else "ASK"
                        print(f"  order {o.order_hash[:12]}… {side_str} {o.size}@{o.price} status={o.status}")
                    for b in snap.balances[:5]:
                        for out in b.outcomes:
                            print(f"  balance ob={b.orderbook_id[:12]}… idx={out.outcome_index} idle={out.idle} on_book={out.on_book}")
                    for g in snap.global_deposits[:5]:
                        print(f"  global deposit mint={g.mint[:12]}… balance={g.balance}")
                elif isinstance(user_update, UserUpdate) and isinstance(user_update.data, OrderUpdate):
                    ou = user_update.data
                    print(f"[user] order update: type={ou.update_type} ob={ou.orderbook_id[:12]}…")
                    if ou.order:
                        print(f"  hash={ou.order.order_hash[:12]}… status={ou.order.status} remaining={ou.order.remaining}")
                elif isinstance(user_update, UserUpdate) and isinstance(user_update.data, UserBalanceUpdate):
                    bu = user_update.data
                    print(f"[user] balance update: ob={bu.orderbook_id[:12]}…")
                else:
                    print(f"[user] event_type={getattr(user_update, 'event_type', '?')}")
                saw_user = True

            elif msg.type == MessageInType.MARKET.value:
                text = str(msg.data)
                print("[market]", text[:500])
                saw_market = True

        elif event.type == WsEventType.ERROR:
            print("ws error:", event.error)
        elif event.type == WsEventType.DISCONNECTED:
            print("disconnected:", event.reason)

        if saw_auth and saw_user:
            done.set()

    ws.on(on_event)

    try:
        await asyncio.wait_for(done.wait(), timeout=15)
    except asyncio.TimeoutError:
        pass

    await ws.disconnect()
    print(f"market event received: {saw_market}")
    await client.close()


asyncio.run(main())
