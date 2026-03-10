"""Notification domain types."""

from dataclasses import dataclass
from enum import Enum
from typing import Optional


class NotificationKind(str, Enum):
    MARKET_RESOLVED = "market_resolved"
    ORDER_FILLED = "order_filled"
    NEW_MARKET = "new_market"
    RULES_CLARIFIED = "rules_clarified"
    GLOBAL = "global"


@dataclass
class MarketData:
    market_pubkey: str
    market_slug: Optional[str] = None
    market_name: Optional[str] = None


@dataclass
class MarketResolvedData:
    market_pubkey: str
    market_slug: Optional[str] = None
    market_name: Optional[str] = None
    winning_outcome: Optional[int] = None


@dataclass
class OrderFilledData:
    order_hash: str
    market_pubkey: str
    side: str = ""
    price: str = "0"
    filled: str = "0"
    remaining: str = "0"
    market_slug: Optional[str] = None
    market_name: Optional[str] = None
    outcome_name: Optional[str] = None
    outcome_icon_url: Optional[str] = None


@dataclass
class Notification:
    id: str
    kind: NotificationKind
    title: str = ""
    message: str = ""
    expires_at: Optional[str] = None
    created_at: Optional[str] = None
    # Data payload varies by kind
    market_data: Optional[MarketData] = None
    market_resolved_data: Optional[MarketResolvedData] = None
    order_filled_data: Optional[OrderFilledData] = None

    def is_global(self) -> bool:
        return self.kind == NotificationKind.GLOBAL

    def market_slug(self) -> Optional[str]:
        if self.market_resolved_data:
            return self.market_resolved_data.market_slug
        if self.market_data:
            return self.market_data.market_slug
        return None


__all__ = [
    "NotificationKind",
    "MarketData",
    "MarketResolvedData",
    "OrderFilledData",
    "Notification",
]
