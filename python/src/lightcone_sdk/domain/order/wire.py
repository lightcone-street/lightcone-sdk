"""Order wire types - raw API shapes."""

from dataclasses import dataclass, field
from decimal import Decimal, InvalidOperation
from typing import Optional, Union

from ...error import _require
from ...shared.types import Side
from . import UserSnapshotOrder, UserSnapshotBalance, GlobalDepositBalance, ConditionalBalance
from ..notification import Notification


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
    balance: Optional[UserOrderUpdateBalance] = None

    @staticmethod
    def from_dict(d: dict) -> "WsOrder":
        remaining = str(d.get("remaining", d.get("remaining_size", "0")))
        filled = str(d.get("filled", d.get("filled_size", "0")))
        size = d.get("size")
        if size is None:
            size = _sum_decimal_strings(remaining, filled)
        bal_raw = d.get("balance")
        balance = UserOrderUpdateBalance.from_dict(bal_raw) if isinstance(bal_raw, dict) else None
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
    timestamp: Optional[str] = None
    tx_signature: Optional[str] = None
    update_type: Optional[str] = None
    order: Optional[WsOrder] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderUpdate":
        order_data = d.get("order")
        return OrderUpdate(
            market_pubkey=_require(d, "market_pubkey", "OrderUpdate"),
            orderbook_id=_require(d, "orderbook_id", "OrderUpdate"),
            timestamp=d.get("timestamp"),
            tx_signature=d.get("tx_signature"),
            update_type=d.get("type", d.get("update_type")),
            order=WsOrder.from_dict(order_data) if order_data else None,
        )


@dataclass
class TriggerOrderUpdate:
    trigger_order_id: str
    status: str
    user_pubkey: str = ""
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
    side: int = 0

    @staticmethod
    def from_dict(d: dict) -> "TriggerOrderUpdate":
        return TriggerOrderUpdate(
            trigger_order_id=_require(d, "trigger_order_id", "TriggerOrderUpdate"),
            status=_require(d, "status", "TriggerOrderUpdate"),
            user_pubkey=d.get("user_pubkey", ""),
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            order_hash=d.get("order_hash", ""),
            trigger_price=str(d.get("trigger_price")) if d.get("trigger_price") is not None else None,
            trigger_above=d.get("trigger_above"),
            update_type=d.get("type", d.get("update_type")),
            result_status=d.get("result_status"),
            result_filled=str(d.get("result_filled", "0")) if d.get("result_filled") is not None else None,
            result_remaining=str(d.get("result_remaining", "0")) if d.get("result_remaining") is not None else None,
            timestamp=d.get("timestamp"),
            side=int(Side.from_wire(d.get("side", 0))),
        )


@dataclass
class UserBalanceUpdate:
    """WebSocket user balance update."""
    market_pubkey: str = ""
    orderbook_id: str = ""
    balance: Optional[UserSnapshotBalance] = None
    timestamp: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "UserBalanceUpdate":
        bal_raw = d.get("balance")
        bal = None
        if isinstance(bal_raw, dict):
            bal = UserSnapshotBalance.from_dict(bal_raw)
        return UserBalanceUpdate(
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            balance=bal,
            timestamp=d.get("timestamp"),
        )


@dataclass
class NotificationUpdate:
    """WebSocket notification push."""
    notification: Optional[Notification] = None

    @staticmethod
    def from_dict(d: dict) -> "NotificationUpdate":
        notif_raw = d.get("notification")
        notif = Notification.from_dict(notif_raw) if isinstance(notif_raw, dict) else None
        return NotificationUpdate(notification=notif)


@dataclass
class GlobalDepositUpdate:
    """WS global deposit update event."""
    mint: str = ""
    balance: str = "0"
    timestamp: Optional[str] = None

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
    timestamp: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "NonceUpdate":
        return NonceUpdate(
            user_pubkey=d.get("user_pubkey", ""),
            new_nonce=d.get("new_nonce", 0),
            timestamp=d.get("timestamp"),
        )


@dataclass
class UserSnapshot:
    """WebSocket user snapshot."""
    orders: list[UserSnapshotOrder] = field(default_factory=list)
    balances: list[UserSnapshotBalance] = field(default_factory=list)
    global_deposits: list[GlobalDepositBalance] = field(default_factory=list)
    notifications: list[Notification] = field(default_factory=list)
    nonce: int = 0

    @staticmethod
    def from_dict(d: dict) -> "UserSnapshot":
        balances_raw = d.get("balances", [])
        if isinstance(balances_raw, dict):
            balances_raw = list(balances_raw.values())
        return UserSnapshot(
            orders=[UserSnapshotOrder.from_dict(o) for o in d.get("orders", [])],
            balances=[UserSnapshotBalance.from_dict(b) for b in balances_raw],
            global_deposits=[GlobalDepositBalance.from_dict(g) for g in d.get("global_deposits", [])],
            notifications=[Notification.from_dict(n) for n in d.get("notifications", [])],
            nonce=d.get("nonce", 0),
        )


UserUpdateData = Union[
    "UserSnapshot", "OrderUpdate", "TriggerOrderUpdate",
    "UserBalanceUpdate", "GlobalDepositUpdate", "NonceUpdate",
    "NotificationUpdate", dict,
]


@dataclass
class UserUpdate:
    event_type: str = ""
    data: Optional[UserUpdateData] = None

    @staticmethod
    def from_dict(d: dict) -> "UserUpdate":
        event_type = d.get("event_type", "")
        if event_type == "snapshot":
            payload = UserSnapshot.from_dict(d)
        elif event_type == "order":
            payload = _parse_order_event(d)
        elif event_type == "balance_update":
            payload = UserBalanceUpdate.from_dict(d)
        elif event_type == "global_deposit_update":
            payload = GlobalDepositUpdate.from_dict(d)
        elif event_type == "nonce":
            payload = NonceUpdate.from_dict(d)
        elif event_type == "notification":
            payload = NotificationUpdate.from_dict(d)
        else:
            payload = d

        return UserUpdate(event_type=event_type, data=payload)


@dataclass
class AuthUpdate:
    status: str = "anonymous"
    authenticated: bool = False
    wallet_address: Optional[str] = None
    code: Optional[str] = None
    message: Optional[str] = None

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
            code=d.get("code"),
            message=d.get("message"),
        )


def _parse_order_event(d: dict) -> Union[OrderUpdate, TriggerOrderUpdate]:
    if d.get("order_type") == "trigger":
        return TriggerOrderUpdate.from_dict(d)
    return OrderUpdate.from_dict(d)


def _sum_decimal_strings(left: str, right: str) -> str:
    try:
        return format(Decimal(left) + Decimal(right), "f")
    except (InvalidOperation, ValueError):
        return "0"
