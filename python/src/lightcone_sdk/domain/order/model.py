"""Order domain model: order kind, committed status, and the limit order."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from enum import Enum

from ...shared.types import FundingSource, OrderBookId, PubkeyStr, Side, TimeInForce


class OrderType(str, Enum):
    """Supported resting-order kind."""

    LIMIT = "limit"

    def label(self) -> str:
        return {
            OrderType.LIMIT: "Limit",
        }[self]


class OrderStatus(str, Enum):
    """Committed lifecycle status of an order.

    Lowercase on the wire, as in ``GET /api/users/order-fills``. Derived with
    the backend's precedence: a closure reason wins, then a complete confirmed
    fill, then pending (unconfirmed) fills, otherwise open.
    """

    #: Resting, or awaiting its first match.
    OPEN = "open"
    #: Some matched base awaits on-chain confirmation.
    PENDING = "pending"
    #: The whole original base is confirmed filled.
    FILLED = "filled"
    #: Stopped resting (cancelled, expired, closure cutoff, or the remainder of
    #: an IOC/FOK order); see the closed reason.
    CLOSED = "closed"
    #: A status this SDK version does not know (decoded only, never derived).
    UNKNOWN = "unknown"

    @classmethod
    def from_wire(cls, value: str) -> OrderStatus:
        for member in cls:
            if member is not cls.UNKNOWN and member.value == value:
                return member
        return cls.UNKNOWN

    @staticmethod
    def derive(
        closed_reason: str | None,
        original_base: Decimal,
        confirmed_base: Decimal,
        pending_base: Decimal,
    ) -> OrderStatus:
        """Apply the backend's status precedence to committed base quantities."""
        if closed_reason is not None:
            return OrderStatus.CLOSED
        if confirmed_base == original_base:
            return OrderStatus.FILLED
        if pending_base > 0:
            return OrderStatus.PENDING
        return OrderStatus.OPEN


@dataclass(kw_only=True)
class LimitOrder:
    """A limit order's committed state.

    Size fields are ``Decimal`` in base-token units and satisfy
    ``size = filled_size + pending_size + remaining_size + cancelled_size``.
    """

    market_pubkey: PubkeyStr
    orderbook_id: OrderBookId
    base_mint: PubkeyStr
    quote_mint: PubkeyStr
    order_hash: str
    side: Side
    #: Limit price, quote units per base unit.
    price: Decimal
    #: Original order size.
    size: Decimal
    #: Base filled by confirmed (on-chain authenticated) executions.
    filled_size: Decimal
    #: Base matched but awaiting on-chain confirmation.
    pending_size: Decimal
    #: Base still resting on the book.
    remaining_size: Decimal
    #: Base cancelled (explicitly, by expiry, IOC/FOK remainder, or closure).
    cancelled_size: Decimal
    #: ``None`` when the backend reported a policy this SDK version does not know.
    time_in_force: TimeInForce | None
    funding_source: FundingSource
    closed_reason: str | None
    status: OrderStatus
    #: Engine acceptance sequence; closures cut off by it.
    accepted_seq: int
    #: Committed revision of this state; newer revisions supersede older ones.
    committed_revision: int
    created_at: datetime
    #: Unix seconds; ``0`` means no expiration.
    expiration: int

    def is_live(self) -> bool:
        """True while the order rests on the book or has fills awaiting
        confirmation (which may still return to the book if they fail)."""
        return self.remaining_size > 0 or self.pending_size > 0


__all__ = ["LimitOrder", "OrderStatus", "OrderType"]
