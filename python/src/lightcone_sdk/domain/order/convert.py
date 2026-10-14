"""Order wire-to-domain conversion."""

from . import (
    FillInfo,
    LimitOrder,
    OrderStatus,
    SubmitOrderResponse,
    SubmitOrderStatus,
    UserSnapshotOrder,
)
from .state import UserOpenLimitOrders
from .wire import WsOrder


def order_from_ws(ws: WsOrder, market_pubkey: str, orderbook_id: str) -> LimitOrder:
    status = OrderStatus.OPEN
    if ws.status:
        try:
            status = OrderStatus(ws.status.upper())
        except ValueError:
            pass

    return LimitOrder(
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
        status=SubmitOrderStatus(d.get("status", "accepted")),
        filled=d.get("filled", "0"),
        remaining=d.get("remaining", "0"),
        fills=fills,
    )


def limit_snapshot_to_order(snapshot: UserSnapshotOrder) -> LimitOrder:
    """Convert a limit-type UserSnapshotOrder to a LimitOrder domain type."""
    try:
        status = OrderStatus(snapshot.status.upper())
    except ValueError:
        status = OrderStatus.OPEN

    return LimitOrder(
        market_pubkey=snapshot.market_pubkey,
        orderbook_id=snapshot.orderbook_id,
        order_hash=snapshot.order_hash,
        side=snapshot.side,
        size=snapshot.size,
        price=snapshot.price,
        filled_size=snapshot.filled,
        remaining_size=snapshot.remaining,
        created_at=snapshot.created_at,
        status=status,
        outcome_index=snapshot.outcome_index,
        tx_signature=snapshot.tx_signature,
        base_mint=snapshot.base_mint,
        quote_mint=snapshot.quote_mint,
    )


def convert_snapshot_orders(
    snapshots: list[UserSnapshotOrder],
) -> UserOpenLimitOrders:
    """Group open limit orders from a decoded account snapshot."""
    open_orders = UserOpenLimitOrders()

    for snapshot in snapshots:
        order = limit_snapshot_to_order(snapshot)
        if order.remaining_size not in ("0", "", None):
            open_orders.upsert(order)

    return open_orders
