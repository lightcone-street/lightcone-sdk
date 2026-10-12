"""Order wire-to-domain conversion."""

from __future__ import annotations

from typing import TYPE_CHECKING

from ...error import ValidationError
from .model import LimitOrder, OrderStatus
from .wire import OrderUpdate, UserOrder, UserSnapshotOrder

if TYPE_CHECKING:
    from .state import UserOpenLimitOrders


def limit_order_from_update(update: OrderUpdate) -> LimitOrder:
    """Convert a live ``order`` fact.

    Raises :class:`~lightcone_sdk.error.ValidationError` when its signed
    amounts define no limit price.
    """
    price = update.price()
    if price is None:
        raise ValidationError(
            f"order {update.order_hash}: signed amounts define no limit price"
        )
    return LimitOrder(
        status=OrderStatus.derive(
            update.closed_reason,
            update.original_base,
            update.confirmed_base,
            update.pending_base,
        ),
        market_pubkey=update.market_pubkey,
        orderbook_id=update.orderbook_id,
        base_mint=update.base_mint,
        quote_mint=update.quote_mint,
        order_hash=update.order_hash,
        side=update.side,
        price=price,
        size=update.original_base,
        filled_size=update.confirmed_base,
        pending_size=update.pending_base,
        remaining_size=update.open_base,
        cancelled_size=update.cancelled_base,
        time_in_force=update.tif,
        funding_source=update.funding_source,
        closed_reason=update.closed_reason,
        accepted_seq=update.accepted_seq,
        committed_revision=update.commit.committed_revision,
        created_at=update.accepted_at,
        expiration=update.expiration,
    )


def limit_order_from_snapshot(order: UserSnapshotOrder) -> LimitOrder:
    """Convert one order of the WS ``user`` snapshot."""
    return LimitOrder(
        status=OrderStatus.derive(
            order.closed_reason,
            order.original_base,
            order.confirmed_base,
            order.pending_base,
        ),
        market_pubkey=order.market_pubkey,
        orderbook_id=order.orderbook_id,
        base_mint=order.base_mint,
        quote_mint=order.quote_mint,
        order_hash=order.order_hash,
        side=order.side,
        price=order.price,
        size=order.original_base,
        filled_size=order.confirmed_base,
        pending_size=order.pending_base,
        remaining_size=order.open_base,
        cancelled_size=order.cancelled_base,
        time_in_force=order.tif,
        funding_source=order.funding_source,
        closed_reason=order.closed_reason,
        accepted_seq=order.accepted_seq,
        committed_revision=order.committed_revision,
        created_at=order.created_at,
        expiration=order.expiration,
    )


def limit_order_from_user_order(order: UserOrder) -> LimitOrder | None:
    """Convert a REST order; ``None`` when the backend omitted the recorded quantities."""
    state = order.state
    if state is None:
        return None
    return LimitOrder(
        status=OrderStatus.derive(
            state.closed_reason,
            state.original_base,
            state.confirmed_base,
            state.pending_base,
        ),
        market_pubkey=order.market_pubkey,
        orderbook_id=order.orderbook_id,
        base_mint=order.base_mint,
        quote_mint=order.quote_mint,
        order_hash=order.order_hash,
        side=order.side,
        price=order.price,
        size=state.original_base,
        filled_size=state.confirmed_base,
        pending_size=state.pending_base,
        remaining_size=state.open_base,
        cancelled_size=state.cancelled_base,
        time_in_force=order.tif,
        funding_source=order.source,
        closed_reason=state.closed_reason,
        accepted_seq=state.accepted_seq,
        committed_revision=state.committed_revision,
        created_at=order.created_at,
        expiration=order.expiration,
    )


def convert_snapshot_orders(orders: list[UserSnapshotOrder]) -> UserOpenLimitOrders:
    """Build the open-order container from a WS ``user`` snapshot page.

    Orders that are neither resting nor awaiting fill confirmation are
    skipped. The snapshot is the first bounded page only; merge later REST
    pages with :meth:`UserOpenLimitOrders.apply_order`.
    """
    from .state import UserOpenLimitOrders

    open_orders = UserOpenLimitOrders()
    for order in orders:
        open_orders.apply_order(limit_order_from_snapshot(order))
    return open_orders


__all__ = [
    "convert_snapshot_orders",
    "limit_order_from_snapshot",
    "limit_order_from_update",
    "limit_order_from_user_order",
]
