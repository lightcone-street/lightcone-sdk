"""Order wire-to-domain conversion."""

from . import Order, OrderStatus, SubmitOrderResponse, FillInfo
from .wire import WsOrder


def order_from_ws(ws: WsOrder, market_pubkey: str, orderbook_id: str) -> Order:
    status = OrderStatus.OPEN
    if ws.status:
        try:
            status = OrderStatus(ws.status.upper())
        except ValueError:
            pass

    return Order(
        order_hash=ws.order_hash,
        market_pubkey=market_pubkey,
        orderbook_id=orderbook_id,
        side=ws.side,
        size=ws.size,
        price=ws.price,
        filled_size=ws.filled_size or "0",
        remaining_size=ws.remaining_size or "0",
        status=status,
    )


def submit_response_from_dict(d: dict) -> SubmitOrderResponse:
    fills = [
        FillInfo(
            counterparty=f.get("counterparty", ""),
            counterparty_order_hash=f.get("counterparty_order_hash", ""),
            fill_amount=f.get("fill_amount", "0"),
            price=f.get("price", "0"),
            is_maker=f.get("is_maker", False),
        )
        for f in d.get("fills", [])
    ]
    return SubmitOrderResponse(
        order_hash=d.get("order_hash", ""),
        filled=d.get("filled", "0"),
        remaining=d.get("remaining", "0"),
        fills=fills,
    )
