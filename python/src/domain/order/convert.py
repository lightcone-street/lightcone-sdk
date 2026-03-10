"""Order wire-to-domain conversion."""

from typing import Optional
from . import Order, OrderStatus, SubmitOrderResponse, FillInfo, TriggerOrder, UserSnapshotOrder
from .wire import WsOrder
from .state import UserOpenOrders, UserTriggerOrders


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


def limit_snapshot_to_order(snapshot: UserSnapshotOrder) -> Order:
    """Convert a limit-type UserSnapshotOrder to an Order domain type."""
    try:
        status = OrderStatus(snapshot.status.upper())
    except ValueError:
        status = OrderStatus.OPEN

    return Order(
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


def trigger_snapshot_to_order(snapshot: UserSnapshotOrder) -> TriggerOrder:
    """Convert a trigger-type UserSnapshotOrder to a TriggerOrder domain type."""
    return TriggerOrder(
        trigger_order_id=snapshot.trigger_order_id or "",
        order_hash=snapshot.order_hash,
        market_pubkey=snapshot.market_pubkey,
        orderbook_id=snapshot.orderbook_id,
        trigger_price=snapshot.trigger_price or "0",
        trigger_type=snapshot.trigger_type or 0,
        side=snapshot.side,
        amount_in=snapshot.amount_in,
        amount_out=snapshot.amount_out,
        time_in_force=snapshot.time_in_force or 0,
        created_at=snapshot.created_at,
    )


def split_snapshot_orders(
    snapshots: list[UserSnapshotOrder],
) -> tuple[UserOpenOrders, UserTriggerOrders]:
    """Split a list of UserSnapshotOrders into limit and trigger containers."""
    open_orders = UserOpenOrders()
    trigger_orders = UserTriggerOrders()

    for s in snapshots:
        if s.order_type == "trigger":
            trigger_orders.insert(trigger_snapshot_to_order(s))
        else:
            order = limit_snapshot_to_order(s)
            open_orders.upsert(order)

    return open_orders, trigger_orders
