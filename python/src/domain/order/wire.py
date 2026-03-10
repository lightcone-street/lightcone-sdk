"""Order wire types - raw API shapes."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class WsOrder:
    """WebSocket order update."""
    order_hash: str
    side: int
    price: str
    size: str
    filled_size: Optional[str] = None
    remaining_size: Optional[str] = None
    status: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "WsOrder":
        return WsOrder(
            order_hash=d.get("order_hash", ""),
            side=d.get("side", 0),
            price=d.get("price", "0"),
            size=d.get("size", "0"),
            filled_size=d.get("filled_size"),
            remaining_size=d.get("remaining_size"),
            status=d.get("status"),
        )


@dataclass
class OrderUpdate:
    """WebSocket order update wrapper."""
    market_pubkey: str
    orderbook_id: str
    timestamp: Optional[str] = None
    order: Optional[WsOrder] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderUpdate":
        order_data = d.get("order")
        return OrderUpdate(
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            timestamp=d.get("timestamp"),
            order=WsOrder.from_dict(order_data) if order_data else None,
        )


@dataclass
class TriggerOrderUpdate:
    trigger_order_id: str
    status: str
    trigger_price: Optional[str] = None
    timestamp: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "TriggerOrderUpdate":
        return TriggerOrderUpdate(
            trigger_order_id=d.get("trigger_order_id", ""),
            status=d.get("status", ""),
            trigger_price=d.get("trigger_price"),
            timestamp=d.get("timestamp"),
        )


@dataclass
class AuthUpdate:
    authenticated: bool = False
    wallet_address: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "AuthUpdate":
        return AuthUpdate(
            authenticated=d.get("authenticated", False),
            wallet_address=d.get("wallet_address"),
        )
