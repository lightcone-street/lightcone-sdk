"""Live orderbook depth with OrderbookState state + rolling TradeHistory buffer.

Demonstrates holding two aggregation views of the same book on one
connection: full precision (pricing) and a grouped view (display).
"""

import asyncio

from common import rest_client, market_and_orderbook
from lightcone_sdk.ws import WsEventType, MessageInType
from lightcone_sdk.ws.subscriptions import BookUpdateParams, TradesParams
from lightcone_sdk.domain.orderbook.aggregation import BookAggregation
from lightcone_sdk.domain.orderbook.state import OrderbookState
from lightcone_sdk.domain.trade.state import TradeHistory
from lightcone_sdk.domain.trade import Trade


async def main():
    client = rest_client()
    _, orderbook = await market_and_orderbook(client)
    orderbook_id = orderbook.orderbook_id

    # One connection can hold multiple aggregation views of the same book.
    # Key the local state by the frame's aggregation: full precision for
    # pricing, a grouped view (5 sig figs, mantissa 2) for display.
    grouped_aggregation = BookAggregation.validate(n_sig_figs=5, mantissa=2)
    books = {
        BookAggregation(): OrderbookState(orderbook_id=orderbook_id),
        grouped_aggregation: OrderbookState(
            orderbook_id=orderbook_id, aggregation=grouped_aggregation
        ),
    }
    trades = TradeHistory(orderbook_id=orderbook_id, max_size=20)

    # Connect WebSocket
    ws = client.ws()
    await ws.connect()
    print("connected")

    # Subscribe the same book at full precision AND grouped, plus trades.
    await ws.subscribe(BookUpdateParams(orderbook_ids=[orderbook_id]))
    await ws.subscribe(
        BookUpdateParams(
            orderbook_ids=[orderbook_id],
            n_sig_figs=grouped_aggregation.n_sig_figs,
            mantissa=grouped_aggregation.mantissa,
        )
    )
    await ws.subscribe(TradesParams(orderbook_ids=[orderbook_id]))

    hits = 0
    max_hits = 4
    done = asyncio.Event()

    async def on_event(event):
        nonlocal hits
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message

            if msg.type == MessageInType.BOOK_UPDATE.value:
                update = msg.data
                # Untagged frames are the full-precision view; frames from
                # the grouped subscription carry n_sig_figs/mantissa.
                aggregation = update.aggregation()
                book = books.get(aggregation)
                if book is None:
                    return
                if update.resync:
                    # Refresh exactly the affected view: re-subscribe with
                    # the SAME aggregation and reset its revision gate.
                    params = BookUpdateParams(
                        orderbook_ids=[update.orderbook_id],
                        n_sig_figs=aggregation.n_sig_figs,
                        mantissa=aggregation.mantissa,
                    )
                    await ws.unsubscribe(params)
                    book.begin_generation()
                    await ws.subscribe(params)
                    return
                book.apply(update)
                print(
                    f"book[{aggregation.key_suffix()}]: seq={book.sequence} "
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
                    sequence=ws_trade.sequence,
                ))
                print(f"trade: {ws_trade.size} {ws_trade.side} @ {ws_trade.price} seq={ws_trade.sequence}")
                hits += 1

        elif event.type == WsEventType.CONNECTED:
            for book in books.values():
                book.begin_generation()
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
