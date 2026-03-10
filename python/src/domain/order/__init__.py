"""Order domain types."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

from ...shared.types import TimeInForce, TriggerType


class OrderType(str, Enum):
    LIMIT = "limit"
    MARKET = "market"
    DEPOSIT = "deposit"
    WITHDRAW = "withdraw"


class OrderStatus(str, Enum):
    OPEN = "OPEN"
    MATCHING = "MATCHING"
    CANCELLED = "CANCELLED"
    FILLED = "FILLED"
    PENDING = "PENDING"


@dataclass
class FillInfo:
    counterparty: str
    counterparty_order_hash: str
    fill_amount: str
    price: str
    is_maker: bool = False


@dataclass
class Order:
    """Order domain type."""
    market_pubkey: str
    orderbook_id: str
    order_hash: str
    side: int
    size: str
    price: str
    filled_size: str = "0"
    remaining_size: str = "0"
    created_at: Optional[str] = None
    status: OrderStatus = OrderStatus.OPEN
    outcome_index: int = 0
    tx_signature: Optional[str] = None
    base_mint: str = ""
    quote_mint: str = ""


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
    order_hash: str
    market_pubkey: str
    orderbook_id: str
    trigger_price: str
    trigger_type: TriggerType
    side: int
    amount_in: str
    amount_out: str
    time_in_force: TimeInForce
    created_at: Optional[str] = None


@dataclass
class TriggerOrderResponse:
    trigger_order_id: str
    order_hash: str


@dataclass
class SubmitOrderResponse:
    order_hash: str
    remaining: str = "0"
    filled: str = "0"
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
    remaining: int = 0


@dataclass
class CancelAllBody:
    user_pubkey: str
    orderbook_id: str
    signature: str
    timestamp: int

    def to_dict(self) -> dict:
        return {
            "user_pubkey": self.user_pubkey,
            "orderbook_id": self.orderbook_id,
            "signature": self.signature,
            "timestamp": self.timestamp,
        }


@dataclass
class CancelAllSuccess:
    cancelled_order_hashes: list[str] = field(default_factory=list)
    count: int = 0
    user_pubkey: str = ""
    orderbook_id: str = ""
    message: str = ""


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


@dataclass
class ConditionalBalance:
    outcome_index: int = 0
    mint: str = ""
    idle: str = "0"
    on_book: str = "0"


@dataclass
class GlobalDepositBalance:
    mint: str = ""
    balance: str = "0"


@dataclass
class UserSnapshotBalance:
    market_pubkey: str = ""
    orderbook_id: str = ""
    outcomes: list[ConditionalBalance] = field(default_factory=list)


@dataclass
class UserSnapshotOrder:
    order_hash: str = ""
    market_pubkey: str = ""
    orderbook_id: str = ""
    side: int = 0
    amount_in: str = "0"
    amount_out: str = "0"
    remaining: str = "0"
    filled: str = "0"
    price: str = "0"
    size: str = "0"
    created_at: Optional[str] = None
    expiration: int = 0
    base_mint: str = ""
    quote_mint: str = ""
    outcome_index: int = 0
    status: str = "OPEN"
    order_type: str = "limit"
    # Trigger-specific fields (present when order_type == "trigger")
    trigger_order_id: Optional[str] = None
    trigger_price: Optional[str] = None
    trigger_type: Optional[TriggerType] = None
    time_in_force: Optional[TimeInForce] = None
    # Limit-specific fields
    tx_signature: Optional[str] = None


@dataclass
class UserOrdersResponse:
    user_pubkey: str = ""
    orders: list[UserSnapshotOrder] = field(default_factory=list)
    balances: list[UserSnapshotBalance] = field(default_factory=list)
    global_deposits: list[GlobalDepositBalance] = field(default_factory=list)
    next_cursor: Optional[str] = None
    has_more: bool = False


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
