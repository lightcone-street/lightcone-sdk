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
    outcome_icon_url_low: Optional[str] = None
    outcome_icon_url_medium: Optional[str] = None
    outcome_icon_url_high: Optional[str] = None


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
        if self.order_filled_data:
            return self.order_filled_data.market_slug
        if self.market_data:
            return self.market_data.market_slug
        return None

    @staticmethod
    def from_dict(d: dict) -> "Notification":
        kind_raw = d.get("kind", d.get("type", "global"))
        try:
            kind = NotificationKind(kind_raw)
        except ValueError:
            kind = NotificationKind.GLOBAL

        market_data = None
        market_resolved_data = None
        order_filled_data = None

        # Parse nested data payload based on kind
        data = d.get("data", {})
        if isinstance(data, dict):
            if kind == NotificationKind.MARKET_RESOLVED:
                market_resolved_data = MarketResolvedData(
                    market_pubkey=data.get("market_pubkey", ""),
                    market_slug=data.get("market_slug"),
                    market_name=data.get("market_name"),
                    winning_outcome=data.get("winning_outcome"),
                )
            elif kind == NotificationKind.ORDER_FILLED:
                order_filled_data = OrderFilledData(
                    order_hash=data.get("order_hash", ""),
                    market_pubkey=data.get("market_pubkey", ""),
                    side=data.get("side", ""),
                    price=data.get("price", "0"),
                    filled=data.get("filled", "0"),
                    remaining=data.get("remaining", "0"),
                    market_slug=data.get("market_slug"),
                    market_name=data.get("market_name"),
                    outcome_name=data.get("outcome_name"),
                    outcome_icon_url_low=data.get("outcome_icon_url_low"),
                    outcome_icon_url_medium=data.get("outcome_icon_url_medium"),
                    outcome_icon_url_high=data.get("outcome_icon_url_high"),
                )
            elif kind in (NotificationKind.NEW_MARKET, NotificationKind.RULES_CLARIFIED):
                market_data = MarketData(
                    market_pubkey=data.get("market_pubkey", ""),
                    market_slug=data.get("market_slug"),
                    market_name=data.get("market_name"),
                )

        return Notification(
            id=d.get("id", ""),
            kind=kind,
            title=d.get("title", ""),
            message=d.get("message", ""),
            expires_at=d.get("expires_at"),
            created_at=d.get("created_at"),
            market_data=market_data,
            market_resolved_data=market_resolved_data,
            order_filled_data=order_filled_data,
        )


__all__ = [
    "NotificationKind",
    "MarketData",
    "MarketResolvedData",
    "OrderFilledData",
    "Notification",
]
