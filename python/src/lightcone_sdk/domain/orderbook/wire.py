"""Orderbook wire types."""

from dataclasses import dataclass, field
from typing import Optional

from ...error import _require


@dataclass
class PriceLevel:
    price: str
    size: str
    orders: Optional[int] = None

    @staticmethod
    def from_dict(d: dict) -> "PriceLevel":
        return PriceLevel(price=d.get("price", "0"), size=d.get("size", "0"), orders=d.get("orders"))

    @staticmethod
    def from_list(lst: list) -> "PriceLevel":
        return PriceLevel(price=str(lst[0]) if lst else "0", size=str(lst[1]) if len(lst) > 1 else "0")


@dataclass
class OrderbookDepthResponse:
    bids: list[PriceLevel] = field(default_factory=list)
    asks: list[PriceLevel] = field(default_factory=list)
    orderbook_id: Optional[str] = None
    market_pubkey: Optional[str] = None
    best_bid: Optional[str] = None
    best_ask: Optional[str] = None
    spread: Optional[str] = None
    tick_size: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderbookDepthResponse":
        return OrderbookDepthResponse(
            bids=[PriceLevel.from_dict(b) if isinstance(b, dict) else PriceLevel.from_list(b) for b in d.get("bids", [])],
            asks=[PriceLevel.from_dict(a) if isinstance(a, dict) else PriceLevel.from_list(a) for a in d.get("asks", [])],
            orderbook_id=d.get("orderbook_id"),
            market_pubkey=d.get("market_pubkey"),
            best_bid=d.get("best_bid"),
            best_ask=d.get("best_ask"),
            spread=d.get("spread"),
            tick_size=d.get("tick_size"),
        )


@dataclass
class OrderbookResponse:
    """Full REST orderbook response."""
    id: int = 0
    market_pubkey: str = ""
    orderbook_id: str = ""
    base_token: str = ""
    quote_token: str = ""
    outcome_index: int = 0
    tick_size: Optional[str] = None
    total_bids: int = 0
    total_asks: int = 0
    last_trade_price: Optional[str] = None
    last_trade_time: Optional[str] = None
    active: bool = True
    created_at: Optional[str] = None
    updated_at: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderbookResponse":
        return OrderbookResponse(
            id=d.get("id", 0),
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            base_token=d.get("base_token", ""),
            quote_token=d.get("quote_token", ""),
            outcome_index=d.get("outcome_index", 0),
            tick_size=d.get("tick_size"),
            total_bids=d.get("total_bids", 0),
            total_asks=d.get("total_asks", 0),
            last_trade_price=d.get("last_trade_price"),
            last_trade_time=d.get("last_trade_time"),
            active=d.get("active", True),
            created_at=d.get("created_at"),
            updated_at=d.get("updated_at"),
        )


@dataclass
class OrderbooksResponse:
    """Paginated orderbooks response."""
    orderbooks: list[OrderbookResponse] = field(default_factory=list)
    total: int = 0

    @staticmethod
    def from_dict(d: dict) -> "OrderbooksResponse":
        return OrderbooksResponse(
            orderbooks=[OrderbookResponse.from_dict(o) for o in d.get("orderbooks", [])],
            total=d.get("total", 0),
        )


@dataclass
class WsBookLevel:
    """WebSocket book level with side."""
    side: int
    price: str
    size: str

    @staticmethod
    def from_dict(d: dict) -> "WsBookLevel":
        return WsBookLevel(
            side=d.get("side", 0),
            price=str(d.get("price", "0")),
            size=str(d.get("size", "0")),
        )


@dataclass
class WsOrderBook:
    """WebSocket orderbook snapshot/delta."""
    orderbook_id: str
    is_snapshot: bool = False
    seq: int = 0
    resync: bool = False
    bids: list[WsBookLevel] = field(default_factory=list)
    asks: list[WsBookLevel] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "WsOrderBook":
        ob_id = d.get("orderbook_id") or d.get("id")
        if ob_id is None:
            from ...error import DeserializationError
            raise DeserializationError("Missing required field 'orderbook_id' in WsOrderBook")
        return WsOrderBook(
            orderbook_id=ob_id,
            is_snapshot=d.get("is_snapshot", False),
            seq=d.get("seq", 0),
            resync=d.get("resync", False),
            bids=[WsBookLevel.from_dict(b) for b in d.get("bids", [])],
            asks=[WsBookLevel.from_dict(a) for a in d.get("asks", [])],
        )


@dataclass
class DecimalsResponse:
    orderbook_id: str = ""
    base_decimals: int = 6
    quote_decimals: int = 6
    price_decimals: int = 6

    @staticmethod
    def from_dict(d: dict) -> "DecimalsResponse":
        return DecimalsResponse(
            orderbook_id=d.get("orderbook_id", ""),
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
            mid_price=d.get("mid_price") or d.get("mid"),
            last_trade_price=d.get("last_trade_price"),
        )
