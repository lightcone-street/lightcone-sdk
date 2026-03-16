"""Price history wire types."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class PriceCandle:
    t: int = 0
    m: Optional[str] = None
    o: Optional[str] = None
    h: Optional[str] = None
    l: Optional[str] = None
    c: Optional[str] = None
    v: Optional[str] = None
    bb: Optional[str] = None
    ba: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "PriceCandle":
        return PriceCandle(
            t=d.get("t", 0),
            m=d.get("m"),
            o=d.get("o"),
            h=d.get("h"),
            l=d.get("l"),
            c=d.get("c"),
            v=d.get("v"),
            bb=d.get("bb"),
            ba=d.get("ba"),
        )


@dataclass
class PriceHistorySnapshot:
    orderbook_id: str
    resolution: str
    candles: list[PriceCandle] = field(default_factory=list)
    last_timestamp: Optional[int] = None
    server_time: Optional[int] = None

    @staticmethod
    def from_dict(d: dict) -> "PriceHistorySnapshot":
        return PriceHistorySnapshot(
            orderbook_id=d.get("orderbook_id", ""),
            resolution=d.get("resolution", "1m"),
            candles=[PriceCandle.from_dict(c) for c in d.get("candles", d.get("prices", []))],
            last_timestamp=d.get("last_timestamp"),
            server_time=d.get("server_time"),
        )


@dataclass
class PriceHistoryUpdate:
    orderbook_id: str
    resolution: str
    candle: Optional[PriceCandle] = None

    @staticmethod
    def from_dict(d: dict) -> "PriceHistoryUpdate":
        candle_data = d.get("candle")
        return PriceHistoryUpdate(
            orderbook_id=d.get("orderbook_id", ""),
            resolution=d.get("resolution", "1m"),
            candle=PriceCandle.from_dict(candle_data) if candle_data else None,
        )


@dataclass
class PriceHistoryHeartbeat:
    server_time: int = 0
    last_processed: Optional[int] = None

    @staticmethod
    def from_dict(d: dict) -> "PriceHistoryHeartbeat":
        return PriceHistoryHeartbeat(
            server_time=d.get("server_time", 0),
            last_processed=d.get("last_processed"),
        )


@dataclass
class OrderbookPriceHistoryResponse:
    orderbook_id: str = ""
    resolution: str = "1m"
    include_ohlcv: bool = False
    prices: list[PriceCandle] = field(default_factory=list)
    next_cursor: Optional[int] = None
    has_more: bool = False
    decimals: dict[str, int] = field(default_factory=dict)

    @staticmethod
    def from_dict(d: dict) -> "OrderbookPriceHistoryResponse":
        return OrderbookPriceHistoryResponse(
            orderbook_id=d.get("orderbook_id", ""),
            resolution=d.get("resolution", "1m"),
            include_ohlcv=d.get("include_ohlcv", False),
            prices=[PriceCandle.from_dict(c) for c in d.get("prices", [])],
            next_cursor=d.get("next_cursor"),
            has_more=d.get("has_more", False),
            decimals=d.get("decimals") or {},
        )


@dataclass
class DepositTokenCandle:
    t: int = 0
    tc: int = 0
    c: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "DepositTokenCandle":
        return DepositTokenCandle(
            t=d.get("t", 0),
            tc=d.get("tc", 0),
            c=str(d.get("c", "0")),
        )


@dataclass
class DepositPriceSnapshot:
    deposit_asset: str = ""
    resolution: str = "1m"
    prices: list[DepositTokenCandle] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "DepositPriceSnapshot":
        return DepositPriceSnapshot(
            deposit_asset=d.get("deposit_asset", ""),
            resolution=d.get("resolution", "1m"),
            prices=[DepositTokenCandle.from_dict(c) for c in d.get("prices", [])],
        )


@dataclass
class DepositPriceTick:
    deposit_asset: str = ""
    price: str = "0"
    event_time: int = 0

    @staticmethod
    def from_dict(d: dict) -> "DepositPriceTick":
        return DepositPriceTick(
            deposit_asset=d.get("deposit_asset", ""),
            price=str(d.get("price", "0")),
            event_time=d.get("event_time", 0),
        )


@dataclass
class DepositPriceCandleUpdate:
    deposit_asset: str = ""
    resolution: str = "1m"
    t: int = 0
    tc: int = 0
    c: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "DepositPriceCandleUpdate":
        return DepositPriceCandleUpdate(
            deposit_asset=d.get("deposit_asset", ""),
            resolution=d.get("resolution", "1m"),
            t=d.get("t", 0),
            tc=d.get("tc", 0),
            c=str(d.get("c", "0")),
        )


@dataclass
class DepositPriceHistoryResponse:
    deposit_asset: str = ""
    binance_symbol: str = ""
    resolution: str = "1m"
    prices: list[DepositTokenCandle] = field(default_factory=list)
    next_cursor: Optional[int] = None
    has_more: bool = False

    @staticmethod
    def from_dict(d: dict) -> "DepositPriceHistoryResponse":
        return DepositPriceHistoryResponse(
            deposit_asset=d.get("deposit_asset", ""),
            binance_symbol=d.get("binance_symbol", ""),
            resolution=d.get("resolution", "1m"),
            prices=[DepositTokenCandle.from_dict(c) for c in d.get("prices", [])],
            next_cursor=d.get("next_cursor"),
            has_more=d.get("has_more", False),
        )


__all__ = [
    "PriceCandle",
    "PriceHistorySnapshot",
    "PriceHistoryUpdate",
    "PriceHistoryHeartbeat",
    "OrderbookPriceHistoryResponse",
    "DepositTokenCandle",
    "DepositPriceSnapshot",
    "DepositPriceTick",
    "DepositPriceCandleUpdate",
    "DepositPriceHistoryResponse",
]
