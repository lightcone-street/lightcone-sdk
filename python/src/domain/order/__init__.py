"""Order domain types."""

from dataclasses import dataclass, field
from enum import Enum, IntEnum
from typing import Optional


class OrderType(str, Enum):
    LIMIT = "limit"
    MARKET = "market"
    DEPOSIT = "deposit"
    WITHDRAW = "withdraw"


class OrderStatus(str, Enum):
    OPEN = "open"
    MATCHING = "matching"
    CANCELLED = "cancelled"
    FILLED = "filled"
    PENDING = "pending"


@dataclass
class FillInfo:
    fill_amount: int
    remaining: int
    price: Optional[str] = None
    timestamp: Optional[str] = None


@dataclass
class Order:
    """Order domain type."""
    order_hash: str
    market_pubkey: str
    orderbook_id: str
    side: int
    size: str
    price: str
    filled_size: Optional[str] = None
    remaining_size: Optional[str] = None
    created_at: Optional[str] = None
    status: OrderStatus = OrderStatus.OPEN
    outcome_index: Optional[int] = None
    order_type: OrderType = OrderType.LIMIT
    expiration: Optional[int] = None


@dataclass
class OrderEvent:
    """WebSocket order event."""
    type: str
    order: Optional[Order] = None
    fill: Optional[FillInfo] = None


@dataclass
class TriggerOrder:
    """Trigger order domain type."""
    trigger_order_id: str
    trigger_price: str
    trigger_type: int
    side: int
    amount_in: int
    amount_out: int
    time_in_force: int
    status: Optional[str] = None
    market_pubkey: Optional[str] = None
    orderbook_id: Optional[str] = None
    created_at: Optional[str] = None


@dataclass
class TriggerOrderResponse:
    trigger_order_id: str
    success: bool
    message: Optional[str] = None


@dataclass
class SubmitOrderResponse:
    order_hash: str
    status: str
    filled: int = 0
    remaining: int = 0
    fills: list[FillInfo] = field(default_factory=list)


@dataclass
class CancelBody:
    order_hash: str
    maker: str
    signature: str

    def to_dict(self) -> dict:
        return {
            "order_hash": self.order_hash,
            "maker": self.maker,
            "signature": self.signature,
        }


@dataclass
class CancelSuccess:
    order_hash: str
    success: bool


@dataclass
class CancelAllBody:
    user_pubkey: str
    signature: str
    timestamp: int
    orderbook_id: Optional[str] = None

    def to_dict(self) -> dict:
        d: dict = {
            "user_pubkey": self.user_pubkey,
            "signature": self.signature,
            "timestamp": self.timestamp,
        }
        if self.orderbook_id:
            d["orderbook_id"] = self.orderbook_id
        return d


@dataclass
class CancelAllSuccess:
    cancelled: list[str] = field(default_factory=list)
    success: bool = True


@dataclass
class CancelTriggerBody:
    trigger_order_id: str
    maker: str
    signature: str

    def to_dict(self) -> dict:
        return {
            "trigger_order_id": self.trigger_order_id,
            "maker": self.maker,
            "signature": self.signature,
        }


@dataclass
class CancelTriggerSuccess:
    trigger_order_id: str
    success: bool


@dataclass
class ConditionalBalance:
    mint: str
    idle: int = 0
    on_book: int = 0


@dataclass
class GlobalDepositBalance:
    mint: str
    amount: int = 0


@dataclass
class UserSnapshotBalance:
    market_pubkey: str
    balances: list[ConditionalBalance] = field(default_factory=list)
    global_deposits: list[GlobalDepositBalance] = field(default_factory=list)


@dataclass
class UserSnapshotOrder:
    order_hash: str
    side: int
    price: str
    size: str
    orderbook_id: str
    status: str = "open"
    order_type: str = "limit"
    trigger_order_id: Optional[str] = None


@dataclass
class UserOrdersResponse:
    orders: list[UserSnapshotOrder] = field(default_factory=list)
    balances: list[UserSnapshotBalance] = field(default_factory=list)
    next_cursor: Optional[str] = None


__all__ = [
    "OrderType",
    "OrderStatus",
    "FillInfo",
    "Order",
    "OrderEvent",
    "TriggerOrder",
    "TriggerOrderResponse",
    "SubmitOrderResponse",
    "CancelBody",
    "CancelSuccess",
    "CancelAllBody",
    "CancelAllSuccess",
    "CancelTriggerBody",
    "CancelTriggerSuccess",
    "ConditionalBalance",
    "GlobalDepositBalance",
    "UserSnapshotBalance",
    "UserSnapshotOrder",
    "UserOrdersResponse",
]
