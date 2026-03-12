"""Price history sub-client."""

from typing import Optional, TYPE_CHECKING

from . import LineData
from .wire import PriceCandle
from .convert import line_data_from_candle

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class PriceHistoryClient:
    """Price history operations."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def get(
        self,
        orderbook_id: str,
        resolution: str,
        from_ts: Optional[int] = None,
        to_ts: Optional[int] = None,
    ) -> list[LineData]:
        """Get price history for an orderbook."""
        params: dict = {
            "orderbook_id": orderbook_id,
            "resolution": resolution,
        }
        if from_ts is not None:
            params["from"] = str(from_ts * 1000)
        if to_ts is not None:
            params["to"] = str(to_ts * 1000)

        data = await self._http.get("/api/price-history", params=params)
        candles = [PriceCandle.from_dict(c) for c in data.get("prices", [])]
        return [line_data_from_candle(c) for c in candles]
