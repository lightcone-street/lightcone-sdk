"""Orderbook ticker domain type."""

from dataclasses import dataclass
from typing import Optional


@dataclass
class TickerData:
    """Ticker data; ``mid_price`` is engine-authoritative and may use
    one-sided-book or last-trade fallback."""
    orderbook_id: str
    best_bid: Optional[str] = None
    best_ask: Optional[str] = None
    mid_price: Optional[str] = None


__all__ = ["TickerData"]
