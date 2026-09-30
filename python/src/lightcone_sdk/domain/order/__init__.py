"""Order domain types."""

from dataclasses import dataclass, field
from decimal import Decimal, InvalidOperation
from enum import Enum

from ...error import DeserializationError, _require


class OrderType(str, Enum):
    """Supported resting-order kind."""

    LIMIT = "limit"

    def label(self) -> str:
        return {
            OrderType.LIMIT: "Limit",
        }[self]


class OrderStatus(str, Enum):
    OPEN = "OPEN"
    MATCHING = "MATCHING"
    CANCELLED = "CANCELLED"
    FILLED = "FILLED"
    PENDING = "PENDING"


class SubmitOrderStatus(str, Enum):
    ACCEPTED = "accepted"
    PARTIAL_FILL = "partial_fill"
    FILLED = "filled"


@dataclass
class FillInfo:
    counterparty: str
    counterparty_order_hash: str
    fill_amount: str
    price: str
    is_maker: bool = False


@dataclass
class LimitOrder:
    """Limit order domain type."""

    market_pubkey: str
    orderbook_id: str
    order_hash: str
    side: int
    size: str
    price: str
    filled_size: str = "0"
    remaining_size: str = "0"
    created_at: str | None = None
    status: OrderStatus = OrderStatus.OPEN
    outcome_index: int = 0
    tx_signature: str | None = None
    base_mint: str = ""
    quote_mint: str = ""


@dataclass
class OrderEvent:
    """WebSocket order event."""

    type: str
    order: LimitOrder | None = None
    fill: FillInfo | None = None


@dataclass
class SubmitOrderResponse:
    order_hash: str
    status: SubmitOrderStatus = SubmitOrderStatus.ACCEPTED
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
    remaining: str = "0"


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
class ConditionalBalance:
    outcome_index: int = 0
    conditional_token: str = ""
    idle: str = "0"
    on_book: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "ConditionalBalance":
        return ConditionalBalance(
            outcome_index=d.get("outcome_index", 0),
            conditional_token=_require(d, "conditional_token", "ConditionalBalance"),
            idle=str(d.get("idle", "0")),
            on_book=str(d.get("on_book", "0")),
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
class UserOutcomeBalance:
    outcome_index: int = 0
    conditional_token: str = ""
    balance: str = "0"
    balance_idle: str = "0"
    balance_on_book: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "UserOutcomeBalance":
        return UserOutcomeBalance(
            outcome_index=d.get("outcome_index", 0),
            conditional_token=_require(d, "conditional_token", "UserOutcomeBalance"),
            balance=str(_require(d, "balance", "UserOutcomeBalance")),
            balance_idle=str(_require(d, "balance_idle", "UserOutcomeBalance")),
            balance_on_book=str(_require(d, "balance_on_book", "UserOutcomeBalance")),
        )

    def is_zero(self) -> bool:
        """True when the outcome holds nothing idle and nothing resting on the book."""
        return not (
            Decimal(self.balance_idle) > Decimal(0)
            or Decimal(self.balance_on_book) > Decimal(0)
        )


@dataclass
class UserDepositAssetBalance:
    deposit_asset: str = ""
    outcomes: list[UserOutcomeBalance] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UserDepositAssetBalance":
        return UserDepositAssetBalance(
            deposit_asset=_require(d, "deposit_asset", "UserDepositAssetBalance"),
            outcomes=[
                UserOutcomeBalance.from_dict(c)
                for c in _require(d, "outcomes", "UserDepositAssetBalance")
            ],
        )


@dataclass
class UserMarketBalance:
    market_pubkey: str = ""
    deposit_assets: list[UserDepositAssetBalance] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UserMarketBalance":
        return UserMarketBalance(
            market_pubkey=_require(d, "market_pubkey", "UserMarketBalance"),
            deposit_assets=[
                UserDepositAssetBalance.from_dict(c)
                for c in _require(d, "deposit_assets", "UserMarketBalance")
            ],
        )


@dataclass
class UserSnapshotOrder:
    """Limit order from a REST response or WebSocket account snapshot."""

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
    created_at: str | None = None
    expiration: int = 0
    base_mint: str = ""
    quote_mint: str = ""
    outcome_index: int = 0
    status: str = OrderStatus.OPEN.value
    order_type: str = OrderType.LIMIT.value
    tx_signature: str | None = None

    @staticmethod
    def from_dict(d: dict) -> "UserSnapshotOrder":
        """Decode a limit order with its required wire kind, amounts, and order hash."""
        from ...shared.types import Side as _Side

        if not isinstance(d, dict):
            raise DeserializationError("UserSnapshotOrder requires an order object")
        if d.get("order_type") != OrderType.LIMIT.value:
            raise DeserializationError("UserSnapshotOrder requires order_type 'limit'")
        order_hash = _require(d, "order_hash", "UserSnapshotOrder")
        if not isinstance(order_hash, str):
            raise DeserializationError("UserSnapshotOrder order_hash must be a string")
        amount_in = d.get("amount_in", d.get("maker_amount"))
        amount_out = d.get("amount_out", d.get("taker_amount"))
        if amount_in is None or amount_out is None:
            raise DeserializationError(
                "UserSnapshotOrder requires amount_in and amount_out"
            )
        remaining = str(d.get("remaining", "0"))
        filled = str(d.get("filled", "0"))
        size = d.get("size")
        if size is None:
            size = _sum_decimal_strings(remaining, filled)
        return UserSnapshotOrder(
            order_hash=order_hash,
            side=int(_Side.from_wire(d.get("side", 0))),
            price=d.get("price", "0"),
            size=str(size),
            orderbook_id=d.get("orderbook_id", ""),
            market_pubkey=d.get("market_pubkey", ""),
            amount_in=amount_in,
            amount_out=amount_out,
            remaining=remaining,
            filled=filled,
            expiration=d.get("expiration", 0),
            base_mint=d.get("base_mint", ""),
            quote_mint=d.get("quote_mint", ""),
            outcome_index=d.get("outcome_index", 0),
            status=d.get("status", OrderStatus.OPEN.value),
            order_type=OrderType.LIMIT.value,
            created_at=d.get("created_at"),
            tx_signature=d.get("tx_signature"),
        )


def _sum_decimal_strings(left: str, right: str) -> str:
    try:
        return format(Decimal(left) + Decimal(right), "f")
    except (InvalidOperation, ValueError):
        return "0"


@dataclass
class UserOrdersResponse:
    user_pubkey: str = ""
    orders: list[UserSnapshotOrder] = field(default_factory=list)
    market_balances: list[UserMarketBalance] = field(default_factory=list)
    next_cursor: str | None = None
    has_more: bool = False


from .wire import (  # noqa: E402 — re-export wire types matching Rust mod.rs
    FillStatus,
    OrderFillEvent,
    Role,
    UserOrderFill,
    UserOrderFillsResponse,
)

__all__ = [
    "OrderType",
    "OrderStatus",
    "FillInfo",
    "LimitOrder",
    "OrderEvent",
    "SubmitOrderResponse",
    "SubmitOrderStatus",
    "CancelBody",
    "CancelSuccess",
    "CancelAllBody",
    "CancelAllSuccess",
    "ConditionalBalance",
    "GlobalDepositBalance",
    "UserMarketBalance",
    "UserDepositAssetBalance",
    "UserOutcomeBalance",
    "UserSnapshotOrder",
    "UserOrdersResponse",
    "Role",
    "FillStatus",
    "OrderFillEvent",
    "UserOrderFill",
    "UserOrderFillsResponse",
]
