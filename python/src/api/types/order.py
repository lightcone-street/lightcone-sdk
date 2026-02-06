"""Order-related types for the Lightcone REST API."""

from dataclasses import dataclass, field
from enum import IntEnum
from typing import Optional

from ..error import DeserializeError


class ApiOrderSide(IntEnum):
    """Order side enum."""

    BID = 0
    ASK = 1


class OrderStatus:
    """Order status enum."""

    ACCEPTED = "accepted"
    PARTIAL_FILL = "partial_fill"
    FILLED = "filled"
    REJECTED = "rejected"


@dataclass
class Fill:
    """Fill information from order matching."""

    counterparty: str
    counterparty_order_hash: str
    fill_amount: str
    price: str
    is_maker: bool

    @classmethod
    def from_dict(cls, data: dict) -> "Fill":
        try:
            return cls(
                counterparty=data["counterparty"],
                counterparty_order_hash=data["counterparty_order_hash"],
                fill_amount=data["fill_amount"],
                price=data["price"],
                is_maker=data["is_maker"],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in Fill: {e}")


@dataclass
class SubmitOrderRequest:
    """Request for POST /api/orders/submit."""

    maker: str
    nonce: int
    market_pubkey: str
    base_token: str
    quote_token: str
    side: int
    maker_amount: int
    taker_amount: int
    signature: str
    orderbook_id: str
    expiration: int = 0

    def to_dict(self) -> dict:
        return {
            "maker": self.maker,
            "nonce": self.nonce,
            "market_pubkey": self.market_pubkey,
            "base_token": self.base_token,
            "quote_token": self.quote_token,
            "side": self.side,
            "maker_amount": self.maker_amount,
            "taker_amount": self.taker_amount,
            "expiration": self.expiration,
            "signature": self.signature,
            "orderbook_id": self.orderbook_id,
        }


@dataclass
class OrderResponse:
    """Response for POST /api/orders/submit."""

    order_hash: str
    status: str
    remaining: str
    filled: str
    fills: list[Fill] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: dict) -> "OrderResponse":
        try:
            return cls(
                order_hash=data["order_hash"],
                status=data["status"],
                remaining=data["remaining"],
                filled=data["filled"],
                fills=[Fill.from_dict(f) for f in data.get("fills", [])],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in OrderResponse: {e}")


@dataclass
class CancelOrderRequest:
    """Request for POST /api/orders/cancel."""

    order_hash: str
    maker: str

    def to_dict(self) -> dict:
        return {
            "order_hash": self.order_hash,
            "maker": self.maker,
        }


@dataclass
class CancelResponse:
    """Response for POST /api/orders/cancel."""

    status: str
    order_hash: str
    remaining: str

    @classmethod
    def from_dict(cls, data: dict) -> "CancelResponse":
        try:
            return cls(
                status=data["status"],
                order_hash=data["order_hash"],
                remaining=data["remaining"],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in CancelResponse: {e}")


@dataclass
class CancelAllOrdersRequest:
    """Request for POST /api/orders/cancel-all."""

    user_pubkey: str
    market_pubkey: Optional[str] = None

    def to_dict(self) -> dict:
        d = {"user_pubkey": self.user_pubkey}
        if self.market_pubkey:
            d["market_pubkey"] = self.market_pubkey
        return d


@dataclass
class CancelAllResponse:
    """Response for POST /api/orders/cancel-all."""

    status: str
    user_pubkey: str
    cancelled_order_hashes: list[str]
    count: int
    message: str
    market_pubkey: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "CancelAllResponse":
        try:
            return cls(
                status=data["status"],
                user_pubkey=data["user_pubkey"],
                cancelled_order_hashes=data.get("cancelled_order_hashes", []),
                count=data["count"],
                message=data["message"],
                market_pubkey=data.get("market_pubkey"),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in CancelAllResponse: {e}")


@dataclass
class UserOrder:
    """User order from GET /api/users/orders."""

    order_hash: str
    market_pubkey: str
    orderbook_id: str
    side: int
    maker_amount: str
    taker_amount: str
    remaining: str
    filled: str
    price: str
    created_at: str
    expiration: int

    @classmethod
    def from_dict(cls, data: dict) -> "UserOrder":
        try:
            return cls(
                order_hash=data["order_hash"],
                market_pubkey=data["market_pubkey"],
                orderbook_id=data["orderbook_id"],
                side=data["side"],
                maker_amount=data["maker_amount"],
                taker_amount=data["taker_amount"],
                remaining=data["remaining"],
                filled=data["filled"],
                price=data["price"],
                created_at=data["created_at"],
                expiration=data["expiration"],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in UserOrder: {e}")


@dataclass
class UserOrderOutcomeBalance:
    """Outcome balance in user orders response."""

    outcome_index: int
    conditional_token: str
    idle: str
    on_book: str

    @classmethod
    def from_dict(cls, data: dict) -> "UserOrderOutcomeBalance":
        try:
            return cls(
                outcome_index=data["outcome_index"],
                conditional_token=data["conditional_token"],
                idle=data["idle"],
                on_book=data["on_book"],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in UserOrderOutcomeBalance: {e}")


@dataclass
class UserBalance:
    """User balance from GET /api/users/orders."""

    market_pubkey: str
    deposit_asset: str
    outcomes: list[UserOrderOutcomeBalance]

    @classmethod
    def from_dict(cls, data: dict) -> "UserBalance":
        try:
            return cls(
                market_pubkey=data["market_pubkey"],
                deposit_asset=data["deposit_asset"],
                outcomes=[
                    UserOrderOutcomeBalance.from_dict(o) for o in data.get("outcomes", [])
                ],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in UserBalance: {e}")


@dataclass
class GetUserOrdersRequest:
    """Request for POST /api/users/orders."""

    user_pubkey: str

    def to_dict(self) -> dict:
        return {"user_pubkey": self.user_pubkey}


@dataclass
class UserOrdersResponse:
    """Response for POST /api/users/orders."""

    user_pubkey: str
    orders: list[UserOrder]
    balances: list[UserBalance]
    next_cursor: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "UserOrdersResponse":
        try:
            return cls(
                user_pubkey=data["user_pubkey"],
                orders=[UserOrder.from_dict(o) for o in data.get("orders", [])],
                balances=[UserBalance.from_dict(b) for b in data.get("balances", [])],
                next_cursor=data.get("next_cursor"),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in UserOrdersResponse: {e}")
