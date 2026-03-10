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


class TimeInForce(IntEnum):
    """Time-in-force policy for orders."""

    GTC = 0   # Good til cancelled
    IOC = 1   # Immediate or cancel
    FOK = 2   # Fill or kill
    ALO = 3   # Add liquidity only


class TriggerType(IntEnum):
    """Trigger order type."""

    STOP_LOSS = 0
    TAKE_PROFIT = 1


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
    time_in_force: Optional[int] = None
    trigger_price: Optional[float] = None
    trigger_type: Optional[int] = None
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
            d["time_in_force"] = self.time_in_force
        if self.trigger_price is not None:
            d["trigger_price"] = self.trigger_price
        if self.trigger_type is not None:
            d["trigger_type"] = self.trigger_type
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
    trigger_type: int
    time_in_force: int

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
            "trigger_type": self.trigger_type,
            "time_in_force": self.time_in_force,
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
    "Resolution",
    "SubmitOrderRequest",
    "SubmitTriggerOrderRequest",
]
