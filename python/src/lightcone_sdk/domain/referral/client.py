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

    async def redeem(self, code: str) -> RedeemResult:
        """Redeem a referral code."""
        data = await self._client._http.post("/api/referral/redeem", {"code": code})
        return RedeemResult(
            success=data.get("success", False),
            is_beta=data.get("is_beta", False),
        )
