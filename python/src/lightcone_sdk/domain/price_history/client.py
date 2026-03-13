"""Price history sub-client."""

from typing import Optional, TYPE_CHECKING

from . import LineData
from .wire import (
    OrderbookPriceHistoryResponse,
    DepositPriceHistoryResponse,
)
from .convert import line_data_from_candle
from ...shared.types import Resolution

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class PriceHistoryClient:
    """Price history operations."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def get(
        self,
        orderbook_id: str,
        resolution: str | Resolution = "1m",
        from_ts: Optional[int] = None,
        to_ts: Optional[int] = None,
        cursor: Optional[int] = None,
        limit: Optional[int] = None,
        include_ohlcv: bool = False,
    ) -> OrderbookPriceHistoryResponse:
        """Get orderbook price history using Unix millisecond timestamps."""
        params: dict = {
            "orderbook_id": orderbook_id,
            "resolution": _resolution_value(resolution),
        }
        if from_ts is not None:
            params["from"] = str(_ensure_unix_milliseconds("from_ts", from_ts))
        if to_ts is not None:
            params["to"] = str(_ensure_unix_milliseconds("to_ts", to_ts))
        if cursor is not None:
            params["cursor"] = str(_ensure_unix_milliseconds("cursor", cursor))
        if limit is not None:
            params["limit"] = str(_ensure_page_limit(limit))
        if include_ohlcv:
            params["include_ohlcv"] = "true"

        data = await self._http.get("/api/price-history", params=params)
        return OrderbookPriceHistoryResponse.from_dict(data)

    async def get_deposit_asset(
        self,
        deposit_asset: str,
        resolution: str | Resolution = "1m",
        from_ts: Optional[int] = None,
        to_ts: Optional[int] = None,
        cursor: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> DepositPriceHistoryResponse:
        """Get deposit-token price history using Unix millisecond timestamps."""
        params: dict = {
            "deposit_asset": deposit_asset,
            "resolution": _resolution_value(resolution),
        }
        if from_ts is not None:
            params["from"] = str(_ensure_unix_milliseconds("from_ts", from_ts))
        if to_ts is not None:
            params["to"] = str(_ensure_unix_milliseconds("to_ts", to_ts))
        if cursor is not None:
            params["cursor"] = str(_ensure_unix_milliseconds("cursor", cursor))
        if limit is not None:
            params["limit"] = str(_ensure_page_limit(limit))

        data = await self._http.get("/api/price-history", params=params)
        return DepositPriceHistoryResponse.from_dict(data)

    async def get_line_data(
        self,
        orderbook_id: str,
        resolution: str | Resolution = "1m",
        from_ts: Optional[int] = None,
        to_ts: Optional[int] = None,
        cursor: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> list[LineData]:
        """Get orderbook midpoint line data for simple charts."""
        response = await self.get(
            orderbook_id,
            resolution=resolution,
            from_ts=from_ts,
            to_ts=to_ts,
            cursor=cursor,
            limit=limit,
            include_ohlcv=False,
        )
        return [line_data_from_candle(c) for c in response.prices]


def _ensure_unix_milliseconds(name: str, value: int) -> int:
    if value < 0:
        raise ValueError(f"{name} must be a non-negative Unix timestamp in milliseconds")
    if value < 10_000_000_000:
        raise ValueError(f"{name} must be a Unix timestamp in milliseconds, not seconds")
    return value


def _ensure_page_limit(value: int) -> int:
    if value < 1 or value > 1000:
        raise ValueError("limit must be an integer between 1 and 1000")
    return value


def _resolution_value(value: str | Resolution) -> str:
    if isinstance(value, Resolution):
        return value.as_str()
    return value
