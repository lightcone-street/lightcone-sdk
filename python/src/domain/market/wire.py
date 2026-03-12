"""Market wire types - raw API response shapes."""

from dataclasses import dataclass, field
from typing import Optional

from ...error import _require


@dataclass
class MarketWire:
    """Raw market data from the API."""
    market_id: int = 0
    market_pubkey: str = ""
    market_name: str = ""
    slug: Optional[str] = None
    description: Optional[str] = None
    definition: Optional[str] = None
    banner_image_url: Optional[str] = None
    icon_url: Optional[str] = None
    category: Optional[str] = None
    tags: list[str] = field(default_factory=list)
    featured_rank: Optional[int] = None
    market_status: Optional[str] = None
    winning_outcome: Optional[int] = None
    has_winning_outcome: bool = False
    volume: Optional[str] = None
    created_at: Optional[str] = None
    activated_at: Optional[str] = None
    settled_at: Optional[str] = None
    outcomes: list[dict] = field(default_factory=list)
    deposit_assets: list[dict] = field(default_factory=list)
    orderbooks: list[dict] = field(default_factory=list)
    # Derived from wire
    oracle: Optional[str] = None
    question_id: Optional[str] = None
    condition_id: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "MarketWire":
        return MarketWire(
            market_id=d.get("market_id", 0),
            market_pubkey=_require(d, "market_pubkey", "MarketWire"),
            market_name=d.get("market_name", ""),
            slug=d.get("slug"),
            description=d.get("description"),
            definition=d.get("definition"),
            banner_image_url=d.get("banner_image_url"),
            icon_url=d.get("icon_url"),
            category=d.get("category"),
            tags=d.get("tags") or [],
            featured_rank=d.get("featured_rank"),
            market_status=d.get("market_status"),
            winning_outcome=d.get("winning_outcome"),
            has_winning_outcome=d.get("has_winning_outcome", False),
            volume=d.get("volume"),
            created_at=d.get("created_at"),
            activated_at=d.get("activated_at"),
            settled_at=d.get("settled_at"),
            outcomes=d.get("outcomes", []),
            deposit_assets=d.get("deposit_assets", []),
            orderbooks=d.get("orderbooks", []),
            oracle=d.get("oracle"),
            question_id=d.get("question_id"),
            condition_id=d.get("condition_id"),
        )


@dataclass
class MarketResponse:
    """API response for market list."""
    markets: list[MarketWire] = field(default_factory=list)
    next_cursor: Optional[int] = None
    has_more: bool = False

    @staticmethod
    def from_dict(d: dict) -> "MarketResponse":
        return MarketResponse(
            markets=[MarketWire.from_dict(m) for m in d.get("markets", [])],
            next_cursor=d.get("next_cursor"),
            has_more=d.get("has_more", False),
        )


@dataclass
class MarketSearchResult:
    """Search result from market search endpoints."""
    slug: str = ""
    market_name: str = ""
    market_status: Optional[str] = None
    category: Optional[str] = None
    tags: list[str] = field(default_factory=list)
    featured_rank: int = 0
    description: Optional[str] = None
    icon_url: Optional[str] = None
    orderbooks: list[dict] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "MarketSearchResult":
        return MarketSearchResult(
            slug=d.get("slug", ""),
            market_name=d.get("market_name", ""),
            market_status=d.get("market_status"),
            category=d.get("category"),
            tags=d.get("tags") or [],
            featured_rank=d.get("featured_rank", 0),
            description=d.get("description"),
            icon_url=d.get("icon_url"),
            orderbooks=d.get("orderbooks", []),
        )


@dataclass
class MarketEvent:
    """WebSocket market event."""
    event_type: str = ""
    market_pubkey: str = ""
    status: Optional[str] = None
    winning_outcome: Optional[int] = None
    orderbook_id: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "MarketEvent":
        return MarketEvent(
            event_type=d.get("event_type", ""),
            market_pubkey=d.get("market_pubkey", ""),
            status=d.get("status"),
            winning_outcome=d.get("winning_outcome"),
            orderbook_id=d.get("orderbook_id"),
        )
