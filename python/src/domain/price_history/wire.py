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
