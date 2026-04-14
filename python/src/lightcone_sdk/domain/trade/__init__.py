"""Trade domain types."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class Trade:
    """Trade domain type."""
    orderbook_id: str
    trade_id: str
    timestamp: str
    price: str
    size: str
    side: int
    sequence: int = 0
    """Monotonic sequence number per orderbook for ordering guarantees. 0 for REST trades."""
    cursor_id: Optional[int] = None


@dataclass
class TradesPage:
    trades: list[Trade] = field(default_factory=list)
    next_cursor: Optional[int] = None
    has_more: bool = False


__all__ = ["Trade", "TradesPage"]
