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


@dataclass
class TradesPage:
    trades: list[Trade] = field(default_factory=list)
    next_cursor: Optional[str] = None
    has_more: bool = False


__all__ = ["Trade", "TradesPage"]
