"""Market-related types for the Lightcone REST API."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

from ..error import DeserializeError


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
        try:
            return cls(
                index=data["index"],
                name=data["name"],
                thumbnail_url=data.get("thumbnail_url"),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in Outcome: {e}")


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
        try:
            return cls(
                orderbook_id=data["orderbook_id"],
                market_pubkey=data["market_pubkey"],
                base_token=data["base_token"],
                quote_token=data["quote_token"],
                tick_size=data["tick_size"],
                created_at=data["created_at"],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in OrderbookSummary: {e}")


@dataclass
class ConditionalToken:
    """Conditional token information."""

    id: int
    outcome_index: int
    token_address: str
    decimals: int
    created_at: str
    name: Optional[str] = None
    symbol: Optional[str] = None
    display_name: Optional[str] = None
    outcome: Optional[str] = None
    deposit_symbol: Optional[str] = None
    short_name: Optional[str] = None
    uri: Optional[str] = None
    description: Optional[str] = None
    icon_url: Optional[str] = None
    metadata_uri: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "ConditionalToken":
        try:
            return cls(
                id=data["id"],
                outcome_index=data["outcome_index"],
                token_address=data["token_address"],
                decimals=data["decimals"],
                created_at=data["created_at"],
                name=data.get("name"),
                symbol=data.get("symbol"),
                display_name=data.get("display_name"),
                outcome=data.get("outcome"),
                deposit_symbol=data.get("deposit_symbol"),
                short_name=data.get("short_name"),
                uri=data.get("uri"),
                description=data.get("description"),
                icon_url=data.get("icon_url"),
                metadata_uri=data.get("metadata_uri"),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in ConditionalToken: {e}")


@dataclass
class DepositAsset:
    """Deposit asset information."""

    deposit_asset: str
    id: int
    market_pubkey: str
    vault: str
    num_outcomes: int
    decimals: int
    conditional_tokens: list[ConditionalToken]
    created_at: str
    display_name: Optional[str] = None
    token_symbol: Optional[str] = None
    symbol: Optional[str] = None
    description: Optional[str] = None
    icon_url: Optional[str] = None
    metadata_uri: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "DepositAsset":
        try:
            return cls(
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
                display_name=data.get("display_name"),
                token_symbol=data.get("token_symbol"),
                symbol=data.get("symbol"),
                description=data.get("description"),
                icon_url=data.get("icon_url"),
                metadata_uri=data.get("metadata_uri"),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in DepositAsset: {e}")


@dataclass
class Market:
    """Market information."""

    outcomes: list[Outcome]
    market_pubkey: str
    market_id: int
    oracle: str
    question_id: str
    condition_id: str
    market_status: ApiMarketStatus
    created_at: str
    market_name: Optional[str] = None
    slug: Optional[str] = None
    description: Optional[str] = None
    definition: Optional[str] = None
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
        try:
            status_str = data.get("market_status", "Pending")
            try:
                status = ApiMarketStatus(status_str)
            except ValueError:
                status = ApiMarketStatus.PENDING

            return cls(
                outcomes=[Outcome.from_dict(o) for o in data.get("outcomes", [])],
                market_pubkey=data["market_pubkey"],
                market_id=data["market_id"],
                oracle=data["oracle"],
                question_id=data["question_id"],
                condition_id=data["condition_id"],
                market_status=status,
                created_at=data["created_at"],
                market_name=data.get("market_name"),
                slug=data.get("slug"),
                description=data.get("description"),
                definition=data.get("definition"),
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
        except KeyError as e:
            raise DeserializeError(f"Missing required field in Market: {e}")


@dataclass
class MarketsResponse:
    """Response for GET /api/markets."""

    markets: list[Market]
    total: int

    @classmethod
    def from_dict(cls, data: dict) -> "MarketsResponse":
        try:
            return cls(
                markets=[Market.from_dict(m) for m in data.get("markets", [])],
                total=data.get("total", 0),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in MarketsResponse: {e}")


@dataclass
class MarketInfoResponse:
    """Response for GET /api/markets/{market_pubkey}."""

    market: Market
    deposit_assets: list[DepositAsset]
    deposit_asset_count: int

    @classmethod
    def from_dict(cls, data: dict) -> "MarketInfoResponse":
        try:
            return cls(
                market=Market.from_dict(data["market"]),
                deposit_assets=[
                    DepositAsset.from_dict(da) for da in data.get("deposit_assets", [])
                ],
                deposit_asset_count=data.get("deposit_asset_count", 0),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in MarketInfoResponse: {e}")


@dataclass
class DepositAssetsResponse:
    """Response for GET /api/markets/{market_pubkey}/deposit-assets."""

    market_pubkey: str
    deposit_assets: list[DepositAsset]
    total: int

    @classmethod
    def from_dict(cls, data: dict) -> "DepositAssetsResponse":
        try:
            return cls(
                market_pubkey=data["market_pubkey"],
                deposit_assets=[
                    DepositAsset.from_dict(da) for da in data.get("deposit_assets", [])
                ],
                total=data.get("total", 0),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in DepositAssetsResponse: {e}")
