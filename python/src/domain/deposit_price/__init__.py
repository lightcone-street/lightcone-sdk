"""Deposit-price websocket domain types."""

from dataclasses import dataclass


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


__all__ = ["DepositPriceKey", "LatestDepositPrice"]
