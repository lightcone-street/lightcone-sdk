"""Trade wire types."""

from dataclasses import dataclass
from typing import Optional

from ...error import _require


@dataclass
class TradeResponseWire:
    id: str
    orderbook_id: str
    taker_pubkey: Optional[str] = None
    maker_pubkey: Optional[str] = None
    side: int = 0
    size: str = "0"
    price: str = "0"
    taker_fee: Optional[str] = None
    maker_fee: Optional[str] = None
    fees: Optional[str] = None
    executed_at: Optional[str] = None
    market_pubkey: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "TradeResponseWire":
        return TradeResponseWire(
            id=_require(d, "id", "TradeResponseWire"),
            orderbook_id=_require(d, "orderbook_id", "TradeResponseWire"),
            taker_pubkey=d.get("taker_pubkey"),
            maker_pubkey=d.get("maker_pubkey"),
            side=d.get("side", 0),
            size=str(d.get("size", "0")),
            price=str(d.get("price", "0")),
            taker_fee=d.get("taker_fee"),
            maker_fee=d.get("maker_fee"),
            fees=d.get("fees"),
            executed_at=d.get("executed_at"),
            market_pubkey=d.get("market_pubkey"),
        )


@dataclass
class TradesResponseWire:
    trades: list[TradeResponseWire]
    next_cursor: Optional[str] = None
    has_more: bool = False

    @staticmethod
    def from_dict(d: dict) -> "TradesResponseWire":
        return TradesResponseWire(
            trades=[TradeResponseWire.from_dict(t) for t in d.get("trades", [])],
            next_cursor=d.get("next_cursor"),
            has_more=d.get("has_more", False),
        )


@dataclass
class WsTrade:
    orderbook_id: str
    price: str
    size: str
    side: int
    timestamp: str
    trade_id: str

    @staticmethod
    def from_dict(d: dict) -> "WsTrade":
        return WsTrade(
            orderbook_id=_require(d, "orderbook_id", "WsTrade"),
            price=str(d.get("price", "0")),
            size=str(d.get("size", "0")),
            side=d.get("side", 0),
            timestamp=d.get("timestamp", ""),
            trade_id=_require(d, "trade_id", "WsTrade"),
        )
