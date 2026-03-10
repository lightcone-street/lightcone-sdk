"""Admin domain types."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class AdminEnvelope:
    """Signed admin request envelope."""
    payload: dict
    signature: str


@dataclass
class MarketMetadataPayload:
    market_id: int
    market_name: Optional[str] = None
    slug: Optional[str] = None
    description: Optional[str] = None
    definition: Optional[str] = None
    banner_image_url: Optional[str] = None
    icon_url: Optional[str] = None
    category: Optional[str] = None
    subcategory: Optional[str] = None
    tags: Optional[list[str]] = None
    featured_rank: Optional[int] = None
    metadata_uri: Optional[str] = None


@dataclass
class OutcomeMetadataPayload:
    market_id: int
    outcome_index: int
    name: Optional[str] = None
    description: Optional[str] = None
    icon_url: Optional[str] = None
    display_name: Optional[str] = None
    short_name: Optional[str] = None
    symbol: Optional[str] = None


@dataclass
class ConditionalTokenMetadataPayload:
    token_address: str
    market_id: Optional[int] = None
    outcome_index: Optional[int] = None
    display_name: Optional[str] = None
    short_name: Optional[str] = None
    symbol: Optional[str] = None
    description: Optional[str] = None
    icon_url: Optional[str] = None
    decimals: Optional[int] = None


@dataclass
class DepositTokenMetadataPayload:
    deposit_asset: str
    market_id: Optional[int] = None
    display_name: Optional[str] = None
    short_name: Optional[str] = None
    symbol: Optional[str] = None
    description: Optional[str] = None
    icon_url: Optional[str] = None
    decimals: Optional[int] = None


@dataclass
class UnifiedMetadataRequest:
    markets: list[MarketMetadataPayload] = field(default_factory=list)
    outcomes: list[OutcomeMetadataPayload] = field(default_factory=list)
    conditional_tokens: list[ConditionalTokenMetadataPayload] = field(default_factory=list)
    deposit_tokens: list[DepositTokenMetadataPayload] = field(default_factory=list)


@dataclass
class AllocateCodesRequest:
    target: dict = field(default_factory=dict)
    batch_id: Optional[str] = None
    vanity_codes: Optional[list[str]] = None
    count: int = 1
    max_uses: int = 1


@dataclass
class AllocateCodesResponse:
    users_count: int = 0
    codes_allocated: int = 0
    user_id: Optional[str] = None
    codes: list[str] = field(default_factory=list)


@dataclass
class WhitelistRequest:
    wallet_addresses: list[str] = field(default_factory=list)
    allocate_codes: bool = False


@dataclass
class WhitelistResponse:
    wallets_added: int = 0
    codes_allocated: int = 0


@dataclass
class RevokeRequest:
    target: dict = field(default_factory=dict)
    reason: Optional[str] = None


@dataclass
class RevokeResponse:
    revoked_count: int = 0
    user_ids: list[str] = field(default_factory=list)


@dataclass
class UnrevokeRequest:
    target: dict = field(default_factory=dict)


@dataclass
class UnrevokeResponse:
    restored_count: int = 0
    user_ids: list[str] = field(default_factory=list)


@dataclass
class CreateNotificationRequest:
    title: str
    message: str
    expires_at: Optional[str] = None


@dataclass
class CreateNotificationResponse:
    status: str = ""


@dataclass
class DismissNotificationRequest:
    notification_id: str = ""


@dataclass
class DismissNotificationResponse:
    status: str = ""


__all__ = [
    "AdminEnvelope",
    "MarketMetadataPayload",
    "OutcomeMetadataPayload",
    "ConditionalTokenMetadataPayload",
    "DepositTokenMetadataPayload",
    "UnifiedMetadataRequest",
    "AllocateCodesRequest",
    "AllocateCodesResponse",
    "WhitelistRequest",
    "WhitelistResponse",
    "RevokeRequest",
    "RevokeResponse",
    "UnrevokeRequest",
    "UnrevokeResponse",
    "CreateNotificationRequest",
    "CreateNotificationResponse",
    "DismissNotificationRequest",
    "DismissNotificationResponse",
]
