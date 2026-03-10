"""Orderbooks sub-client."""

from typing import Optional, TYPE_CHECKING

from .wire import OrderbookDepthResponse, DecimalsResponse

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Orderbooks:
    """Orderbook operations sub-client."""

    def __init__(self, http: "LightconeHttp", decimals_cache: Optional[dict] = None):
        self._http = http
        self._decimals_cache: dict[str, DecimalsResponse] = decimals_cache or {}

    async def get(self, orderbook_id: str, depth: Optional[int] = None) -> OrderbookDepthResponse:
        """Get orderbook depth."""
        params: dict = {}
        if depth is not None:
            params["depth"] = str(depth)
        data = await self._http.get(
            f"/api/orderbook/{orderbook_id}",
            params=params or None,
        )
        return OrderbookDepthResponse.from_dict(data)

    async def decimals(self, orderbook_id: str) -> DecimalsResponse:
        """Get decimal configuration (cached)."""
        if orderbook_id in self._decimals_cache:
            return self._decimals_cache[orderbook_id]

        data = await self._http.get(f"/api/orderbooks/{orderbook_id}/decimals")
        result = DecimalsResponse.from_dict(data)
        self._decimals_cache[orderbook_id] = result
        return result

    def clear_cache(self) -> None:
        """Clear the decimals cache."""
        self._decimals_cache.clear()
