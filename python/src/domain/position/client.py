"""Positions sub-client."""

from typing import TYPE_CHECKING

from . import Position
from .wire import PositionsResponseWire, MarketPositionsResponseWire
from .convert import position_from_wire

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Positions:
    """Position operations sub-client."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def get(self, user_pubkey: str) -> list[Position]:
        """Get all positions for a user."""
        data = await self._http.get(f"/api/users/{user_pubkey}/positions")
        resp = PositionsResponseWire.from_dict(data)
        return [position_from_wire(p) for p in resp.positions]

    async def get_for_market(self, user_pubkey: str, market_pubkey: str) -> list[Position]:
        """Get positions in a specific market."""
        data = await self._http.get(
            f"/api/users/{user_pubkey}/markets/{market_pubkey}/positions"
        )
        resp = MarketPositionsResponseWire.from_dict(data)
        return [position_from_wire(p) for p in resp.positions]
