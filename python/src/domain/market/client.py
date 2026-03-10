"""Markets sub-client."""

from typing import Optional, TYPE_CHECKING
from urllib.parse import quote as url_quote

from . import Market
from .wire import MarketWire, MarketResponse, MarketSearchResult
from .convert import market_from_wire

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Markets:
    """Markets query operations."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def get(
        self,
        cursor: Optional[str] = None,
        limit: Optional[int] = None,
    ) -> tuple[list[Market], Optional[str]]:
        """Get all markets (paginated).

        Returns:
            Tuple of (markets, next_cursor)
        """
        params: dict = {}
        if cursor is not None:
            params["cursor"] = cursor
        if limit is not None:
            params["limit"] = str(limit)

        data = await self._http.get("/api/markets", params=params or None)
        resp = MarketResponse.from_dict(data)
        markets = [market_from_wire(m) for m in resp.markets]
        return markets, resp.next_cursor

    async def get_by_slug(self, slug: str) -> Market:
        """Get a market by its URL slug."""
        data = await self._http.get(f"/api/markets/by-slug/{url_quote(slug, safe='')}")
        wire = MarketWire.from_dict(data.get("market", data))
        return market_from_wire(wire)

    async def get_by_pubkey(self, pubkey: str) -> Market:
        """Get a market by its pubkey."""
        data = await self._http.get(f"/api/markets/{url_quote(pubkey, safe='')}")
        wire = MarketWire.from_dict(data.get("market", data))
        return market_from_wire(wire)

    async def search(self, query: str, limit: Optional[int] = None) -> list[Market]:
        """Search markets by query string."""
        encoded = url_quote(query, safe='')
        params: dict = {}
        if limit is not None:
            params["limit"] = str(limit)
        data = await self._http.get(
            f"/api/markets/search/by-query/{encoded}",
            params=params or None,
        )
        markets_data = data if isinstance(data, list) else data.get("markets", [])
        return [market_from_wire(MarketWire.from_dict(m)) for m in markets_data]

    async def featured(self) -> list[Market]:
        """Get featured markets."""
        data = await self._http.get("/api/markets/search/featured")
        markets_data = data if isinstance(data, list) else data.get("markets", [])
        return [market_from_wire(MarketWire.from_dict(m)) for m in markets_data]
