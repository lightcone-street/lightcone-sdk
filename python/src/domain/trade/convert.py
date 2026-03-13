"""Trade wire-to-domain conversion."""

from . import Trade
from .wire import TradeResponseWire, WsTrade


def trade_from_wire(wire: TradeResponseWire) -> Trade:
    return Trade(
        orderbook_id=wire.orderbook_id,
        trade_id=wire.id,
        timestamp=wire.executed_at or "",
        price=wire.price,
        size=wire.size,
        side=wire.side,
        market_pubkey=wire.market_pubkey or "",
    )


def trade_from_ws(ws: WsTrade) -> Trade:
    return Trade(
        orderbook_id=ws.orderbook_id,
        trade_id=ws.trade_id,
        timestamp=ws.timestamp,
        price=ws.price,
        size=ws.size,
        side=ws.side,
    )
