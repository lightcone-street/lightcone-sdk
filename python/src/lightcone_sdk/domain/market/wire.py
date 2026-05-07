"""Market wire types - raw API response shapes."""

from dataclasses import dataclass, field
from typing import Optional

from ...error import _require


@dataclass
class OutcomeWire:
    """Raw outcome from the API."""
    index: int = 0
    name: str = ""
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""

    @staticmethod
    def from_dict(d: dict, fallback_index: int = 0) -> "OutcomeWire":
        return OutcomeWire(
            index=d.get("index", fallback_index),
            name=d.get("name", ""),
            icon_url_low=d.get("icon_url_low", ""),
            icon_url_medium=d.get("icon_url_medium", ""),
            icon_url_high=d.get("icon_url_high", ""),
        )


@dataclass
class ConditionalMintWire:
    """Raw conditional mint nested inside a deposit asset."""
    id: int = 0
    token_address: str = ""
    outcome_index: int = 0
    outcome: str = ""
    short_symbol: str = ""
    symbol: str = ""
    description: Optional[str] = None
    decimals: int = 6
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""

    @staticmethod
    def from_dict(d: dict) -> "ConditionalMintWire":
        return ConditionalMintWire(
            id=d.get("id", 0),
            token_address=d.get("token_address", ""),
            outcome_index=d.get("outcome_index", 0),
            outcome=d.get("outcome", ""),
            short_symbol=d.get("short_symbol", ""),
            symbol=d.get("symbol", ""),
            description=d.get("description"),
            decimals=d.get("decimals") or 6,
            icon_url_low=d.get("icon_url_low", ""),
            icon_url_medium=d.get("icon_url_medium", ""),
            icon_url_high=d.get("icon_url_high", ""),
        )


@dataclass
class DepositAssetWire:
    """Raw deposit asset from the API."""
    id: int = 0
    market_pubkey: str = ""
    deposit_asset: str = ""
    num_outcomes: int = 0
    display_name: str = ""
    token_symbol: str = ""
    symbol: str = ""
    description: Optional[str] = None
    decimals: int = 6
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""
    conditional_mints: list[ConditionalMintWire] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "DepositAssetWire":
        return DepositAssetWire(
            id=d.get("id", 0),
            market_pubkey=d.get("market_pubkey", ""),
            deposit_asset=d.get("deposit_asset", ""),
            num_outcomes=d.get("num_outcomes", 0),
            display_name=d.get("display_name", ""),
            token_symbol=d.get("token_symbol", ""),
            symbol=d.get("symbol", ""),
            description=d.get("description"),
            decimals=d.get("decimals") or 6,
            icon_url_low=d.get("icon_url_low", ""),
            icon_url_medium=d.get("icon_url_medium", ""),
            icon_url_high=d.get("icon_url_high", ""),
            conditional_mints=[
                ConditionalMintWire.from_dict(cm)
                for cm in d.get("conditional_mints", [])
            ],
        )


@dataclass
class OrderbookWire:
    """Raw orderbook nested inside a market response."""
    id: int = 0
    market_pubkey: str = ""
    orderbook_id: str = ""
    base_token: str = ""
    quote_token: str = ""
    outcome_index: int = 0
    tick_size: int = 0
    total_bids: int = 0
    total_asks: int = 0
    last_trade_price: Optional[str] = None
    last_trade_time: Optional[str] = None
    active: bool = True

    @staticmethod
    def from_dict(d: dict) -> "OrderbookWire":
        return OrderbookWire(
            id=d.get("id", 0),
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            base_token=d.get("base_token", ""),
            quote_token=d.get("quote_token", ""),
            outcome_index=d.get("outcome_index", 0),
            tick_size=d.get("tick_size", 0),
            total_bids=d.get("total_bids", 0),
            total_asks=d.get("total_asks", 0),
            last_trade_price=d.get("last_trade_price"),
            last_trade_time=d.get("last_trade_time"),
            active=d.get("active", True),
        )


@dataclass
class SearchOrderbook:
    """Minimal orderbook info returned from search endpoints."""
    orderbook_id: str = ""
    outcome_name: str = ""
    outcome_index: int = 0
    deposit_base_asset: str = ""
    deposit_quote_asset: str = ""
    deposit_base_symbol: str = ""
    deposit_quote_symbol: str = ""
    base_icon_url_low: str = ""
    base_icon_url_medium: str = ""
    base_icon_url_high: str = ""
    quote_icon_url_low: str = ""
    quote_icon_url_medium: str = ""
    quote_icon_url_high: str = ""
    conditional_base_mint: str = ""
    conditional_quote_mint: str = ""
    outcome_icon_url_low: Optional[str] = None
    outcome_icon_url_medium: Optional[str] = None
    outcome_icon_url_high: Optional[str] = None
    conditional_base_symbol: Optional[str] = None
    conditional_quote_symbol: Optional[str] = None
    latest_mid_price: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "SearchOrderbook":
        return SearchOrderbook(
            orderbook_id=d.get("orderbook_id", ""),
            outcome_name=d.get("outcome_name", ""),
            outcome_index=d.get("outcome_index", 0),
            deposit_base_asset=d.get("deposit_base_asset", ""),
            deposit_quote_asset=d.get("deposit_quote_asset", ""),
            deposit_base_symbol=d.get("deposit_base_symbol", ""),
            deposit_quote_symbol=d.get("deposit_quote_symbol", ""),
            base_icon_url_low=d.get("base_icon_url_low", ""),
            base_icon_url_medium=d.get("base_icon_url_medium", ""),
            base_icon_url_high=d.get("base_icon_url_high", ""),
            quote_icon_url_low=d.get("quote_icon_url_low", ""),
            quote_icon_url_medium=d.get("quote_icon_url_medium", ""),
            quote_icon_url_high=d.get("quote_icon_url_high", ""),
            conditional_base_mint=d.get("conditional_base_mint", ""),
            conditional_quote_mint=d.get("conditional_quote_mint", ""),
            outcome_icon_url_low=d.get("outcome_icon_url_low"),
            outcome_icon_url_medium=d.get("outcome_icon_url_medium"),
            outcome_icon_url_high=d.get("outcome_icon_url_high"),
            conditional_base_symbol=d.get("conditional_base_symbol"),
            conditional_quote_symbol=d.get("conditional_quote_symbol"),
            latest_mid_price=d.get("latest_mid_price"),
        )


@dataclass
class MarketWire:
    """Raw market data from the API."""
    market_id: int = 0
    market_pubkey: str = ""
    market_name: str = ""
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
    tags: list[str] = field(default_factory=list)
    featured_rank: Optional[int] = None
    market_status: Optional[str] = None
    winning_outcome: Optional[int] = None
    has_winning_outcome: bool = False
    volume: Optional[str] = None
    created_at: Optional[str] = None
    activated_at: Optional[str] = None
    settled_at: Optional[str] = None
    outcomes: list[OutcomeWire] = field(default_factory=list)
    deposit_assets: list[DepositAssetWire] = field(default_factory=list)
    orderbooks: list[OrderbookWire] = field(default_factory=list)
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
            banner_image_url_low=d.get("banner_image_url_low"),
            banner_image_url_medium=d.get("banner_image_url_medium"),
            banner_image_url_high=d.get("banner_image_url_high"),
            icon_url_low=d.get("icon_url_low"),
            icon_url_medium=d.get("icon_url_medium"),
            icon_url_high=d.get("icon_url_high"),
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
            outcomes=[
                OutcomeWire.from_dict(o, fallback_index=i)
                for i, o in enumerate(d.get("outcomes", []))
            ],
            deposit_assets=[
                DepositAssetWire.from_dict(da)
                for da in d.get("deposit_assets", [])
            ],
            orderbooks=[
                OrderbookWire.from_dict(ob)
                for ob in d.get("orderbooks", [])
            ],
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
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    orderbooks: list[SearchOrderbook] = field(default_factory=list)

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
            icon_url_low=d.get("icon_url_low"),
            icon_url_medium=d.get("icon_url_medium"),
            icon_url_high=d.get("icon_url_high"),
            orderbooks=[
                SearchOrderbook.from_dict(ob)
                for ob in d.get("orderbooks", [])
            ],
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


@dataclass
class GlobalDepositAssetWire:
    """Raw globally whitelisted deposit asset from the API."""
    id: int = 0
    mint: str = ""
    display_name: Optional[str] = None
    symbol: Optional[str] = None
    description: Optional[str] = None
    icon_url_low: Optional[str] = None
    icon_url_medium: Optional[str] = None
    icon_url_high: Optional[str] = None
    decimals: Optional[int] = None
    whitelist_index: int = 0
    active: bool = False

    @staticmethod
    def from_dict(d: dict) -> "GlobalDepositAssetWire":
        return GlobalDepositAssetWire(
            id=d.get("id", 0),
            mint=_require(d, "mint", "GlobalDepositAssetWire"),
            display_name=d.get("display_name"),
            symbol=d.get("symbol"),
            description=d.get("description"),
            icon_url_low=d.get("icon_url_low"),
            icon_url_medium=d.get("icon_url_medium"),
            icon_url_high=d.get("icon_url_high"),
            decimals=d.get("decimals"),
            whitelist_index=d.get("whitelist_index", 0),
            active=d.get("active", False),
        )


@dataclass
class DepositMintsResponse:
    """API response for GET /api/markets/{market_pubkey}/deposit-assets."""

    market_pubkey: str = ""
    deposit_assets: list[DepositAssetWire] = field(default_factory=list)
    total: int = 0

    @staticmethod
    def from_dict(d: dict) -> "DepositMintsResponse":
        return DepositMintsResponse(
            market_pubkey=d.get("market_pubkey", ""),
            deposit_assets=[
                DepositAssetWire.from_dict(a)
                for a in d.get("deposit_assets", [])
            ],
            total=d.get("total", 0),
        )


@dataclass
class GlobalDepositAssetsListWire:
    """API response envelope for the global deposit asset whitelist."""
    assets: list[GlobalDepositAssetWire] = field(default_factory=list)
    total: int = 0

    @staticmethod
    def from_dict(d: dict) -> "GlobalDepositAssetsListWire":
        return GlobalDepositAssetsListWire(
            assets=[
                GlobalDepositAssetWire.from_dict(a)
                for a in d.get("assets", [])
            ],
            total=d.get("total", 0),
        )
