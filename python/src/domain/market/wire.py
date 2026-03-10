"""Market wire types - raw API response shapes."""

from dataclasses import dataclass, field
from typing import Any, Optional


@dataclass
class MarketWire:
    """Raw market data from the API."""
    id: str
    pubkey: str
    name: str
    slug: Optional[str] = None
    description: Optional[str] = None
    status: Optional[str] = None
    volume: Optional[str] = None
    outcomes: list[dict] = field(default_factory=list)
    conditional_tokens: list[dict] = field(default_factory=list)
    deposit_assets: list[dict] = field(default_factory=list)
    orderbook_pairs: list[dict] = field(default_factory=list)
    token_metadata: list[dict] = field(default_factory=list)
    icon_url: Optional[str] = None
    category: Optional[str] = None
    featured: bool = False
    created_at: Optional[str] = None
    resolved_at: Optional[str] = None
    winning_outcome: Optional[int] = None

    @staticmethod
    def from_dict(d: dict) -> "MarketWire":
        return MarketWire(
            id=d.get("id", ""),
            pubkey=d.get("pubkey", ""),
            name=d.get("name", ""),
            slug=d.get("slug"),
            description=d.get("description"),
            status=d.get("status"),
            volume=d.get("volume"),
            outcomes=d.get("outcomes", []),
            conditional_tokens=d.get("conditional_tokens", []),
            deposit_assets=d.get("deposit_assets", []),
            orderbook_pairs=d.get("orderbook_pairs", []),
            token_metadata=d.get("token_metadata", []),
            icon_url=d.get("icon_url"),
            category=d.get("category"),
            featured=d.get("featured", False),
            created_at=d.get("created_at"),
            resolved_at=d.get("resolved_at"),
            winning_outcome=d.get("winning_outcome"),
        )


@dataclass
class MarketResponse:
    """API response for market list."""
    markets: list[MarketWire] = field(default_factory=list)
    total: int = 0
    next_cursor: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "MarketResponse":
        return MarketResponse(
            markets=[MarketWire.from_dict(m) for m in d.get("markets", [])],
            total=d.get("total", 0),
            next_cursor=d.get("next_cursor"),
        )


@dataclass
class MarketSearchResult:
    markets: list[MarketWire] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "MarketSearchResult":
        return MarketSearchResult(
            markets=[MarketWire.from_dict(m) for m in d.get("markets", [])],
        )


@dataclass
class MarketEvent:
    """WebSocket market event."""
    type: str
    market_pubkey: str
    data: Optional[dict] = None
