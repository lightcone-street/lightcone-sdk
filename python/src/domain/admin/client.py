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
        return await self._http.post("/api/admin/metadata", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def allocate_codes(self, envelope: AdminEnvelope) -> dict:
        """Allocate referral codes."""
        return await self._http.post("/api/admin/referral/allocate", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def whitelist(self, envelope: AdminEnvelope) -> dict:
        """Whitelist wallet addresses."""
        return await self._http.post("/api/admin/referral/whitelist", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def revoke(self, envelope: AdminEnvelope) -> dict:
        """Revoke access."""
        return await self._http.post("/api/admin/referral/revoke", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def unrevoke(self, envelope: AdminEnvelope) -> dict:
        """Unrevoke access."""
        return await self._http.post("/api/admin/referral/unrevoke", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def create_notification(self, envelope: AdminEnvelope) -> dict:
        """Create a notification."""
        return await self._http.post("/api/admin/notifications", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })

    async def dismiss_notification(self, envelope: AdminEnvelope) -> dict:
        """Dismiss a notification."""
        return await self._http.post("/api/admin/notifications/dismiss", {
            "payload": envelope.payload,
            "signature": envelope.signature,
        })
