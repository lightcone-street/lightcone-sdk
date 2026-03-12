"""Admin domain types aligned with the Rust SDK."""

from dataclasses import dataclass, field
from typing import Generic, Optional, TypeVar

T = TypeVar("T")


def _compact_dict(payload: dict) -> dict:
    return {
        key: value
        for key, value in payload.items()
        if value is not None
    }


def _serialize_payload(value):
    if hasattr(value, "to_dict"):
        return value.to_dict()
    return value


@dataclass(frozen=True)
class TargetSpec:
    kind: str
    value: Optional[str] = None

    @staticmethod
    def all() -> "TargetSpec":
        return TargetSpec(kind="all")

    @staticmethod
    def user_id(user_id: str) -> "TargetSpec":
        return TargetSpec(kind="user_id", value=user_id)

    @staticmethod
    def wallet_address(wallet_address: str) -> "TargetSpec":
        return TargetSpec(kind="wallet_address", value=wallet_address)

    @staticmethod
    def code(code: str) -> "TargetSpec":
        return TargetSpec(kind="code", value=code)

    @staticmethod
    def batch_id(batch_id: str) -> "TargetSpec":
        return TargetSpec(kind="batch_id", value=batch_id)

    def to_dict(self):
        if self.kind == "all":
            return "all"
        if self.value is None:
            raise ValueError(f"TargetSpec '{self.kind}' requires a value")
        return {self.kind: self.value}


@dataclass
class AdminEnvelope(Generic[T]):
    """Signed admin request envelope."""

    payload: T
    signature: str

    def to_dict(self) -> dict:
        return {
            "payload": _serialize_payload(self.payload),
            "signature": self.signature,
        }


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
    s3_synced: Optional[bool] = None
    s3_synced_at: Optional[str] = None
    s3_error: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class OutcomeMetadataPayload:
    market_id: int
    outcome_index: int
    name: Optional[str] = None
    icon_url: Optional[str] = None
    description: Optional[str] = None
    metadata_uri: Optional[str] = None
    s3_synced: Optional[bool] = None
    s3_synced_at: Optional[str] = None
    s3_error: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class ConditionalTokenMetadataPayload:
    conditional_mint_id: int
    outcome_index: Optional[int] = None
    display_name: Optional[str] = None
    outcome: Optional[str] = None
    deposit_symbol: Optional[str] = None
    short_name: Optional[str] = None
    description: Optional[str] = None
    icon_url: Optional[str] = None
    metadata_uri: Optional[str] = None
    decimals: Optional[int] = None
    s3_synced: Optional[bool] = None
    s3_synced_at: Optional[str] = None
    s3_error: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class DepositTokenMetadataPayload:
    deposit_asset: str
    display_name: Optional[str] = None
    symbol: Optional[str] = None
    token_symbol: Optional[str] = None
    description: Optional[str] = None
    icon_url: Optional[str] = None
    metadata_uri: Optional[str] = None
    decimals: Optional[int] = None
    s3_synced: Optional[bool] = None
    s3_synced_at: Optional[str] = None
    s3_error: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class UnifiedMetadataRequest:
    markets: list[MarketMetadataPayload] = field(default_factory=list)
    outcomes: list[OutcomeMetadataPayload] = field(default_factory=list)
    conditional_tokens: list[ConditionalTokenMetadataPayload] = field(default_factory=list)
    deposit_tokens: list[DepositTokenMetadataPayload] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "markets": [_serialize_payload(payload) for payload in self.markets],
            "outcomes": [_serialize_payload(payload) for payload in self.outcomes],
            "conditional_tokens": [
                _serialize_payload(payload) for payload in self.conditional_tokens
            ],
            "deposit_tokens": [_serialize_payload(payload) for payload in self.deposit_tokens],
        }


@dataclass
class UnifiedMetadataResponse:
    status: str = ""
    markets: list[dict] = field(default_factory=list)
    outcomes: list[dict] = field(default_factory=list)
    conditional_tokens: list[dict] = field(default_factory=list)
    deposit_tokens: list[dict] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UnifiedMetadataResponse":
        return UnifiedMetadataResponse(
            status=d.get("status", ""),
            markets=d.get("markets") or [],
            outcomes=d.get("outcomes") or [],
            conditional_tokens=d.get("conditional_tokens") or [],
            deposit_tokens=d.get("deposit_tokens") or [],
        )


@dataclass
class AllocateCodesRequest:
    target: TargetSpec
    batch_id: Optional[str] = None
    vanity_codes: Optional[list[str]] = None
    count: Optional[int] = None
    max_uses: Optional[int] = None

    def to_dict(self) -> dict:
        return _compact_dict({
            "target": self.target.to_dict(),
            "batch_id": self.batch_id,
            "vanity_codes": self.vanity_codes,
            "count": self.count,
            "max_uses": self.max_uses,
        })


@dataclass
class AllocateCodesResponse:
    status: str = ""
    users_count: Optional[int] = None
    codes_allocated: Optional[int] = None
    user_id: Optional[str] = None
    codes: Optional[list[str]] = None

    @staticmethod
    def from_dict(d: dict) -> "AllocateCodesResponse":
        return AllocateCodesResponse(
            status=d.get("status", ""),
            users_count=d.get("users_count"),
            codes_allocated=d.get("codes_allocated"),
            user_id=d.get("user_id"),
            codes=d.get("codes"),
        )


@dataclass
class WhitelistRequest:
    wallet_addresses: list[str] = field(default_factory=list)
    allocate_codes: Optional[bool] = None

    def to_dict(self) -> dict:
        return _compact_dict({
            "wallet_addresses": self.wallet_addresses,
            "allocate_codes": self.allocate_codes,
        })


@dataclass
class WhitelistResponse:
    status: str = ""
    wallets_added: int = 0
    codes_allocated: int = 0

    @staticmethod
    def from_dict(d: dict) -> "WhitelistResponse":
        return WhitelistResponse(
            status=d.get("status", ""),
            wallets_added=d.get("wallets_added", 0),
            codes_allocated=d.get("codes_allocated", 0),
        )


@dataclass
class RevokeRequest:
    target: TargetSpec
    reason: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict({
            "target": self.target.to_dict(),
            "reason": self.reason,
        })


@dataclass
class RevokeResponse:
    revoked_count: int = 0
    user_ids: list[str] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "RevokeResponse":
        return RevokeResponse(
            revoked_count=d.get("revoked_count", 0),
            user_ids=d.get("user_ids") or [],
        )


@dataclass
class UnrevokeRequest:
    target: TargetSpec

    def to_dict(self) -> dict:
        return {"target": self.target.to_dict()}


@dataclass
class UnrevokeResponse:
    restored_count: int = 0
    user_ids: list[str] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UnrevokeResponse":
        return UnrevokeResponse(
            restored_count=d.get("restored_count", 0),
            user_ids=d.get("user_ids") or [],
        )


@dataclass
class CreateNotificationRequest:
    title: str
    message: str
    expires_at: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class CreateNotificationResponse:
    status: str = ""

    @staticmethod
    def from_dict(d: dict) -> "CreateNotificationResponse":
        return CreateNotificationResponse(status=d.get("status", ""))


@dataclass
class DismissNotificationRequest:
    notification_id: str = ""

    def to_dict(self) -> dict:
        return {"notification_id": self.notification_id}


@dataclass
class DismissNotificationResponse:
    status: str = ""

    @staticmethod
    def from_dict(d: dict) -> "DismissNotificationResponse":
        return DismissNotificationResponse(status=d.get("status", ""))


__all__ = [
    "TargetSpec",
    "AdminEnvelope",
    "MarketMetadataPayload",
    "OutcomeMetadataPayload",
    "ConditionalTokenMetadataPayload",
    "DepositTokenMetadataPayload",
    "UnifiedMetadataRequest",
    "UnifiedMetadataResponse",
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
