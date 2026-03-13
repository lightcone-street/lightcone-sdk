"""Price history domain types."""

from dataclasses import dataclass
from typing import Optional


@dataclass
class LineData:
    """Single price point for charting."""
    time: int
    value: str


@dataclass
class PriceHistoryKey:
    """Key for price history lookups."""
    orderbook_id: str
    resolution: str


__all__ = ["LineData", "PriceHistoryKey"]
