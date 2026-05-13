"""Admin domain types aligned with the Rust SDK."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional, Union

from ...error import SdkError
from ..market.wire import MarketWire


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


def _serialize_query_value(value) -> str:
    if isinstance(value, Enum):
        return str(value.value)
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value)


AdminMarketsRangeValue = Union[int, float, str]


# ============================================================================
# ADMIN AUTH
# ============================================================================


@dataclass(frozen=True)
class AdminNonceResponse:
    """Response from GET /api/admin/nonce."""

    nonce: str
    message: str

    @staticmethod
    def from_dict(data: dict) -> "AdminNonceResponse":
        return AdminNonceResponse(
            nonce=data.get("nonce", ""),
            message=data.get("message", ""),
        )


@dataclass(frozen=True)
class AdminLoginRequest:
    """Request payload for POST /api/admin/login."""

    message: str
    signature_bs58: str
    pubkey_bytes: list[int]

    def to_dict(self) -> dict:
        return {
            "message": self.message,
            "signature_bs58": self.signature_bs58,
            "pubkey_bytes": self.pubkey_bytes,
        }


@dataclass(frozen=True)
class AdminLoginResponse:
    """Response from POST /api/admin/login."""

    wallet_address: str
    expires_at: int

    @staticmethod
    def from_dict(data: dict) -> "AdminLoginResponse":
        return AdminLoginResponse(
            wallet_address=data.get("wallet_address", ""),
            expires_at=data.get("expires_at", 0),
        )


# ============================================================================
# TARGET SPEC
# ============================================================================


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
            raise SdkError(f"TargetSpec '{self.kind}' requires a value")
        return {self.kind: self.value}


@dataclass
class MarketMetadataPayload:
    market_id: int
    market_name: Optional[str] = None
    slug: Optional[str] = None
    description: Optional[str] = None
    definition: Optional[str] = None
    banner_image_url_low: Optional[str] = None
    banner_image_url_medium: Optional[str] = None
    banner_image_url_high: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    category: Optional[str] = None
    subcategory: Optional[str] = None
    tags: Optional[list[str]] = None
    featured_rank: Optional[int] = None
    metadata_uri: Optional[str] = None
    resolution: Optional[bool] = None
    resolution_by: Optional[int] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class OutcomeMetadataPayload:
    market_id: int
    outcome_index: int
    name: Optional[str] = None
    name_long: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    description: Optional[str] = None
    metadata_uri: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class ConditionalTokenMetadataPayload:
    conditional_mint_id: int
    outcome_index: Optional[int] = None
    outcome: Optional[str] = None
    deposit_symbol: Optional[str] = None
    short_symbol: Optional[str] = None
    description: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    metadata_uri: Optional[str] = None
    decimals: Optional[int] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class DepositTokenMetadataPayload:
    deposit_asset: str
    display_name: Optional[str] = None
    symbol: Optional[str] = None
    token_symbol: Optional[str] = None
    description: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    metadata_uri: Optional[str] = None
    decimals: Optional[int] = None
    min_order_size: Optional[int] = None
    binance_symbol: Optional[str] = None
    binance_enabled: Optional[bool] = None
    okx_inst_id: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class DepositTokenMetadataResponse:
    id: int = 0
    deposit_asset: str = ""
    display_name: str = ""
    symbol: str = ""
    token_symbol: Optional[str] = None
    binance_symbol: Optional[str] = None
    binance_enabled: bool = False
    okx_inst_id: Optional[str] = None
    description: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    metadata_uri: Optional[str] = None
    decimals: Optional[int] = None
    min_order_size: int = 0
    created_at: str = ""
    updated_at: str = ""

    @staticmethod
    def from_dict(d: dict) -> "DepositTokenMetadataResponse":
        return DepositTokenMetadataResponse(
            id=d.get("id", 0),
            deposit_asset=d.get("deposit_asset", ""),
            display_name=d.get("display_name", ""),
            symbol=d.get("symbol", ""),
            token_symbol=d.get("token_symbol"),
            binance_symbol=d.get("binance_symbol"),
            binance_enabled=d.get("binance_enabled", False),
            okx_inst_id=d.get("okx_inst_id"),
            description=d.get("description"),
            icon_url_low=d.get("icon_url_low"),
            icon_url_medium=d.get("icon_url_medium"),
            icon_url_high=d.get("icon_url_high"),
            metadata_uri=d.get("metadata_uri"),
            decimals=d.get("decimals"),
            min_order_size=d.get("min_order_size", 0),
            created_at=d.get("created_at", ""),
            updated_at=d.get("updated_at", ""),
        )


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
    markets: list[dict] = field(default_factory=list)
    outcomes: list[dict] = field(default_factory=list)
    conditional_tokens: list[dict] = field(default_factory=list)
    deposit_tokens: list[DepositTokenMetadataResponse] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UnifiedMetadataResponse":
        return UnifiedMetadataResponse(
            markets=d.get("markets") or [],
            outcomes=d.get("outcomes") or [],
            conditional_tokens=d.get("conditional_tokens") or [],
            deposit_tokens=[
                DepositTokenMetadataResponse.from_dict(token)
                for token in d.get("deposit_tokens", [])
            ],
        )


@dataclass
class MetadataCategoriesResponse:
    categories: list[str] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "MetadataCategoriesResponse":
        return MetadataCategoriesResponse(categories=d.get("categories") or [])


@dataclass
class AddMetadataCategoryRequest:
    category: str

    def to_dict(self) -> dict:
        return {"category": self.category}


@dataclass
class AddMetadataCategoryResponse:
    category: str = ""

    @staticmethod
    def from_dict(d: dict) -> "AddMetadataCategoryResponse":
        return AddMetadataCategoryResponse(category=d.get("category", ""))


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


# ============================================================================
# ADMIN MARKETS
# ============================================================================


class AdminMarketStatusFilter(str, Enum):
    """SDK-facing filter values for GET /api/admin/markets."""

    ALL = "all"
    ACTIVE = "active"
    RESOLVED = "resolved"


class AdminMarketStatus(str, Enum):
    """Database lifecycle values returned by GET /api/admin/markets."""

    ACTIVE = "Active"
    RESOLVED = "Resolved"


@dataclass
class AdminMarketsQuery:
    cursor: Optional[int] = None
    limit: Optional[int] = None
    sort_by: Optional[str] = None
    sort_direction: Optional[str] = None
    market_status: Optional[AdminMarketStatusFilter] = None
    category: Optional[str] = None
    search: Optional[str] = None

    min_volume_24h_usd: Optional[AdminMarketsRangeValue] = None
    max_volume_24h_usd: Optional[AdminMarketsRangeValue] = None
    min_volume_7d_usd: Optional[AdminMarketsRangeValue] = None
    max_volume_7d_usd: Optional[AdminMarketsRangeValue] = None
    min_volume_30d_usd: Optional[AdminMarketsRangeValue] = None
    max_volume_30d_usd: Optional[AdminMarketsRangeValue] = None
    min_volume_total_usd: Optional[AdminMarketsRangeValue] = None
    max_volume_total_usd: Optional[AdminMarketsRangeValue] = None

    min_unique_traders_24h: Optional[int] = None
    max_unique_traders_24h: Optional[int] = None
    min_unique_traders_7d: Optional[int] = None
    max_unique_traders_7d: Optional[int] = None
    min_unique_traders_30d: Optional[int] = None
    max_unique_traders_30d: Optional[int] = None
    min_unique_traders_total: Optional[int] = None
    max_unique_traders_total: Optional[int] = None

    min_open_interest_usd: Optional[AdminMarketsRangeValue] = None
    max_open_interest_usd: Optional[AdminMarketsRangeValue] = None

    min_fees_24h_usd: Optional[AdminMarketsRangeValue] = None
    max_fees_24h_usd: Optional[AdminMarketsRangeValue] = None
    min_fees_7d_usd: Optional[AdminMarketsRangeValue] = None
    max_fees_7d_usd: Optional[AdminMarketsRangeValue] = None
    min_fees_30d_usd: Optional[AdminMarketsRangeValue] = None
    max_fees_30d_usd: Optional[AdminMarketsRangeValue] = None
    min_fees_total_usd: Optional[AdminMarketsRangeValue] = None
    max_fees_total_usd: Optional[AdminMarketsRangeValue] = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {}
        for key, value in self.__dict__.items():
            if value is not None:
                params[key] = _serialize_query_value(value)
        return params


@dataclass(frozen=True)
class AdminMarketRow:
    rank: int
    market_id: int
    market_pubkey: str
    market_status: AdminMarketStatus
    slug: Optional[str]
    market_name: Optional[str]
    category: Optional[str]
    icon_url: Optional[str]
    num_outcomes: int
    resolution: bool
    resolution_by: Optional[int]
    open_interest_usd: str
    volume_24h_usd: str
    volume_7d_usd: str
    volume_30d_usd: str
    volume_total_usd: str
    unique_traders_24h: int
    unique_traders_7d: int
    unique_traders_30d: int
    unique_traders_total: int
    fees_24h_usd: str
    fees_7d_usd: str
    fees_30d_usd: str
    fees_total_usd: str
    created_at: str
    activated_at: Optional[str]
    settled_at: Optional[str]
    updated_at: str

    @staticmethod
    def from_dict(d: dict) -> "AdminMarketRow":
        return AdminMarketRow(
            rank=d.get("rank", 0),
            market_id=d.get("market_id", 0),
            market_pubkey=d.get("market_pubkey", ""),
            market_status=AdminMarketStatus(d.get("market_status", "Active")),
            slug=d.get("slug"),
            market_name=d.get("market_name"),
            category=d.get("category"),
            icon_url=d.get("icon_url"),
            num_outcomes=d.get("num_outcomes", 0),
            resolution=d.get("resolution", False),
            resolution_by=d.get("resolution_by"),
            open_interest_usd=str(d.get("open_interest_usd", "0")),
            volume_24h_usd=str(d.get("volume_24h_usd", "0")),
            volume_7d_usd=str(d.get("volume_7d_usd", "0")),
            volume_30d_usd=str(d.get("volume_30d_usd", "0")),
            volume_total_usd=str(d.get("volume_total_usd", "0")),
            unique_traders_24h=d.get("unique_traders_24h", 0),
            unique_traders_7d=d.get("unique_traders_7d", 0),
            unique_traders_30d=d.get("unique_traders_30d", 0),
            unique_traders_total=d.get("unique_traders_total", 0),
            fees_24h_usd=str(d.get("fees_24h_usd", "0")),
            fees_7d_usd=str(d.get("fees_7d_usd", "0")),
            fees_30d_usd=str(d.get("fees_30d_usd", "0")),
            fees_total_usd=str(d.get("fees_total_usd", "0")),
            created_at=d.get("created_at", ""),
            activated_at=d.get("activated_at"),
            settled_at=d.get("settled_at"),
            updated_at=d.get("updated_at", ""),
        )


@dataclass
class AdminMarketsResponse:
    timestamp: int = 0
    sort_by: str = ""
    sort_direction: str = ""
    total: int = 0
    limit: int = 0
    next_cursor: Optional[int] = None
    has_more: bool = False
    markets: list[AdminMarketRow] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "AdminMarketsResponse":
        return AdminMarketsResponse(
            timestamp=d.get("timestamp", 0),
            sort_by=d.get("sort_by", ""),
            sort_direction=d.get("sort_direction", ""),
            total=d.get("total", 0),
            limit=d.get("limit", 0),
            next_cursor=d.get("next_cursor"),
            has_more=d.get("has_more", False),
            markets=[
                AdminMarketRow.from_dict(market)
                for market in d.get("markets", [])
            ],
        )


# ============================================================================
# MARKETS TO SETTLE ADMIN
# ============================================================================


@dataclass
class MarketsToSettleCountResponse:
    markets_to_settle_count: int = 0

    @staticmethod
    def from_dict(d: dict) -> "MarketsToSettleCountResponse":
        return MarketsToSettleCountResponse(
            markets_to_settle_count=d.get("markets_to_settle_count", 0),
        )


@dataclass
class MarketsToSettleQuery:
    cursor: Optional[int] = None
    limit: Optional[int] = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {}
        if self.cursor is not None:
            params["cursor"] = str(self.cursor)
        if self.limit is not None:
            params["limit"] = str(self.limit)
        return params


@dataclass
class MarketsToSettleResponse:
    markets: list[MarketWire] = field(default_factory=list)
    next_cursor: Optional[int] = None
    has_more: bool = False

    @staticmethod
    def from_dict(d: dict) -> "MarketsToSettleResponse":
        return MarketsToSettleResponse(
            markets=[MarketWire.from_dict(market) for market in d.get("markets", [])],
            next_cursor=d.get("next_cursor"),
            has_more=d.get("has_more", False),
        )


# ============================================================================
# REFERRAL CONFIG / CODES ADMIN
# ============================================================================


@dataclass(frozen=True)
class ReferralConfig:
    """Response from POST /api/admin/referral/config/get and /update."""

    default_code_count: int
    updated_at: str

    @staticmethod
    def from_dict(d: dict) -> "ReferralConfig":
        return ReferralConfig(
            default_code_count=d.get("default_code_count", 0),
            updated_at=d.get("updated_at", ""),
        )


@dataclass
class UpdateConfigRequest:
    """Request payload for POST /api/admin/referral/config/update."""

    default_code_count: Optional[int] = None

    def to_dict(self) -> dict:
        return _compact_dict({"default_code_count": self.default_code_count})


@dataclass
class ListCodesRequest:
    """Request payload for POST /api/admin/referral/codes (admin list)."""

    limit: int = 100
    offset: int = 0
    owner_user_id: Optional[str] = None
    batch_id: Optional[str] = None
    code: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict({
            "limit": self.limit,
            "offset": self.offset,
            "owner_user_id": self.owner_user_id,
            "batch_id": self.batch_id,
            "code": self.code,
        })


@dataclass(frozen=True)
class CodeListEntry:
    """A single referral code returned from the admin list endpoint."""

    code: str
    owner_user_id: str
    batch_id: str
    is_vanity: bool
    max_uses: int
    use_count: int
    created_at: str

    @staticmethod
    def from_dict(d: dict) -> "CodeListEntry":
        return CodeListEntry(
            code=d.get("code", ""),
            owner_user_id=d.get("owner_user_id", ""),
            batch_id=d.get("batch_id", ""),
            is_vanity=d.get("is_vanity", False),
            max_uses=d.get("max_uses", 0),
            use_count=d.get("use_count", 0),
            created_at=d.get("created_at", ""),
        )


@dataclass
class ListCodesResponse:
    """Response from POST /api/admin/referral/codes."""

    codes: list[CodeListEntry] = field(default_factory=list)
    count: int = 0

    @staticmethod
    def from_dict(d: dict) -> "ListCodesResponse":
        return ListCodesResponse(
            codes=[CodeListEntry.from_dict(c) for c in d.get("codes", [])],
            count=d.get("count", 0),
        )


@dataclass
class UpdateCodeRequest:
    """Request payload for POST /api/admin/referral/codes/update."""

    code: str
    max_uses: int

    def to_dict(self) -> dict:
        return {"code": self.code, "max_uses": self.max_uses}


@dataclass
class UpdateCodeResponse:
    """Response from POST /api/admin/referral/codes/update."""

    status: str = ""
    code: str = ""
    max_uses: int = 0

    @staticmethod
    def from_dict(d: dict) -> "UpdateCodeResponse":
        return UpdateCodeResponse(
            status=d.get("status", ""),
            code=d.get("code", ""),
            max_uses=d.get("max_uses", 0),
        )


# ============================================================================
# ADMIN LOGS
# ============================================================================


@dataclass
class AdminLogEventsQuery:
    """Filter set for GET /api/admin/logs/events — all fields optional."""

    from_ms: Optional[int] = None
    to_ms: Optional[int] = None
    service_name: Optional[str] = None
    environment: Optional[str] = None
    category: Optional[str] = None
    severity: Optional[str] = None
    component: Optional[str] = None
    operation: Optional[str] = None
    fingerprint: Optional[str] = None
    response_status: Optional[str] = None
    user_visible: Optional[bool] = None
    request_id: Optional[str] = None
    user_pubkey: Optional[str] = None
    market_pubkey: Optional[str] = None
    orderbook_id: Optional[str] = None
    order_hash: Optional[str] = None
    trigger_order_id: Optional[str] = None
    tx_signature: Optional[str] = None
    checkpoint_signature: Optional[str] = None
    limit: Optional[int] = None
    cursor: Optional[str] = None

    def to_query(self) -> dict[str, str]:
        """Return the non-None fields as a string dict for URL query encoding."""
        params: dict[str, str] = {}
        for key, value in self.__dict__.items():
            if value is None:
                continue
            if isinstance(value, bool):
                params[key] = "true" if value else "false"
            else:
                params[key] = str(value)
        return params


@dataclass(frozen=True)
class AdminLogEvent:
    """A single structured log event."""

    id: int
    public_id: str
    service_name: str
    environment: str
    component: str
    operation: str
    category: str
    severity: str
    occurred_at_ms: int
    created_at_ms: int
    user_visible: bool
    message: str
    context: Any
    occurred_at: Optional[str] = None
    created_at: Optional[str] = None
    request_id: Optional[str] = None
    user_pubkey: Optional[str] = None
    market_pubkey: Optional[str] = None
    orderbook_id: Optional[str] = None
    order_hash: Optional[str] = None
    trigger_order_id: Optional[str] = None
    tx_signature: Optional[str] = None
    checkpoint_signature: Optional[str] = None
    http_status: Optional[int] = None
    grpc_code: Optional[str] = None
    fingerprint: Optional[str] = None
    response_status: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "AdminLogEvent":
        return AdminLogEvent(
            id=d.get("id", 0),
            public_id=d.get("public_id", ""),
            service_name=d.get("service_name", ""),
            environment=d.get("environment", ""),
            component=d.get("component", ""),
            operation=d.get("operation", ""),
            category=d.get("category", ""),
            severity=d.get("severity", ""),
            occurred_at_ms=d.get("occurred_at_ms", 0),
            created_at_ms=d.get("created_at_ms", 0),
            user_visible=d.get("user_visible", False),
            message=d.get("message", ""),
            context=d.get("context"),
            occurred_at=d.get("occurred_at"),
            created_at=d.get("created_at"),
            request_id=d.get("request_id"),
            user_pubkey=d.get("user_pubkey"),
            market_pubkey=d.get("market_pubkey"),
            orderbook_id=d.get("orderbook_id"),
            order_hash=d.get("order_hash"),
            trigger_order_id=d.get("trigger_order_id"),
            tx_signature=d.get("tx_signature"),
            checkpoint_signature=d.get("checkpoint_signature"),
            http_status=d.get("http_status"),
            grpc_code=d.get("grpc_code"),
            fingerprint=d.get("fingerprint"),
            response_status=d.get("response_status"),
        )


@dataclass
class AdminLogEventsResponse:
    """Response from GET /api/admin/logs/events."""

    events: list[AdminLogEvent] = field(default_factory=list)
    limit: int = 0
    next_cursor: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "AdminLogEventsResponse":
        return AdminLogEventsResponse(
            events=[AdminLogEvent.from_dict(e) for e in d.get("events", [])],
            limit=d.get("limit", 0),
            next_cursor=d.get("next_cursor"),
        )


@dataclass
class AdminLogMetricsQuery:
    """Query for GET /api/admin/logs/metrics."""

    windows: Optional[str] = None  # CSV e.g. "1h,24h"
    scopes: Optional[str] = None   # CSV e.g. "service,component"
    limit_per_scope: Optional[int] = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {}
        if self.windows is not None:
            params["windows"] = self.windows
        if self.scopes is not None:
            params["scopes"] = self.scopes
        if self.limit_per_scope is not None:
            params["limit_per_scope"] = str(self.limit_per_scope)
        return params


@dataclass(frozen=True)
class AdminLogMetricSummary:
    scope_key: str
    total_count: int
    error_count: int
    critical_count: int
    user_visible_count: int
    computed_at_ms: int
    computed_at: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "AdminLogMetricSummary":
        return AdminLogMetricSummary(
            scope_key=d.get("scope_key", ""),
            total_count=d.get("total_count", 0),
            error_count=d.get("error_count", 0),
            critical_count=d.get("critical_count", 0),
            user_visible_count=d.get("user_visible_count", 0),
            computed_at_ms=d.get("computed_at_ms", 0),
            computed_at=d.get("computed_at"),
        )


@dataclass
class AdminLogMetricBreakdown:
    window: str
    scope: str
    rows: list[AdminLogMetricSummary] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "AdminLogMetricBreakdown":
        return AdminLogMetricBreakdown(
            window=d.get("window", ""),
            scope=d.get("scope", ""),
            rows=[AdminLogMetricSummary.from_dict(r) for r in d.get("rows", [])],
        )


@dataclass
class AdminLogMetricsResponse:
    """Response from GET /api/admin/logs/metrics."""

    computed_at_ms: int = 0
    breakdowns: list[AdminLogMetricBreakdown] = field(default_factory=list)
    computed_at: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "AdminLogMetricsResponse":
        return AdminLogMetricsResponse(
            computed_at_ms=d.get("computed_at_ms", 0),
            breakdowns=[
                AdminLogMetricBreakdown.from_dict(b)
                for b in d.get("breakdowns", [])
            ],
            computed_at=d.get("computed_at"),
        )


@dataclass
class AdminLogMetricHistoryQuery:
    """Query for GET /api/admin/logs/metrics/history."""

    scope: str
    scope_key: Optional[str] = None
    resolution: str = "1h"
    from_ms: Optional[int] = None
    to_ms: Optional[int] = None
    limit: Optional[int] = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {
            "scope": self.scope,
            "resolution": self.resolution,
        }
        if self.scope_key is not None:
            params["scope_key"] = self.scope_key
        if self.from_ms is not None:
            params["from_ms"] = str(self.from_ms)
        if self.to_ms is not None:
            params["to_ms"] = str(self.to_ms)
        if self.limit is not None:
            params["limit"] = str(self.limit)
        return params


@dataclass(frozen=True)
class AdminLogMetricPoint:
    bucket_start_ms: int
    total_count: int
    error_count: int
    critical_count: int
    user_visible_count: int
    bucket_start: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "AdminLogMetricPoint":
        return AdminLogMetricPoint(
            bucket_start_ms=d.get("bucket_start_ms", 0),
            total_count=d.get("total_count", 0),
            error_count=d.get("error_count", 0),
            critical_count=d.get("critical_count", 0),
            user_visible_count=d.get("user_visible_count", 0),
            bucket_start=d.get("bucket_start"),
        )


@dataclass
class AdminLogMetricHistoryResponse:
    """Response from GET /api/admin/logs/metrics/history."""

    scope: str = ""
    scope_key: str = ""
    resolution: str = ""
    from_ms: int = 0
    to_ms: int = 0
    points: list[AdminLogMetricPoint] = field(default_factory=list)
    from_: Optional[str] = None
    to: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "AdminLogMetricHistoryResponse":
        return AdminLogMetricHistoryResponse(
            scope=d.get("scope", ""),
            scope_key=d.get("scope_key", ""),
            resolution=d.get("resolution", ""),
            from_ms=d.get("from_ms", 0),
            to_ms=d.get("to_ms", 0),
            points=[AdminLogMetricPoint.from_dict(p) for p in d.get("points", [])],
            from_=d.get("from"),
            to=d.get("to"),
        )


@dataclass
class CriticalLogErrors24hCountResponse:
    critical_log_errors_24h: int = 0

    @staticmethod
    def from_dict(d: dict) -> "CriticalLogErrors24hCountResponse":
        return CriticalLogErrors24hCountResponse(
            critical_log_errors_24h=d.get("critical_log_errors_24h", 0),
        )


# ============================================================================
# MARKET DEPLOYMENT ASSET UPLOAD
# ============================================================================


@dataclass
class MarketDeploymentMarket:
    """Market-level fields for a deployment asset upload.

    Image uploads are quality-specific WebP data URLs. Hosted URL fields are
    preserved separately and are used when no matching data URL is supplied.
    """

    name: str
    slug: str
    description: Optional[str] = None
    definition: Optional[str] = None
    banner_image_url_low: Optional[str] = None
    banner_image_url_medium: Optional[str] = None
    banner_image_url_high: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    category: Optional[str] = None
    subcategory: Optional[str] = None
    tags: list[str] = field(default_factory=list)
    featured_rank: Optional[int] = None
    banner_image_data_url_low: Optional[str] = None
    banner_image_content_type_low: Optional[str] = None
    banner_image_data_url_medium: Optional[str] = None
    banner_image_content_type_medium: Optional[str] = None
    banner_image_data_url_high: Optional[str] = None
    banner_image_content_type_high: Optional[str] = None
    icon_image_data_url_low: Optional[str] = None
    icon_image_content_type_low: Optional[str] = None
    icon_image_data_url_medium: Optional[str] = None
    icon_image_content_type_medium: Optional[str] = None
    icon_image_data_url_high: Optional[str] = None
    icon_image_content_type_high: Optional[str] = None

    def to_dict(self) -> dict:
        payload = {
            "name": self.name,
            "slug": self.slug,
            "description": self.description,
            "definition": self.definition,
            "banner_image_url_low": self.banner_image_url_low,
            "banner_image_url_medium": self.banner_image_url_medium,
            "banner_image_url_high": self.banner_image_url_high,
            "icon_url_low": self.icon_url_low,
            "icon_url_medium": self.icon_url_medium,
            "icon_url_high": self.icon_url_high,
            "category": self.category,
            "subcategory": self.subcategory,
            "tags": self.tags,
            "featured_rank": self.featured_rank,
            "banner_image_data_url_low": self.banner_image_data_url_low,
            "banner_image_content_type_low": self.banner_image_content_type_low,
            "banner_image_data_url_medium": self.banner_image_data_url_medium,
            "banner_image_content_type_medium": self.banner_image_content_type_medium,
            "banner_image_data_url_high": self.banner_image_data_url_high,
            "banner_image_content_type_high": self.banner_image_content_type_high,
            "icon_image_data_url_low": self.icon_image_data_url_low,
            "icon_image_content_type_low": self.icon_image_content_type_low,
            "icon_image_data_url_medium": self.icon_image_data_url_medium,
            "icon_image_content_type_medium": self.icon_image_content_type_medium,
            "icon_image_data_url_high": self.icon_image_data_url_high,
            "icon_image_content_type_high": self.icon_image_content_type_high,
        }
        return _compact_dict(payload)


@dataclass
class MarketDeploymentOutcome:
    index: int
    name: str
    symbol: str
    description: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    icon_image_data_url_low: Optional[str] = None
    icon_image_content_type_low: Optional[str] = None
    icon_image_data_url_medium: Optional[str] = None
    icon_image_content_type_medium: Optional[str] = None
    icon_image_data_url_high: Optional[str] = None
    icon_image_content_type_high: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class MarketDeploymentDepositAsset:
    mint: str
    display_name: str
    symbol: str
    decimals: int
    description: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class MarketDeploymentConditionalToken:
    outcome_index: int
    deposit_mint: str
    conditional_mint: str
    name: str
    symbol: str
    image_data_url_high: str
    image_content_type_high: str
    description: Optional[str] = None
    image_data_url_low: Optional[str] = None
    image_content_type_low: Optional[str] = None
    image_data_url_medium: Optional[str] = None
    image_content_type_medium: Optional[str] = None

    def to_dict(self) -> dict:
        return _compact_dict(self.__dict__)


@dataclass
class UploadMarketDeploymentAssetsRequest:
    """Request payload for POST /api/admin/metadata/upload-market-deployment-assets."""

    market_id: int
    market_pubkey: str
    market: MarketDeploymentMarket
    outcomes: list[MarketDeploymentOutcome] = field(default_factory=list)
    deposit_assets: list[MarketDeploymentDepositAsset] = field(default_factory=list)
    conditional_tokens: list[MarketDeploymentConditionalToken] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "market_id": self.market_id,
            "market_pubkey": self.market_pubkey,
            "market": self.market.to_dict(),
            "outcomes": [outcome.to_dict() for outcome in self.outcomes],
            "deposit_assets": [asset.to_dict() for asset in self.deposit_assets],
            "conditional_tokens": [
                token.to_dict() for token in self.conditional_tokens
            ],
        }


@dataclass(frozen=True)
class UploadedMarketImages:
    banner_image_url_low: Optional[str] = None
    banner_image_url_medium: Optional[str] = None
    banner_image_url_high: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "UploadedMarketImages":
        return UploadedMarketImages(
            banner_image_url_low=d.get("banner_image_url_low"),
            banner_image_url_medium=d.get("banner_image_url_medium"),
            banner_image_url_high=d.get("banner_image_url_high"),
            icon_url_low=d.get("icon_url_low"),
            icon_url_medium=d.get("icon_url_medium"),
            icon_url_high=d.get("icon_url_high"),
        )


@dataclass(frozen=True)
class UploadedOutcomeImages:
    index: int
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "UploadedOutcomeImages":
        return UploadedOutcomeImages(
            index=d.get("index", 0),
            icon_url_low=d.get("icon_url_low"),
            icon_url_medium=d.get("icon_url_medium"),
            icon_url_high=d.get("icon_url_high"),
        )


@dataclass(frozen=True)
class UploadedDepositAssetImages:
    mint: str
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "UploadedDepositAssetImages":
        return UploadedDepositAssetImages(
            mint=d.get("mint", ""),
            icon_url_low=d.get("icon_url_low"),
            icon_url_medium=d.get("icon_url_medium"),
            icon_url_high=d.get("icon_url_high"),
        )


@dataclass(frozen=True)
class UploadedConditionalToken:
    conditional_mint: str
    metadata_uri: str
    image_url_low: Optional[str] = None
    image_url_medium: Optional[str] = None
    image_url_high: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "UploadedConditionalToken":
        return UploadedConditionalToken(
            conditional_mint=d.get("conditional_mint", ""),
            metadata_uri=d.get("metadata_uri", ""),
            image_url_low=d.get("image_url_low"),
            image_url_medium=d.get("image_url_medium"),
            image_url_high=d.get("image_url_high"),
        )


@dataclass
class UploadMarketDeploymentAssetsResponse:
    """Response from POST /api/admin/metadata/upload-market-deployment-assets."""

    market_metadata_uri: str = ""
    market: UploadedMarketImages = field(default_factory=UploadedMarketImages)
    outcomes: list[UploadedOutcomeImages] = field(default_factory=list)
    deposit_assets: list[UploadedDepositAssetImages] = field(default_factory=list)
    tokens: list[UploadedConditionalToken] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UploadMarketDeploymentAssetsResponse":
        return UploadMarketDeploymentAssetsResponse(
            market_metadata_uri=d.get("market_metadata_uri", ""),
            market=UploadedMarketImages.from_dict(d.get("market") or {}),
            outcomes=[
                UploadedOutcomeImages.from_dict(outcome)
                for outcome in d.get("outcomes", [])
            ],
            deposit_assets=[
                UploadedDepositAssetImages.from_dict(asset)
                for asset in d.get("deposit_assets", [])
            ],
            tokens=[
                UploadedConditionalToken.from_dict(token)
                for token in d.get("tokens", [])
            ],
        )


__all__ = [
    "AdminNonceResponse",
    "AdminLoginRequest",
    "AdminLoginResponse",
    "TargetSpec",
    "MarketMetadataPayload",
    "OutcomeMetadataPayload",
    "ConditionalTokenMetadataPayload",
    "DepositTokenMetadataPayload",
    "DepositTokenMetadataResponse",
    "UnifiedMetadataRequest",
    "UnifiedMetadataResponse",
    "MetadataCategoriesResponse",
    "AddMetadataCategoryRequest",
    "AddMetadataCategoryResponse",
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
    "AdminMarketStatusFilter",
    "AdminMarketStatus",
    "AdminMarketsRangeValue",
    "AdminMarketsQuery",
    "AdminMarketRow",
    "AdminMarketsResponse",
    "MarketsToSettleCountResponse",
    "MarketsToSettleQuery",
    "MarketsToSettleResponse",
    "ReferralConfig",
    "UpdateConfigRequest",
    "ListCodesRequest",
    "ListCodesResponse",
    "CodeListEntry",
    "UpdateCodeRequest",
    "UpdateCodeResponse",
    "AdminLogEventsQuery",
    "AdminLogEventsResponse",
    "AdminLogEvent",
    "AdminLogMetricsQuery",
    "AdminLogMetricsResponse",
    "AdminLogMetricBreakdown",
    "AdminLogMetricSummary",
    "AdminLogMetricHistoryQuery",
    "AdminLogMetricHistoryResponse",
    "AdminLogMetricPoint",
    "CriticalLogErrors24hCountResponse",
    "MarketDeploymentMarket",
    "MarketDeploymentOutcome",
    "MarketDeploymentDepositAsset",
    "MarketDeploymentConditionalToken",
    "UploadMarketDeploymentAssetsRequest",
    "UploadMarketDeploymentAssetsResponse",
    "UploadedMarketImages",
    "UploadedOutcomeImages",
    "UploadedDepositAssetImages",
    "UploadedConditionalToken",
]
