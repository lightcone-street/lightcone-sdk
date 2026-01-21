"""Market-related types for the Lightcone REST API."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class ApiMarketStatus(Enum):
    """Market status enum matching the API specification."""

    PENDING = "Pending"
    ACTIVE = "Active"
    SETTLED = "Settled"


@dataclass
class Outcome:
    """Outcome information for a market."""

    index: int
    name: str
    thumbnail_url: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "Outcome":
        return cls(
            index=data["index"],
            name=data["name"],
            thumbnail_url=data.get("thumbnail_url"),
        )


@dataclass
class OrderbookSummary:
    """Orderbook summary embedded in market response."""

    orderbook_id: str
    market_pubkey: str
    base_token: str
    quote_token: str
    tick_size: int
    created_at: str

    @classmethod
    def from_dict(cls, data: dict) -> "OrderbookSummary":
        return cls(
            orderbook_id=data["orderbook_id"],
            market_pubkey=data["market_pubkey"],
            base_token=data["base_token"],
            quote_token=data["quote_token"],
            tick_size=data["tick_size"],
            created_at=data["created_at"],
        )


@dataclass
class ConditionalToken:
    """Conditional token information."""

    id: int
    outcome_index: int
    token_address: str
    name: str
    symbol: str
    display_name: str
    outcome: str
    deposit_symbol: str
    short_name: str
    decimals: int
    created_at: str
    uri: Optional[str] = None
    description: Optional[str] = None
    icon_url: Optional[str] = None
    metadata_uri: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "ConditionalToken":
        return cls(
            id=data["id"],
            outcome_index=data["outcome_index"],
            token_address=data["token_address"],
            name=data["name"],
            symbol=data["symbol"],
            display_name=data["display_name"],
            outcome=data["outcome"],
            deposit_symbol=data["deposit_symbol"],
            short_name=data["short_name"],
            decimals=data["decimals"],
            created_at=data["created_at"],
            uri=data.get("uri"),
            description=data.get("description"),
            icon_url=data.get("icon_url"),
            metadata_uri=data.get("metadata_uri"),
        )


@dataclass
class DepositAsset:
    """Deposit asset information."""

    display_name: str
    token_symbol: str
    symbol: str
    deposit_asset: str
    id: int
    market_pubkey: str
    vault: str
    num_outcomes: int
    decimals: int
    conditional_tokens: list[ConditionalToken]
    created_at: str
    description: Optional[str] = None
    icon_url: Optional[str] = None
    metadata_uri: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "DepositAsset":
        return cls(
            display_name=data["display_name"],
            token_symbol=data["token_symbol"],
            symbol=data["symbol"],
            deposit_asset=data["deposit_asset"],
            id=data["id"],
            market_pubkey=data["market_pubkey"],
            vault=data["vault"],
            num_outcomes=data["num_outcomes"],
            decimals=data["decimals"],
            conditional_tokens=[
                ConditionalToken.from_dict(ct) for ct in data.get("conditional_tokens", [])
            ],
            created_at=data["created_at"],
            description=data.get("description"),
            icon_url=data.get("icon_url"),
            metadata_uri=data.get("metadata_uri"),
        )


@dataclass
class Market:
    """Market information."""

    market_name: str
    slug: str
    description: str
    definition: str
    outcomes: list[Outcome]
    market_pubkey: str
    market_id: int
    oracle: str
    question_id: str
    condition_id: str
    market_status: ApiMarketStatus
    created_at: str
    banner_image_url: Optional[str] = None
    thumbnail_url: Optional[str] = None
    category: Optional[str] = None
    tags: list[str] = field(default_factory=list)
    featured_rank: int = 0
    winning_outcome: int = 0
    has_winning_outcome: bool = False
    activated_at: Optional[str] = None
    settled_at: Optional[str] = None
    deposit_assets: list[DepositAsset] = field(default_factory=list)
    orderbooks: list[OrderbookSummary] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: dict) -> "Market":
        status_str = data.get("market_status", "Pending")
        try:
            status = ApiMarketStatus(status_str)
        except ValueError:
            status = ApiMarketStatus.PENDING

        return cls(
            market_name=data["market_name"],
            slug=data["slug"],
            description=data["description"],
            definition=data["definition"],
            outcomes=[Outcome.from_dict(o) for o in data.get("outcomes", [])],
            market_pubkey=data["market_pubkey"],
            market_id=data["market_id"],
            oracle=data["oracle"],
            question_id=data["question_id"],
            condition_id=data["condition_id"],
            market_status=status,
            created_at=data["created_at"],
            banner_image_url=data.get("banner_image_url"),
            thumbnail_url=data.get("thumbnail_url"),
            category=data.get("category"),
            tags=data.get("tags", []),
            featured_rank=data.get("featured_rank", 0),
            winning_outcome=data.get("winning_outcome", 0),
            has_winning_outcome=data.get("has_winning_outcome", False),
            activated_at=data.get("activated_at"),
            settled_at=data.get("settled_at"),
            deposit_assets=[
                DepositAsset.from_dict(da) for da in data.get("deposit_assets", [])
            ],
            orderbooks=[
                OrderbookSummary.from_dict(ob) for ob in data.get("orderbooks", [])
            ],
        )


@dataclass
class MarketsResponse:
    """Response for GET /api/markets."""

    markets: list[Market]
    total: int

    @classmethod
    def from_dict(cls, data: dict) -> "MarketsResponse":
        return cls(
            markets=[Market.from_dict(m) for m in data.get("markets", [])],
            total=data.get("total", 0),
        )


@dataclass
class MarketInfoResponse:
    """Response for GET /api/markets/{market_pubkey}."""

    market: Market
    deposit_assets: list[DepositAsset]
    deposit_asset_count: int

    @classmethod
    def from_dict(cls, data: dict) -> "MarketInfoResponse":
        return cls(
            market=Market.from_dict(data["market"]),
            deposit_assets=[
                DepositAsset.from_dict(da) for da in data.get("deposit_assets", [])
            ],
            deposit_asset_count=data.get("deposit_asset_count", 0),
        )


@dataclass
class DepositAssetsResponse:
    """Response for GET /api/markets/{market_pubkey}/deposit-assets."""

    market_pubkey: str
    deposit_assets: list[DepositAsset]
    total: int

    @classmethod
    def from_dict(cls, data: dict) -> "DepositAssetsResponse":
        return cls(
            market_pubkey=data["market_pubkey"],
            deposit_assets=[
                DepositAsset.from_dict(da) for da in data.get("deposit_assets", [])
            ],
            total=data.get("total", 0),
        )
