"""Orderbook domain types."""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Optional, TYPE_CHECKING

if TYPE_CHECKING:
    from ..market import ConditionalToken


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

    def decimals(self) -> "OrderbookDecimals":
        """Derive scaling decimals from this pair's token metadata.

        This is the recommended way to get OrderbookDecimals — no REST call needed.
        """
        from ...shared.scaling import OrderbookDecimals
        base_decimals = self.base.decimals
        quote_decimals = self.quote.decimals
        return OrderbookDecimals(
            orderbook_id=self.orderbook_id,
            base_decimals=base_decimals,
            quote_decimals=quote_decimals,
            price_decimals=max(6 + quote_decimals - base_decimals, 0),
            tick_size=max(self.tick_size, 0),
        )

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
        pct = float((conditional - deposit) / deposit * 100)
        sign = "+" if pct > 0 else "-"
        return OutcomeImpact(
            sign=sign,
            is_positive=pct > 0,
            pct=abs(pct),
            dollar=str(abs(conditional - deposit)),
        )


@dataclass
class OutcomeImpact:
    """Price impact calculation result."""
    sign: str = ""
    pct: float = 0.0
    dollar: str = "0"
    is_positive: bool = False


class OrderBookValidationError(Exception):
    pass


__all__ = [
    "OrderBookPair",
    "OutcomeImpact",
    "OrderBookValidationError",
]
