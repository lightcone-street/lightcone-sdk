"""Referral domain types."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class ReferralCodeInfo:
    code: str
    max_uses: int = 0
    use_count: int = 0


@dataclass
class ReferralStatus:
    is_beta: bool = False
    source: Optional[str] = None
    referral_codes: list[ReferralCodeInfo] = field(default_factory=list)


@dataclass
class RedeemResult:
    success: bool = False
    is_beta: bool = False


__all__ = ["ReferralCodeInfo", "ReferralStatus", "RedeemResult"]
