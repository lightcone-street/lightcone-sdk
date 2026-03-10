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
    is_maker: bool = False
    remaining: Optional[str] = None
    filled: Optional[str] = None
    fill_amount: Optional[str] = None
    base_mint: str = ""
    quote_mint: str = ""
    outcome_index: int = 0
    created_at: Optional[str] = None

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
            is_maker=d.get("is_maker", False),
            remaining=d.get("remaining"),
            filled=d.get("filled"),
            fill_amount=d.get("fill_amount"),
            base_mint=d.get("base_mint", ""),
            quote_mint=d.get("quote_mint", ""),
            outcome_index=d.get("outcome_index", 0),
            created_at=d.get("created_at"),
        )


@dataclass
class OrderUpdate:
    """WebSocket order update wrapper."""
    market_pubkey: str
    orderbook_id: str
    timestamp: Optional[str] = None
    tx_signature: Optional[str] = None
    update_type: Optional[str] = None
    order: Optional[WsOrder] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderUpdate":
        order_data = d.get("order")
        return OrderUpdate(
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            timestamp=d.get("timestamp"),
            tx_signature=d.get("tx_signature"),
            update_type=d.get("update_type"),
            order=WsOrder.from_dict(order_data) if order_data else None,
        )


@dataclass
class TriggerOrderUpdate:
    trigger_order_id: str
    status: str
    market_pubkey: str = ""
    orderbook_id: str = ""
    order_hash: str = ""
    trigger_price: Optional[str] = None
    trigger_above: Optional[bool] = None
    update_type: Optional[str] = None
    result_status: Optional[str] = None
    result_filled: Optional[str] = None
    result_remaining: Optional[str] = None
    timestamp: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "TriggerOrderUpdate":
        return TriggerOrderUpdate(
            trigger_order_id=d.get("trigger_order_id", ""),
            status=d.get("status", ""),
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            order_hash=d.get("order_hash", ""),
            trigger_price=d.get("trigger_price"),
            trigger_above=d.get("trigger_above"),
            update_type=d.get("update_type"),
            result_status=d.get("result_status"),
            result_filled=d.get("result_filled"),
            result_remaining=d.get("result_remaining"),
            timestamp=d.get("timestamp"),
        )


@dataclass
class UserBalanceUpdate:
    """WebSocket user balance update."""
    market_pubkey: str = ""
    orderbook_id: str = ""
    balance: Optional[dict] = None
    timestamp: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "UserBalanceUpdate":
        return UserBalanceUpdate(
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            balance=d.get("balance"),
            timestamp=d.get("timestamp"),
        )


@dataclass
class NotificationUpdate:
    """WebSocket notification push."""
    notification: Optional[dict] = None

    @staticmethod
    def from_dict(d: dict) -> "NotificationUpdate":
        return NotificationUpdate(notification=d.get("notification"))


@dataclass
class UserSnapshot:
    """WebSocket user snapshot."""
    orders: list[dict] = field(default_factory=list)
    balances: list[dict] = field(default_factory=list)
    global_deposits: list[dict] = field(default_factory=list)
    notifications: list[dict] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UserSnapshot":
        return UserSnapshot(
            orders=d.get("orders", []),
            balances=d.get("balances", []),
            global_deposits=d.get("global_deposits", []),
            notifications=d.get("notifications", []),
        )


@dataclass
class AuthUpdate:
    authenticated: bool = False
    wallet_address: Optional[str] = None
    code: Optional[str] = None
    message: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "AuthUpdate":
        return AuthUpdate(
            authenticated=d.get("authenticated", False),
            wallet_address=d.get("wallet_address"),
            code=d.get("code"),
            message=d.get("message"),
        )
