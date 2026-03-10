"""Notification domain types."""

from dataclasses import dataclass
from typing import Optional


class NotificationKind:
    ORDER_FILLED = "order_filled"
    MARKET_RESOLVED = "market_resolved"
    TRIGGER_EXECUTED = "trigger_executed"
    SYSTEM = "system"


@dataclass
class MarketData:
    market_pubkey: str
    market_name: Optional[str] = None


@dataclass
class MarketResolvedData:
    market_pubkey: str
    market_name: Optional[str] = None
    winning_outcome: Optional[int] = None


@dataclass
class OrderFilledData:
    order_hash: str
    orderbook_id: str
    fill_amount: int = 0
    price: Optional[str] = None


@dataclass
class Notification:
    id: str
    kind: str
    message: Optional[str] = None
    created_at: Optional[str] = None
    read: bool = False
    market_data: Optional[MarketData] = None
    market_resolved_data: Optional[MarketResolvedData] = None
    order_filled_data: Optional[OrderFilledData] = None


__all__ = [
    "NotificationKind",
    "MarketData",
    "MarketResolvedData",
    "OrderFilledData",
    "Notification",
]
