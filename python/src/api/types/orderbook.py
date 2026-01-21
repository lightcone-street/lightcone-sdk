"""Orderbook-related types for the Lightcone REST API."""

from dataclasses import dataclass
from typing import Optional


@dataclass
class PriceLevel:
    """Price level in the orderbook."""

    price: str
    size: str
    orders: int

    @classmethod
    def from_dict(cls, data: dict) -> "PriceLevel":
        return cls(
            price=data["price"],
            size=data["size"],
            orders=data["orders"],
        )


@dataclass
class OrderbookResponse:
    """Response for GET /api/orderbook/{orderbook_id}."""

    market_pubkey: str
    orderbook_id: str
    bids: list[PriceLevel]
    asks: list[PriceLevel]
    tick_size: str
    best_bid: Optional[str] = None
    best_ask: Optional[str] = None
    spread: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "OrderbookResponse":
        return cls(
            market_pubkey=data["market_pubkey"],
            orderbook_id=data["orderbook_id"],
            bids=[PriceLevel.from_dict(b) for b in data.get("bids", [])],
            asks=[PriceLevel.from_dict(a) for a in data.get("asks", [])],
            tick_size=data["tick_size"],
            best_bid=data.get("best_bid"),
            best_ask=data.get("best_ask"),
            spread=data.get("spread"),
        )
