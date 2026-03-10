"""Admin sub-client."""

from typing import TYPE_CHECKING

from . import AdminEnvelope

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Admin:
    """Admin operations sub-client."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def upsert_metadata(self, envelope: AdminEnvelope) -> dict:
        """Upsert market/token metadata."""
        return await self._http.post("/api/admin/upsert-metadata", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def allocate_codes(self, envelope: AdminEnvelope) -> dict:
        """Allocate referral codes."""
        return await self._http.post("/api/admin/allocate-codes", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def whitelist(self, envelope: AdminEnvelope) -> dict:
        """Whitelist wallet addresses."""
        return await self._http.post("/api/admin/whitelist", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def revoke(self, envelope: AdminEnvelope) -> dict:
        """Revoke access."""
        return await self._http.post("/api/admin/revoke", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def unrevoke(self, envelope: AdminEnvelope) -> dict:
        """Unrevoke access."""
        return await self._http.post("/api/admin/unrevoke", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })
