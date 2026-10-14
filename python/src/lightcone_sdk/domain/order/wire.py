"""Order wire types - raw API shapes."""

from dataclasses import dataclass, field
from decimal import Decimal, InvalidOperation
from enum import Enum
from typing import Union

from ...error import DeserializationError, _require
from ...shared.types import Side
from ..notification import Notification
from . import (
    ConditionalBalance,
    GlobalDepositBalance,
    UserMarketBalance,
    UserSnapshotOrder,
)


@dataclass
class UserOrderUpdateBalance:
    """Balance update included with order events."""

    outcomes: list[ConditionalBalance] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UserOrderUpdateBalance":
        return UserOrderUpdateBalance(
            outcomes=[ConditionalBalance.from_dict(o) for o in d.get("outcomes", [])],
        )


@dataclass
class WsOrder:
    """WebSocket order update."""

    order_hash: str
    side: int
    price: str
    size: str
    filled_size: str | None = None
    remaining_size: str | None = None
    status: str | None = None
    is_maker: bool = False
    remaining: str | None = None
    filled: str | None = None
    fill_amount: str | None = None
    base_mint: str = ""
    quote_mint: str = ""
    outcome_index: int = 0
    created_at: str | None = None
    balance: UserOrderUpdateBalance | None = None

    @staticmethod
    def from_dict(d: dict) -> "WsOrder":
        remaining = str(d.get("remaining", d.get("remaining_size", "0")))
        filled = str(d.get("filled", d.get("filled_size", "0")))
        size = d.get("size")
        if size is None:
            size = _sum_decimal_strings(remaining, filled)
        bal_raw = d.get("balance")
        balance = (
            UserOrderUpdateBalance.from_dict(bal_raw)
            if isinstance(bal_raw, dict)
            else None
        )
        return WsOrder(
            order_hash=_require(d, "order_hash", "WsOrder"),
            side=int(Side.from_wire(d.get("side", 0))),
            price=str(d.get("price", "0")),
            size=str(size),
            filled_size=filled,
            remaining_size=remaining,
            status=d.get("status"),
            is_maker=d.get("is_maker", False),
            remaining=remaining,
            filled=filled,
            fill_amount=str(d.get("fill_amount", "0")),
            base_mint=d.get("base_mint", ""),
            quote_mint=d.get("quote_mint", ""),
            outcome_index=d.get("outcome_index", 0),
            created_at=d.get("created_at"),
            balance=balance,
        )


@dataclass
class OrderUpdate:
    """WebSocket order update wrapper."""

    market_pubkey: str
    orderbook_id: str
    timestamp: str | None = None
    tx_signature: str | None = None
    update_type: str | None = None
    order: WsOrder | None = None

    @staticmethod
    def from_dict(d: dict) -> "OrderUpdate":
        """Decode a live limit order, requiring its order payload."""
        order_data = _require(d, "order", "OrderUpdate")
        if not isinstance(order_data, dict):
            raise DeserializationError("OrderUpdate requires an order object")
        return OrderUpdate(
            market_pubkey=_require(d, "market_pubkey", "OrderUpdate"),
            orderbook_id=_require(d, "orderbook_id", "OrderUpdate"),
            timestamp=d.get("timestamp"),
            tx_signature=d.get("tx_signature"),
            update_type=d.get("type", d.get("update_type")),
            order=WsOrder.from_dict(order_data),
        )


@dataclass
class UserBalanceUpdate:
    """WebSocket user balance update."""

    market_pubkey: str = ""
    market_balance: UserMarketBalance = field(default_factory=UserMarketBalance)
    timestamp: str | None = None

    @staticmethod
    def from_dict(d: dict) -> "UserBalanceUpdate":
        return UserBalanceUpdate(
            market_pubkey=_require(d, "market_pubkey", "UserBalanceUpdate"),
            market_balance=UserMarketBalance.from_dict(
                _require(d, "market_balance", "UserBalanceUpdate")
            ),
            timestamp=d.get("timestamp"),
        )


@dataclass
class NotificationUpdate:
    """WebSocket notification push."""

    notification: Notification | None = None

    @staticmethod
    def from_dict(d: dict) -> "NotificationUpdate":
        notif_raw = d.get("notification")
        notif = (
            Notification.from_dict(notif_raw) if isinstance(notif_raw, dict) else None
        )
        return NotificationUpdate(notification=notif)


@dataclass
class GlobalDepositUpdate:
    """WS global deposit update event."""

    mint: str = ""
    balance: str = "0"
    timestamp: str | None = None

    @staticmethod
    def from_dict(d: dict) -> "GlobalDepositUpdate":
        return GlobalDepositUpdate(
            mint=d.get("mint", ""),
            balance=d.get("balance", "0"),
            timestamp=d.get("timestamp"),
        )


@dataclass
class NonceUpdate:
    """WS nonce update event."""

    user_pubkey: str = ""
    new_nonce: int = 0
    timestamp: str | None = None

    @staticmethod
    def from_dict(d: dict) -> "NonceUpdate":
        return NonceUpdate(
            user_pubkey=d.get("user_pubkey", ""),
            new_nonce=d.get("new_nonce", 0),
            timestamp=d.get("timestamp"),
        )


@dataclass
class UserSnapshot:
    """Account snapshot containing limit orders and the accompanying account data."""

    orders: list[UserSnapshotOrder] = field(default_factory=list)
    market_balances: list[UserMarketBalance] = field(default_factory=list)
    global_deposits: list[GlobalDepositBalance] = field(default_factory=list)
    notifications: list[Notification] = field(default_factory=list)
    nonce: int = 0

    @staticmethod
    def from_dict(d: dict) -> "UserSnapshot":
        orders = d.get("orders", [])
        if not isinstance(orders, list):
            raise DeserializationError("UserSnapshot orders must be an array")
        return UserSnapshot(
            orders=[UserSnapshotOrder.from_dict(o) for o in orders],
            market_balances=[
                UserMarketBalance.from_dict(b)
                for b in _require(d, "market_balances", "UserSnapshot")
            ],
            global_deposits=[
                GlobalDepositBalance.from_dict(g) for g in d.get("global_deposits", [])
            ],
            notifications=[
                Notification.from_dict(n) for n in d.get("notifications", [])
            ],
            nonce=d.get("nonce", 0),
        )


UserUpdateData = Union[
    "UserSnapshot",
    "OrderUpdate",
    "UserBalanceUpdate",
    "GlobalDepositUpdate",
    "NonceUpdate",
    "NotificationUpdate",
    dict,
]


@dataclass
class UserUpdate:
    event_type: str = ""
    data: UserUpdateData | None = None

    @staticmethod
    def from_dict(d: dict) -> "UserUpdate":
        event_type = d.get("event_type", "")
        if event_type == "snapshot":
            payload = UserSnapshot.from_dict(d)
        elif event_type == "order":
            if d.get("order_type") != "limit":
                raise DeserializationError("Order event requires order_type 'limit'")
            payload = OrderUpdate.from_dict(d)
        elif event_type == "market_balance_update":
            payload = UserBalanceUpdate.from_dict(d)
        elif event_type == "global_deposit_update":
            payload = GlobalDepositUpdate.from_dict(d)
        elif event_type == "nonce":
            payload = NonceUpdate.from_dict(d)
        elif event_type == "notification":
            payload = NotificationUpdate.from_dict(d)
        else:
            raise DeserializationError(f"Invalid user update event_type '{event_type}'")

        return UserUpdate(event_type=event_type, data=payload)


@dataclass
class AuthUpdate:
    status: str = "anonymous"
    authenticated: bool = False
    wallet_address: str | None = None
    reason: str | None = None

    @staticmethod
    def from_dict(d: dict) -> "AuthUpdate":
        status = d.get("status")
        if status is None:
            authenticated = d.get("authenticated", False)
            status = "authenticated" if authenticated else "anonymous"
        else:
            authenticated = status == "authenticated"

        return AuthUpdate(
            status=status,
            authenticated=authenticated,
            wallet_address=d.get("wallet", d.get("wallet_address")),
            reason=d.get("reason"),
        )


class Role(str, Enum):
    """Whether the user was the maker or taker on an order."""

    MAKER = "maker"
    TAKER = "taker"


class FillStatus(str, Enum):
    """Status of a filled order, derived from DB state after the fact."""

    FILLED = "filled"
    CANCELLED = "cancelled"
    PARTIALLY_FILLED = "partially_filled"


@dataclass
class OrderFillEvent:
    """A single fill event within an order."""

    fill_amount: str = "0"
    tx_signature: str = ""
    filled_at: str = ""

    @staticmethod
    def from_dict(d: dict) -> "OrderFillEvent":
        return OrderFillEvent(
            fill_amount=str(d.get("fill_amount", "0")),
            tx_signature=d.get("tx_signature", ""),
            filled_at=str(d.get("filled_at", "")),
        )


@dataclass
class UserOrderFill:
    """An order the user participated in, with nested fill events."""

    order_hash: str = ""
    market_pubkey: str = ""
    orderbook_id: str = ""
    side: int = 0
    role: str = ""
    price: str = "0"
    size: str = "0"
    filled_size: str = "0"
    remaining_size: str = "0"
    base_mint: str = ""
    quote_mint: str = ""
    outcome_index: int = 0
    status: str = ""
    created_at: str = ""
    fills: list[OrderFillEvent] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UserOrderFill":
        from ...shared.types import Side as _Side

        return UserOrderFill(
            order_hash=d.get("order_hash", ""),
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            side=int(_Side.from_wire(d.get("side", 0))),
            role=d.get("role", ""),
            price=str(d.get("price", "0")),
            size=str(d.get("size", "0")),
            filled_size=str(d.get("filled_size", "0")),
            remaining_size=str(d.get("remaining_size", "0")),
            base_mint=d.get("base_mint", ""),
            quote_mint=d.get("quote_mint", ""),
            outcome_index=d.get("outcome_index", 0),
            status=d.get("status", ""),
            created_at=str(d.get("created_at", "")),
            fills=[OrderFillEvent.from_dict(f) for f in d.get("fills", [])],
        )


@dataclass
class UserOrderFillsResponse:
    """Response from GET /api/users/order-fills."""

    orders: list[UserOrderFill] = field(default_factory=list)
    next_cursor: str | None = None
    has_more: bool = False

    @staticmethod
    def from_dict(d: dict) -> "UserOrderFillsResponse":
        return UserOrderFillsResponse(
            orders=[UserOrderFill.from_dict(o) for o in d.get("orders", [])],
            next_cursor=d.get("next_cursor"),
            has_more=d.get("has_more", False),
        )


def _sum_decimal_strings(left: str, right: str) -> str:
    try:
        return format(Decimal(left) + Decimal(right), "f")
    except (InvalidOperation, ValueError):
        return "0"
