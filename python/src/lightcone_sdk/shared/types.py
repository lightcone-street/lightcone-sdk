"""Shared type definitions used across the Lightcone SDK."""

from dataclasses import dataclass
from decimal import Decimal, DecimalException
from enum import Enum, IntEnum
from typing import TYPE_CHECKING, NewType

from ..error import SdkError

if TYPE_CHECKING:
    from ..domain.market.tokens import ConditionalToken
    from ..domain.orderbook import OrderBookPair

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
        return "Buy" if self == Side.BID else "Sell"

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

    def spend_denominator(self) -> "Denominator":
        """The denomination of the asset this side spends (Bid spends quote,
        Ask spends base). Also a trade form's default display denomination."""
        return Denominator.QUOTE if self == Side.BID else Denominator.BASE

    def receive_denominator(self) -> "Denominator":
        """The denomination of the asset this side receives (Bid receives
        base, Ask receives quote)."""
        return Denominator.BASE if self == Side.BID else Denominator.QUOTE

    def apply_impact_protection(
        self, worst_fill_price: Decimal, protection_percent: Decimal
    ) -> Decimal | None:
        """The price to submit with a market (IOC) order: the worst book fill
        price padded by the impact-protection percentage in the direction that
        lets the order fill.

        Returns None unless the fill price and protection are finite and
        positive. Protection has no policy maximum; Ask protection at or above
        100% saturates at zero.
        """
        if (
            not worst_fill_price.is_finite()
            or not protection_percent.is_finite()
            or worst_fill_price <= 0
            or protection_percent <= 0
        ):
            return None
        try:
            factor = protection_percent / Decimal(100)
            if self == Side.BID:
                # buying: willing to pay more
                price = worst_fill_price * (Decimal(1) + factor)
            elif protection_percent >= 100:
                price = Decimal(0)
            else:
                # selling: willing to receive less
                price = worst_fill_price * (Decimal(1) - factor)
        except DecimalException:
            return None
        return price if price.is_finite() else None


class Denominator(str, Enum):
    """Order denomination: base or quote asset."""

    BASE = "Base"
    QUOTE = "Quote"

    @classmethod
    def all(cls) -> list["Denominator"]:
        return [cls.QUOTE, cls.BASE]

    def token(self, pair: "OrderBookPair") -> "ConditionalToken":
        """The conditional token this denomination refers to on ``pair``."""
        return pair.base if self == Denominator.BASE else pair.quote

    def symbol(self, pair: "OrderBookPair") -> str:
        return self.token(pair).symbol

    def deposit_symbol(self, pair: "OrderBookPair") -> str:
        return self.token(pair).deposit_symbol

    def convert_to(
        self,
        target: "Denominator",
        amount: Decimal,
        base_price_in_quote: Decimal,
    ) -> Decimal | None:
        """Convert ``amount`` from this denomination into ``target`` at the
        given price (quote per one base).

        Same-denomination conversion is the identity and never needs a price;
        crossing denominations requires a positive price — None otherwise.
        """
        if self == target:
            return amount
        if base_price_in_quote <= 0:
            return None
        if self == Denominator.BASE:
            return amount * base_price_in_quote
        return amount / base_price_in_quote


class TimeInForce(IntEnum):
    """Time-in-force policy for orders."""

    GTC = 0  # Good til cancelled
    IOC = 1  # Immediate or cancel
    FOK = 2  # Fill or kill
    ALO = 3  # Add liquidity only

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


class DepositSource(IntEnum):
    """Where collateral should be sourced when matching an order.

    Use None for the default behavior (auto: global if available, then market).
    """

    GLOBAL = 0
    MARKET = 1

    def as_str(self) -> str:
        return "global" if self == DepositSource.GLOBAL else "market"


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


# ---------------------------------------------------------------------------
# Request / response shapes
# ---------------------------------------------------------------------------


@dataclass
class SubmitOrderRequest:
    """Ordinary signed order request with exact integer amounts and execution policy."""

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
    time_in_force: TimeInForce | None = None
    deposit_source: DepositSource | None = None

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
        if self.deposit_source is not None:
            d["deposit_source"] = self.deposit_source.as_str()
        return d


__all__ = [
    "OrderBookId",
    "PubkeyStr",
    "Side",
    "Denominator",
    "TimeInForce",
    "OrderUpdateType",
    "DepositSource",
    "Resolution",
    "SubmitOrderRequest",
]
