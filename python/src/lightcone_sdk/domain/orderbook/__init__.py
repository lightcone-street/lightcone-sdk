"""Orderbook domain types."""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from enum import Enum
from typing import Optional, TYPE_CHECKING

from .aggregation import BookAggregation, FULL_PRECISION

if TYPE_CHECKING:
    from ..market import ConditionalToken


class ImpactDirection(str, Enum):
    """Direction of a conditional token's price impact."""

    NEGATIVE = "negative"
    ZERO = "zero"
    POSITIVE = "positive"

    @property
    def sign(self) -> str:
        """Return the display sign for this direction."""
        if self == ImpactDirection.NEGATIVE:
            return "-"
        if self == ImpactDirection.POSITIVE:
            return "+"
        return ""


@dataclass
class OrderBookPair:
    """Orderbook pair with metadata."""
    id: int
    market_pubkey: str
    orderbook_id: str
    base: ConditionalToken
    quote: ConditionalToken
    outcome_index: int
    tick_size: int = 0
    total_bids: int = 0
    total_asks: int = 0
    last_trade_price: Optional[str] = None
    last_trade_time: Optional[str] = None
    active: bool = True

    @property
    def symbol(self) -> str:
        """Display symbol delegated to the base conditional token — enables
        `sort_by_display_priority` to treat orderbook pairs like tokens."""
        return self.base.symbol

    def market(self) -> "Pubkey":
        """Return the market as a ``Pubkey``."""
        from solders.pubkey import Pubkey
        return Pubkey.from_string(self.market_pubkey)

    def base_mint(self) -> "Pubkey":
        """Return the base conditional-token mint as a ``Pubkey``."""
        from solders.pubkey import Pubkey
        return Pubkey.from_string(self.base.pubkey)

    def quote_mint(self) -> "Pubkey":
        """Return the quote conditional-token mint as a ``Pubkey``."""
        from solders.pubkey import Pubkey
        return Pubkey.from_string(self.quote.pubkey)

    @staticmethod
    def impact_pct(deposit_price: str, conditional_price: str) -> tuple[float, str]:
        """Price impact as percentage relative to a deposit asset price."""
        deposit = Decimal(deposit_price)
        if deposit == 0:
            return (0.0, "")
        conditional = Decimal(conditional_price)
        if conditional == 0:
            return (0.0, "")
        val = float((conditional - deposit) / deposit * 100)
        sign = "+" if val > 0 else ""
        return (val, sign)

    @staticmethod
    def impact(
        deposit_asset_price: str,
        conditional_price: str,
    ) -> OutcomeImpact:
        """Full impact calculation with sign, percentage, and dollar difference."""
        deposit = Decimal(deposit_asset_price)
        conditional = Decimal(conditional_price)
        if deposit == 0:
            return OutcomeImpact(pct=0.0, dollar="0")
        dollar_delta = conditional - deposit
        pct = float(dollar_delta / deposit * 100)
        if dollar_delta > 0:
            direction = ImpactDirection.POSITIVE
        elif dollar_delta < 0:
            direction = ImpactDirection.NEGATIVE
        else:
            direction = ImpactDirection.ZERO
        return OutcomeImpact(
            direction=direction,
            pct=abs(pct),
            dollar=str(abs(dollar_delta)),
        )


@dataclass
class OutcomeImpact:
    """Price impact calculation result."""
    direction: ImpactDirection = ImpactDirection.ZERO
    pct: float = 0.0
    dollar: str = "0"

    @property
    def sign(self) -> str:
        """Return the display sign derived from the impact direction."""
        return self.direction.sign


class OrderBookValidationError(Exception):
    pass


__all__ = [
    "BookAggregation",
    "FULL_PRECISION",
    "ImpactDirection",
    "OrderBookPair",
    "OutcomeImpact",
    "OrderBookValidationError",
]
