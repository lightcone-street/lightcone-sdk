"""Markets sub-client."""

from typing import Optional, TYPE_CHECKING
from urllib.parse import quote as url_quote

from . import Market, MarketsResult, Status
from .wire import MarketWire, MarketResponse, MarketSearchResult
from .convert import market_from_wire, validation_errors_from_wire

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Markets:
    """Markets query operations."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def get(
        self,
        cursor: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> MarketsResult:
        """Get markets with Rust-aligned filtering and validation reporting."""
        url = "/api/markets"
        query_parts: list[str] = []
        if cursor is not None:
            query_parts.append(f"cursor={cursor}")
        if limit is not None:
            query_parts.append(f"limit={limit}")
        if query_parts:
            url += "?" + "&".join(query_parts)

        data = await self._http.get(url)
        resp = MarketResponse.from_dict(data)
        markets: list[Market] = []
        validation_errors: list[str] = []

        for wire_market in resp.markets:
            errors = validation_errors_from_wire(wire_market)
            validation_errors.extend(errors)
            if errors:
                continue

            market = market_from_wire(wire_market)
            if market.status in {Status.ACTIVE, Status.RESOLVED}:
                markets.append(market)

        return MarketsResult(
            markets=markets,
            validation_errors=validation_errors,
        )

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

    async def search(self, query: str, limit: Optional[int] = None) -> list[MarketSearchResult]:
        """Search markets by query string."""
        encoded = url_quote(query, safe='')
        url = f"/api/markets/search/by-query/{encoded}"
        if limit is not None:
            url += f"?limit={limit}"
        data = await self._http.get(url)
        markets_data = data if isinstance(data, list) else data.get("markets", [])
        return [MarketSearchResult.from_dict(m) for m in markets_data]

    async def featured(self) -> list[MarketSearchResult]:
        """Get featured markets."""
        data = await self._http.get("/api/markets/search/featured")
        markets_data = data if isinstance(data, list) else data.get("markets", [])
        results = [MarketSearchResult.from_dict(m) for m in markets_data]
        return [
            result for result in results
            if result.market_status in {"Active", "Resolved"}
        ]
