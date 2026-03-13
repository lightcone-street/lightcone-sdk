"""Positions sub-client."""

from typing import TYPE_CHECKING

from .wire import PositionsResponseWire, MarketPositionsResponseWire

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Positions:
    """Position operations sub-client."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def get(self, user_pubkey: str) -> PositionsResponseWire:
        """Get all positions for a user."""
        data = await self._http.get(f"/api/users/{user_pubkey}/positions")
        return PositionsResponseWire.from_dict(data)

    async def get_for_market(
        self,
        user_pubkey: str,
        market_pubkey: str,
    ) -> MarketPositionsResponseWire:
        """Get positions in a specific market."""
        data = await self._http.get(
            f"/api/users/{user_pubkey}/markets/{market_pubkey}/positions"
        )
        return MarketPositionsResponseWire.from_dict(data)
