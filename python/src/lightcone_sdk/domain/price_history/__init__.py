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


@dataclass
class DepositPriceKey:
    """Key for deposit-price lookups."""

    deposit_asset: str
    resolution: str


@dataclass
class LatestDepositPrice:
    """Latest live deposit-price tick."""

    price: str
    event_time: int


__all__ = [
    "LineData",
    "PriceHistoryKey",
    "DepositPriceKey",
    "LatestDepositPrice",
]
