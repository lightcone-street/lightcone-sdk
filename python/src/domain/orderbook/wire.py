"""Orderbook wire types."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class PriceLevel:
    price: str
    size: str

    @staticmethod
    def from_dict(d: dict) -> "PriceLevel":
        return PriceLevel(price=d.get("price", "0"), size=d.get("size", "0"))

    @staticmethod
    def from_list(lst: list) -> "PriceLevel":
        return PriceLevel(price=str(lst[0]) if lst else "0", size=str(lst[1]) if len(lst) > 1 else "0")


@dataclass
class OrderbookDepthResponse:
    bids: list[PriceLevel] = field(default_factory=list)
    asks: list[PriceLevel] = field(default_factory=list)
    orderbook_id: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderbookDepthResponse":
        return OrderbookDepthResponse(
            bids=[PriceLevel.from_dict(b) if isinstance(b, dict) else PriceLevel.from_list(b) for b in d.get("bids", [])],
            asks=[PriceLevel.from_dict(a) if isinstance(a, dict) else PriceLevel.from_list(a) for a in d.get("asks", [])],
            orderbook_id=d.get("orderbook_id"),
        )


@dataclass
class DecimalsResponse:
    base_decimals: int = 6
    quote_decimals: int = 6
    price_decimals: int = 6

    @staticmethod
    def from_dict(d: dict) -> "DecimalsResponse":
        return DecimalsResponse(
            base_decimals=d.get("base_decimals", 6),
            quote_decimals=d.get("quote_decimals", 6),
            price_decimals=d.get("price_decimals", 6),
        )


@dataclass
class WsTickerData:
    orderbook_id: str
    best_bid: Optional[str] = None
    best_ask: Optional[str] = None
    mid_price: Optional[str] = None
    last_trade_price: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "WsTickerData":
        return WsTickerData(
            orderbook_id=d.get("orderbook_id", ""),
            best_bid=d.get("best_bid"),
            best_ask=d.get("best_ask"),
            mid_price=d.get("mid_price"),
            last_trade_price=d.get("last_trade_price"),
        )
