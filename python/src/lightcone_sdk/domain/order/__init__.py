"""Order domain types."""

from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum
from typing import Optional

from ...shared.types import TimeInForce, TriggerType


class OrderType(str, Enum):
    LIMIT = "limit"
    MARKET = "market"
    DEPOSIT = "deposit"
    MERGE = "merge"
    WITHDRAW = "withdraw"
    STOP_LIMIT = "StopLimit"
    TAKE_PROFIT_LIMIT = "TakeProfitLimit"


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

    def limit_price(self) -> Optional[Decimal]:
        """Derive the limit price from pre-scaled amounts.

        ``amount_in`` and ``amount_out`` are already human-readable decimals
        (scaled by the snapshot/websocket layer), so no further decimal
        conversion is needed.

        For Ask: maker gives base, receives quote -> price = quote / base
        For Bid: maker gives quote, receives base -> price = quote / base
        """
        amount_in = Decimal(self.amount_in)
        amount_out = Decimal(self.amount_out)

        if self.side == 1 and amount_in > 0:  # Ask
            return amount_out / amount_in
        elif self.side == 0 and amount_out > 0:  # Bid
            return amount_in / amount_out
        return None


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
    salt: str

    def to_dict(self) -> dict:
        return {
            "user_pubkey": self.user_pubkey,
            "orderbook_id": self.orderbook_id,
            "signature": self.signature,
            "timestamp": self.timestamp,
            "salt": self.salt,
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

    @staticmethod
    def from_dict(d: dict) -> "ConditionalBalance":
        return ConditionalBalance(
            outcome_index=d.get("outcome_index", 0),
            mint=d.get("mint", d.get("conditional_token", "")),
            idle=d.get("idle", "0"),
            on_book=d.get("on_book", "0"),
        )


@dataclass
class GlobalDepositBalance:
    mint: str = ""
    balance: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "GlobalDepositBalance":
        return GlobalDepositBalance(
            mint=d.get("mint", ""),
            balance=d.get("balance", "0"),
        )


@dataclass
class UserSnapshotBalance:
    market_pubkey: str = ""
    orderbook_id: str = ""
    outcomes: list[ConditionalBalance] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UserSnapshotBalance":
        return UserSnapshotBalance(
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            outcomes=[
                ConditionalBalance.from_dict(c)
                for c in d.get("outcomes", [])
            ],
        )


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

    @staticmethod
    def from_dict(d: dict) -> "UserSnapshotOrder":
        from ...shared.types import Side as _Side

        trigger_type_raw = d.get("trigger_type")
        time_in_force_raw = d.get("time_in_force")
        return UserSnapshotOrder(
            order_hash=d.get("order_hash", ""),
            side=int(_Side.from_wire(d.get("side", 0))),
            price=d.get("price", "0"),
            size=d.get("size", "0"),
            orderbook_id=d.get("orderbook_id", ""),
            market_pubkey=d.get("market_pubkey", ""),
            amount_in=d.get("amount_in", d.get("maker_amount", "0")),
            amount_out=d.get("amount_out", d.get("taker_amount", "0")),
            remaining=d.get("remaining", "0"),
            filled=d.get("filled", "0"),
            expiration=d.get("expiration", 0),
            base_mint=d.get("base_mint", ""),
            quote_mint=d.get("quote_mint", ""),
            outcome_index=d.get("outcome_index", 0),
            status=d.get("status", "open"),
            order_type=d.get("order_type", "limit"),
            created_at=d.get("created_at"),
            trigger_order_id=d.get("trigger_order_id"),
            trigger_price=d.get("trigger_price"),
            trigger_type=TriggerType.from_wire(trigger_type_raw) if trigger_type_raw is not None else None,
            time_in_force=TimeInForce.from_wire(time_in_force_raw) if time_in_force_raw is not None else None,
            tx_signature=d.get("tx_signature"),
        )


@dataclass
class UserOrdersResponse:
    user_pubkey: str = ""
    orders: list[UserSnapshotOrder] = field(default_factory=list)
    balances: list[UserSnapshotBalance] = field(default_factory=list)
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
