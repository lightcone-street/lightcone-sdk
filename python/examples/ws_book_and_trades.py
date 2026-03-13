"""Live orderbook depth with OrderbookSnapshot state + rolling TradeHistory buffer."""

import asyncio

from common import rest_client, market_and_orderbook
from src.ws import WsEventType, MessageInType
from src.ws.subscriptions import BookUpdateParams, TradesParams
from src.domain.orderbook.state import OrderbookSnapshot
from src.domain.trade.state import TradeHistory
from src.domain.trade import Trade


async def main():
    client = rest_client()
    _, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id

    # State trackers
    book = OrderbookSnapshot(orderbook_id=orderbook_id)
    trades = TradeHistory(orderbook_id=orderbook_id, max_size=20)

    # Connect WebSocket
    ws = client.ws()
    await ws.connect()
    print("connected")

    # Subscribe to book updates and trades
    await ws.subscribe(BookUpdateParams(orderbook_ids=[orderbook_id]))
    await ws.subscribe(TradesParams(orderbook_ids=[orderbook_id]))

    hits = 0
    max_hits = 4
    done = asyncio.Event()

    def on_event(event):
        nonlocal hits
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message

            if msg.type == MessageInType.BOOK_UPDATE.value:
                book.apply(msg.data)
                print(
                    f"book: seq={book.sequence} "
                    f"bid={book.best_bid()} "
                    f"ask={book.best_ask()}"
                )
                hits += 1

            elif msg.type == MessageInType.TRADES.value:
                ws_trade = msg.data
                trades.push(Trade(
                    orderbook_id=orderbook_id,
                    trade_id=ws_trade.trade_id,
                    timestamp=ws_trade.timestamp,
                    price=ws_trade.price,
                    size=ws_trade.size,
                    side=ws_trade.side,
                ))
                print(f"trade: {ws_trade.size} {ws_trade.side} @ {ws_trade.price}")
                hits += 1

        elif event.type == WsEventType.ERROR:
            print(f"ws error: {event.error}")

        if hits >= max_hits:
            done.set()

    ws.on(on_event)

    try:
        await asyncio.wait_for(done.wait(), timeout=30)
    except asyncio.TimeoutError:
        pass

    await ws.disconnect()
    print(f"buffered trades: {len(trades)}")
    await client.close()


asyncio.run(main())
