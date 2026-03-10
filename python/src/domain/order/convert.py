"""Order wire-to-domain conversion."""

from . import Order, OrderStatus, OrderType, SubmitOrderResponse, FillInfo
from .wire import WsOrder


def order_from_ws(ws: WsOrder, market_pubkey: str, orderbook_id: str) -> Order:
    status = OrderStatus.OPEN
    if ws.status:
        try:
            status = OrderStatus(ws.status.lower())
        except ValueError:
            pass

    return Order(
        order_hash=ws.order_hash,
        market_pubkey=market_pubkey,
        orderbook_id=orderbook_id,
        side=ws.side,
        size=ws.size,
        price=ws.price,
        filled_size=ws.filled_size,
        remaining_size=ws.remaining_size,
        status=status,
    )


def submit_response_from_dict(d: dict) -> SubmitOrderResponse:
    fills = [
        FillInfo(
            fill_amount=f.get("fill_amount", 0),
            remaining=f.get("remaining", 0),
            price=f.get("price"),
            timestamp=f.get("timestamp"),
        )
        for f in d.get("fills", [])
    ]
    return SubmitOrderResponse(
        order_hash=d.get("order_hash", ""),
        status=d.get("status", ""),
        filled=d.get("filled", 0),
        remaining=d.get("remaining", 0),
        fills=fills,
    )
