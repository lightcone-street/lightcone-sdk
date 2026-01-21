"""Price history types for the Lightcone REST API."""

from dataclasses import dataclass
from typing import Optional, Union

from ...shared.types import Resolution


@dataclass
class PricePoint:
    """Price point data."""

    timestamp: int  # t
    midpoint: str  # m
    open: Optional[str] = None  # o
    high: Optional[str] = None  # h
    low: Optional[str] = None  # l
    close: Optional[str] = None  # c
    volume: Optional[str] = None  # v
    best_bid: Optional[str] = None  # bb
    best_ask: Optional[str] = None  # ba

    @classmethod
    def from_dict(cls, data: dict) -> "PricePoint":
        return cls(
            timestamp=data["t"],
            midpoint=data["m"],
            open=data.get("o"),
            high=data.get("h"),
            low=data.get("l"),
            close=data.get("c"),
            volume=data.get("v"),
            best_bid=data.get("bb"),
            best_ask=data.get("ba"),
        )


@dataclass
class PriceHistoryParams:
    """Query parameters for GET /api/price-history."""

    orderbook_id: str
    resolution: Optional[Union[Resolution, str]] = None
    from_timestamp: Optional[int] = None
    to_timestamp: Optional[int] = None
    cursor: Optional[int] = None
    limit: Optional[int] = None
    include_ohlcv: Optional[bool] = None

    @classmethod
    def new(cls, orderbook_id: str) -> "PriceHistoryParams":
        """Create new params with required orderbook_id."""
        return cls(orderbook_id=orderbook_id)

    def with_resolution(self, resolution: Union[Resolution, str]) -> "PriceHistoryParams":
        """Set resolution."""
        self.resolution = resolution
        return self

    def with_time_range(self, from_ts: int, to_ts: int) -> "PriceHistoryParams":
        """Set time range."""
        self.from_timestamp = from_ts
        self.to_timestamp = to_ts
        return self

    def with_cursor(self, cursor: int) -> "PriceHistoryParams":
        """Set pagination cursor."""
        self.cursor = cursor
        return self

    def with_limit(self, limit: int) -> "PriceHistoryParams":
        """Set result limit."""
        self.limit = limit
        return self

    def with_ohlcv(self) -> "PriceHistoryParams":
        """Include OHLCV data."""
        self.include_ohlcv = True
        return self

    def to_query_params(self) -> dict:
        """Convert to query parameters dict."""
        params = {"orderbook_id": self.orderbook_id}
        if self.resolution is not None:
            if isinstance(self.resolution, Resolution):
                params["resolution"] = self.resolution.as_str()
            else:
                params["resolution"] = self.resolution
        if self.from_timestamp is not None:
            params["from"] = self.from_timestamp
        if self.to_timestamp is not None:
            params["to"] = self.to_timestamp
        if self.cursor is not None:
            params["cursor"] = self.cursor
        if self.limit is not None:
            params["limit"] = self.limit
        if self.include_ohlcv is not None:
            params["include_ohlcv"] = self.include_ohlcv
        return params


@dataclass
class PriceHistoryResponse:
    """Response for GET /api/price-history."""

    orderbook_id: str
    resolution: str
    include_ohlcv: bool
    prices: list[PricePoint]
    has_more: bool
    next_cursor: Optional[int] = None

    @classmethod
    def from_dict(cls, data: dict) -> "PriceHistoryResponse":
        return cls(
            orderbook_id=data["orderbook_id"],
            resolution=data["resolution"],
            include_ohlcv=data["include_ohlcv"],
            prices=[PricePoint.from_dict(p) for p in data.get("prices", [])],
            has_more=data["has_more"],
            next_cursor=data.get("next_cursor"),
        )
