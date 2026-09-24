"""Jurisdiction sub-client."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .wire import JurisdictionResponse

if TYPE_CHECKING:
    from ...client import LightconeClient


class Jurisdiction:
    """The backend classification of this direct caller."""

    def __init__(self, client: LightconeClient) -> None:
        self._client = client

    async def geoblock(self) -> JurisdictionResponse:
        """GET /api/geoblock using this client's ordinary API transport."""
        data = await self._client._http.get("/api/geoblock")
        return JurisdictionResponse.from_dict(data)
