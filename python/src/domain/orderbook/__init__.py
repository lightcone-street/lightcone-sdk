"""Orderbook domain types."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class OrderBookPair:
    """Orderbook pair with metadata."""
    id: str
    market_pubkey: str
    orderbook_id: str
    base_token: str
    quote_token: str
    outcome_index: int
    tick_size: Optional[str] = None
    total_bids: int = 0
    total_asks: int = 0
    last_trade_price: Optional[str] = None
    last_trade_time: Optional[str] = None
    active: bool = True


@dataclass
class OutcomeImpact:
    """Price impact calculation result."""
    pct: float
    dollar: float
    sign: str = ""
    is_positive: bool = False


def impact(current_price: float, new_price: float) -> float:
    """Calculate absolute price impact."""
    return abs(new_price - current_price)


def impact_pct(current_price: float, new_price: float) -> float:
    """Calculate percentage price impact."""
    if current_price == 0:
        return 0.0
    return abs(new_price - current_price) / current_price * 100


class OrderBookValidationError(Exception):
    pass


__all__ = [
    "OrderBookPair",
    "OutcomeImpact",
    "OrderBookValidationError",
    "impact",
    "impact_pct",
]
