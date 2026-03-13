"""Admin sub-client."""

from typing import TYPE_CHECKING

from . import (
    AdminEnvelope,
    AllocateCodesResponse,
    CreateNotificationResponse,
    DismissNotificationResponse,
    RevokeResponse,
    UnifiedMetadataResponse,
    UnrevokeResponse,
    WhitelistResponse,
)

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Admin:
    """Admin operations sub-client."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def upsert_metadata(self, envelope: AdminEnvelope) -> UnifiedMetadataResponse:
        """Upsert market/token metadata."""
        data = await self._http.post("/api/admin/metadata", envelope.to_dict())
        return UnifiedMetadataResponse.from_dict(data)

    async def allocate_codes(self, envelope: AdminEnvelope) -> AllocateCodesResponse:
        """Allocate referral codes."""
        data = await self._http.post("/api/admin/referral/allocate", envelope.to_dict())
        return AllocateCodesResponse.from_dict(data)

    async def whitelist(self, envelope: AdminEnvelope) -> WhitelistResponse:
        """Whitelist wallet addresses."""
        data = await self._http.post("/api/admin/referral/whitelist", envelope.to_dict())
        return WhitelistResponse.from_dict(data)

    async def revoke(self, envelope: AdminEnvelope) -> RevokeResponse:
        """Revoke access."""
        data = await self._http.post("/api/admin/referral/revoke", envelope.to_dict())
        return RevokeResponse.from_dict(data)

    async def unrevoke(self, envelope: AdminEnvelope) -> UnrevokeResponse:
        """Unrevoke access."""
        data = await self._http.post("/api/admin/referral/unrevoke", envelope.to_dict())
        return UnrevokeResponse.from_dict(data)

    async def create_notification(self, envelope: AdminEnvelope) -> CreateNotificationResponse:
        """Create a notification."""
        data = await self._http.post("/api/admin/notifications", envelope.to_dict())
        return CreateNotificationResponse.from_dict(data)

    async def dismiss_notification(
        self,
        envelope: AdminEnvelope,
    ) -> DismissNotificationResponse:
        """Dismiss a notification."""
        data = await self._http.post("/api/admin/notifications/dismiss", envelope.to_dict())
        return DismissNotificationResponse.from_dict(data)
