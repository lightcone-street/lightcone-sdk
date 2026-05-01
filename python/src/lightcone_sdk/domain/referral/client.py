"""Referrals sub-client."""

from __future__ import annotations

from typing import TYPE_CHECKING

from . import ReferralStatus, ReferralCodeInfo, RedeemResult

if TYPE_CHECKING:
    from ...client import LightconeClient


class Referrals:
    """Referral operations sub-client."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    async def get_status(self) -> ReferralStatus:
        """Get referral status for the authenticated user."""
        data = await self._client._http.get("/api/referral/status")
        return _referral_status_from_wire(data)

    async def get_status_with_auth(self, auth_token: str) -> ReferralStatus:
        """Same as :meth:`get_status`, with an explicit per-call ``auth_token``.

        Intended for server-side cookie forwarding (SSR / route handlers)
        where the per-request browser cookie can't propagate to the SDK's
        process-wide cookie store. The override is used only for this call
        and never written back to the shared store.
        """
        data = await self._client._http.get_with_auth(
            "/api/referral/status",
            auth_token=auth_token,
        )
        return _referral_status_from_wire(data)

    async def redeem(self, code: str) -> RedeemResult:
        """Redeem a referral code."""
        data = await self._client._http.post("/api/referral/redeem", {"code": code})
        return RedeemResult(
            success=data.get("success", False),
            is_beta=data.get("is_beta", False),
        )


def _referral_status_from_wire(data: dict) -> ReferralStatus:
    codes = [
        ReferralCodeInfo(
            code=c.get("code", ""),
            max_uses=c.get("max_uses", 0),
            use_count=c.get("use_count", 0),
        )
        for c in data.get("referral_codes", [])
    ]
    return ReferralStatus(
        is_beta=data.get("is_beta", False),
        source=data.get("source"),
        referral_codes=codes,
    )
