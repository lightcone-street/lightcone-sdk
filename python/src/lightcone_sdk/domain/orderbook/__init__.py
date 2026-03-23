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

    def impact_pct(self, deposit_price: str) -> tuple[float, str]:
        """Price impact as percentage relative to a deposit asset price."""
        dp = Decimal(deposit_price)
        if dp == 0:
            return (0.0, "")
        if self.last_trade_price is not None:
            conditional = Decimal(self.last_trade_price)
            val = float((conditional - dp) / dp * 100)
            sign = "+" if val > 0 else ""
            return (val, sign)
        return (0.0, "")

    def impact(
        self,
        deposit_asset_price: str,
        conditional_price: str,
    ) -> OutcomeImpact:
        """Full impact calculation with sign, percentage, and dollar difference."""
        dap = Decimal(deposit_asset_price)
        cp = Decimal(conditional_price)
        if dap == 0:
            return OutcomeImpact(pct=0.0, dollar="0")
        pct = float((cp - dap) / dap * 100)
        sign = "+" if pct > 0 else "-"
        return OutcomeImpact(
            sign=sign,
            is_positive=pct > 0,
            pct=abs(pct),
            dollar=str(abs(cp - dap)),
        )


@dataclass
class OutcomeImpact:
    """Price impact calculation result."""
    sign: str = ""
    pct: float = 0.0
    dollar: str = "0"
    is_positive: bool = False


class OrderBookValidationError(Exception):
    """Base validation error for orderbook conversions."""
    pass


class BaseTokenNotFound(OrderBookValidationError):
    """Raised when the base token mint is not found in the conditional tokens list."""

    def __init__(self, message: str):
        super().__init__(f"Base token not found: {message}")


class QuoteTokenNotFound(OrderBookValidationError):
    """Raised when the quote token mint is not found in the conditional tokens list."""

    def __init__(self, message: str):
        super().__init__(f"Quote token not found: {message}")


__all__ = [
    "OrderBookPair",
    "OutcomeImpact",
    "OrderBookValidationError",
    "BaseTokenNotFound",
    "QuoteTokenNotFound",
]
