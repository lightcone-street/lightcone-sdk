"""Referrals sub-client."""

from typing import TYPE_CHECKING

from . import ReferralStatus, ReferralCodeInfo, RedeemResult

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Referrals:
    """Referral operations sub-client."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def get_status(self) -> ReferralStatus:
        """Get referral status for the authenticated user."""
        data = await self._http.get("/api/referral/status")
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
        data = await self._http.post("/api/referral/redeem", {"code": code})
        return RedeemResult(
            success=data.get("success", False),
            is_beta=data.get("is_beta", False),
        )
