"""Shared type definitions used across the Lightcone SDK."""

from dataclasses import dataclass
from enum import IntEnum
from typing import NewType, Optional


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

    STOP_LOSS = 0
    TAKE_PROFIT = 1

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


class TriggerStatus(IntEnum):
    """Trigger order execution status."""

    PENDING = 0
    TRIGGERED = 1
    CANCELLED = 2
    EXPIRED = 3
    FAILED = 4


class TriggerResultStatus(IntEnum):
    """Result status for a triggered order."""

    SUCCESS = 0
    FAILED = 1
    PARTIAL = 2


class OrderUpdateType(IntEnum):
    """Type of order update received via WebSocket."""

    NEW = 0
    FILL = 1
    CANCEL = 2
    EXPIRE = 3


class TriggerUpdateType(IntEnum):
    """Type of trigger order update received via WebSocket."""

    NEW = 0
    TRIGGERED = 1
    CANCELLED = 2
    EXPIRED = 3
    FAILED = 4


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
            raise ValueError(f"Invalid resolution: {s}")
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
    time_in_force: Optional[TimeInForce] = None
    trigger_price: Optional[float] = None
    trigger_type: Optional[TriggerType] = None
    deposit_source: Optional["DepositSource"] = None

    def to_dict(self) -> dict:
        d = {
            "maker": self.maker,
            "nonce": self.nonce,
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
    """Trigger order submission request."""

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

    def to_dict(self) -> dict:
        return {
            "maker": self.maker,
            "nonce": self.nonce,
            "market_pubkey": self.market_pubkey,
            "base_token": self.base_token,
            "quote_token": self.quote_token,
            "side": self.side,
            "amount_in": self.amount_in,
            "amount_out": self.amount_out,
            "expiration": self.expiration,
            "signature": self.signature,
            "orderbook_id": self.orderbook_id,
            "trigger_price": self.trigger_price,
            "trigger_type": self.trigger_type.as_wire(),
            "tif": self.time_in_force.as_wire(),
        }


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
