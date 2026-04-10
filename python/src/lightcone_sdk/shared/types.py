"""Shared type definitions used across the Lightcone SDK."""

from dataclasses import dataclass
from enum import Enum, IntEnum
from typing import NewType, Optional

from ..error import SdkError


# ---------------------------------------------------------------------------
# Branded types (NewType for type safety)
# ---------------------------------------------------------------------------

OrderBookId = NewType("OrderBookId", str)
PubkeyStr = NewType("PubkeyStr", str)


# ---------------------------------------------------------------------------
# Enums
# ---------------------------------------------------------------------------


class Side(IntEnum):
    """Order side."""

    BID = 0
    ASK = 1

    def label(self) -> str:
        return "Bid" if self == Side.BID else "Ask"

    def as_wire(self) -> str:
        return "bid" if self == Side.BID else "ask"

    @classmethod
    def from_wire(cls, value: "Side | int | str") -> "Side":
        if isinstance(value, cls):
            return value
        if isinstance(value, str):
            normalized = value.lower()
            if normalized in {"bid", "buy"}:
                return cls.BID
            if normalized in {"ask", "sell"}:
                return cls.ASK
        return cls(int(value))


class TimeInForce(IntEnum):
    """Time-in-force policy for orders."""

    GTC = 0   # Good til cancelled
    IOC = 1   # Immediate or cancel
    FOK = 2   # Fill or kill
    ALO = 3   # Add liquidity only

    def as_wire(self) -> str:
        return _TIME_IN_FORCE_TO_STR[self.value]

    @classmethod
    def from_wire(cls, value: "TimeInForce | int | str") -> "TimeInForce":
        if isinstance(value, cls):
            return value
        if isinstance(value, str):
            normalized = value.upper()
            if normalized in _STR_TO_TIME_IN_FORCE:
                return cls(_STR_TO_TIME_IN_FORCE[normalized])
        return cls(int(value))


class TriggerType(IntEnum):
    """Trigger order type."""

    TAKE_PROFIT = 0
    STOP_LOSS = 1

    def as_wire(self) -> str:
        return _TRIGGER_TYPE_TO_STR[self.value]

    @classmethod
    def from_wire(cls, value: "TriggerType | int | str") -> "TriggerType":
        if isinstance(value, cls):
            return value
        if isinstance(value, str):
            normalized = value.upper()
            if normalized in _STR_TO_TRIGGER_TYPE:
                return cls(_STR_TO_TRIGGER_TYPE[normalized])
        return cls(int(value))


class DepositSource(IntEnum):
    """Where collateral should be sourced when matching an order.

    Use None for the default behavior (auto: global if available, then market).
    """

    GLOBAL = 0
    MARKET = 1

    def as_str(self) -> str:
        return "global" if self == DepositSource.GLOBAL else "market"


class TriggerStatus(str, Enum):
    """Lifecycle status of a trigger order from WS updates."""

    CREATED = "created"
    TRIGGERED = "triggered"
    FAILED = "failed"
    EXPIRED = "expired"
    INVALIDATED = "invalidated"

    def as_wire(self) -> str:
        return self.value

    @classmethod
    def from_wire(cls, value: "TriggerStatus | str") -> "TriggerStatus":
        if isinstance(value, cls):
            return value
        return cls(str(value).lower())


class TriggerResultStatus(str, Enum):
    """Result status of a triggered order after matching."""

    FILLED = "filled"
    ACCEPTED = "accepted"
    REJECTED = "rejected"

    def as_wire(self) -> str:
        return self.value

    @classmethod
    def from_wire(cls, value: "TriggerResultStatus | str") -> "TriggerResultStatus":
        if isinstance(value, cls):
            return value
        return cls(str(value).lower())


class OrderUpdateType(str, Enum):
    """Rust-aligned limit-order WS update type."""

    PLACEMENT = "PLACEMENT"
    UPDATE = "UPDATE"
    CANCELLATION = "CANCELLATION"

    def as_wire(self) -> str:
        return self.value

    @classmethod
    def from_wire(cls, value: "OrderUpdateType | str") -> "OrderUpdateType":
        if isinstance(value, cls):
            return value
        return cls(str(value).upper())


class TriggerUpdateType(str, Enum):
    """Rust-aligned trigger-order WS update type."""

    CREATED = "CREATED"
    TRIGGERED = "TRIGGERED"
    FAILED = "FAILED"
    EXPIRED = "EXPIRED"
    INVALIDATED = "INVALIDATED"

    def as_wire(self) -> str:
        return self.value

    @classmethod
    def from_wire(cls, value: "TriggerUpdateType | str") -> "TriggerUpdateType":
        if isinstance(value, cls):
            return value
        return cls(str(value).upper())


class Resolution(IntEnum):
    """Price history candle resolution."""

    ONE_MINUTE = 0
    FIVE_MINUTES = 1
    FIFTEEN_MINUTES = 2
    ONE_HOUR = 3
    FOUR_HOURS = 4
    ONE_DAY = 5

    def as_str(self) -> str:
        """Get the string representation for API calls."""
        return _RESOLUTION_TO_STR[self.value]

    @classmethod
    def from_str(cls, s: str) -> "Resolution":
        """Parse a resolution string."""
        if s not in _STR_TO_RESOLUTION:
            raise SdkError(f"Invalid resolution: {s}")
        return cls(_STR_TO_RESOLUTION[s])

    def seconds(self) -> int:
        """Get the resolution in seconds."""
        return _RESOLUTION_SECONDS[self.value]

    def __str__(self) -> str:
        return self.as_str()


# Mappings defined outside enum to avoid IntEnum treating them as members
_RESOLUTION_TO_STR: dict[int, str] = {
    0: "1m",
    1: "5m",
    2: "15m",
    3: "1h",
    4: "4h",
    5: "1d",
}
_STR_TO_RESOLUTION: dict[str, int] = {v: k for k, v in _RESOLUTION_TO_STR.items()}
_RESOLUTION_SECONDS: dict[int, int] = {
    0: 60,
    1: 300,
    2: 900,
    3: 3600,
    4: 14400,
    5: 86400,
}
_TIME_IN_FORCE_TO_STR: dict[int, str] = {
    TimeInForce.GTC.value: "GTC",
    TimeInForce.IOC.value: "IOC",
    TimeInForce.FOK.value: "FOK",
    TimeInForce.ALO.value: "ALO",
}
_STR_TO_TIME_IN_FORCE: dict[str, int] = {
    value: key for key, value in _TIME_IN_FORCE_TO_STR.items()
}
_TRIGGER_TYPE_TO_STR: dict[int, str] = {
    TriggerType.STOP_LOSS.value: "SL",
    TriggerType.TAKE_PROFIT.value: "TP",
}
_STR_TO_TRIGGER_TYPE: dict[str, int] = {
    value: key for key, value in _TRIGGER_TYPE_TO_STR.items()
}


# ---------------------------------------------------------------------------
# Request / response shapes
# ---------------------------------------------------------------------------


@dataclass
class SubmitOrderRequest:
    """Order submission request."""

    maker: str
    nonce: int
    market_pubkey: str
    base_token: str
    quote_token: str
    side: int
    amount_in: int
    amount_out: int
    expiration: int
    signature: str
    orderbook_id: str
    salt: int = 0
    time_in_force: Optional[TimeInForce] = None
    trigger_price: Optional[float] = None
    trigger_type: Optional[TriggerType] = None
    deposit_source: Optional["DepositSource"] = None

    def to_dict(self) -> dict:
        d = {
            "maker": self.maker,
            "nonce": self.nonce,
            "salt": self.salt,
            "market_pubkey": self.market_pubkey,
            "base_token": self.base_token,
            "quote_token": self.quote_token,
            "side": self.side,
            "amount_in": self.amount_in,
            "amount_out": self.amount_out,
            "expiration": self.expiration,
            "signature": self.signature,
            "orderbook_id": self.orderbook_id,
        }
        if self.time_in_force is not None:
            d["tif"] = self.time_in_force.as_wire()
        if self.trigger_price is not None:
            d["trigger_price"] = self.trigger_price
        if self.trigger_type is not None:
            d["trigger_type"] = self.trigger_type.as_wire()
        if self.deposit_source is not None:
            d["deposit_source"] = self.deposit_source.as_str()
        return d


@dataclass
class SubmitTriggerOrderRequest:
    """Compatibility shim for trigger order submission.

    Rust models trigger and limit submissions with the same request shape. This
    helper still exists for callers that build trigger orders directly.
    """

    maker: str
    nonce: int
    market_pubkey: str
    base_token: str
    quote_token: str
    side: int
    amount_in: int
    amount_out: int
    expiration: int
    signature: str
    orderbook_id: str
    trigger_price: str
    trigger_type: TriggerType
    time_in_force: TimeInForce
    salt: int = 0
    deposit_source: Optional["DepositSource"] = None

    def to_submit_order_request(self) -> SubmitOrderRequest:
        return SubmitOrderRequest(
            maker=self.maker,
            nonce=self.nonce,
            market_pubkey=self.market_pubkey,
            base_token=self.base_token,
            quote_token=self.quote_token,
            side=self.side,
            amount_in=self.amount_in,
            amount_out=self.amount_out,
            expiration=self.expiration,
            signature=self.signature,
            orderbook_id=self.orderbook_id,
            salt=self.salt,
            time_in_force=self.time_in_force,
            trigger_price=float(self.trigger_price),
            trigger_type=self.trigger_type,
            deposit_source=self.deposit_source,
        )

    def to_dict(self) -> dict:
        return self.to_submit_order_request().to_dict()


__all__ = [
    "OrderBookId",
    "PubkeyStr",
    "Side",
    "TimeInForce",
    "TriggerType",
    "TriggerStatus",
    "TriggerResultStatus",
    "OrderUpdateType",
    "TriggerUpdateType",
    "DepositSource",
    "Resolution",
    "SubmitOrderRequest",
    "SubmitTriggerOrderRequest",
]
