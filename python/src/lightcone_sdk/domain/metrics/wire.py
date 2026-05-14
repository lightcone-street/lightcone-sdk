"""Metrics wire types - mirror the backend's dto::metrics shapes.

Decimal-bearing fields are stored as `str` to match the existing SDK
convention (wire types in ``order/wire.py``, ``market/wire.py`` etc. also
use ``str`` for Decimal-valued fields). Consumers who need numeric math
can wrap a field in ``decimal.Decimal`` themselves.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Literal, Optional


def _str_list(raw: list) -> list[str]:
    return [str(x) for x in raw]


# ─── Deposit token ──────────────────────────────────────────────────────────


@dataclass
class DepositTokenVolumeMetrics:
    """Entry in /api/metrics/deposit-tokens; nested in platform/market/category."""

    deposit_asset: str = ""
    volume_24h_usd: str = "0"
    volume_7d_usd: str = "0"
    volume_30d_usd: str = "0"
    volume_total_usd: str = "0"
    taker_bid_volume_24h_usd: str = "0"
    taker_bid_volume_7d_usd: str = "0"
    taker_bid_volume_30d_usd: str = "0"
    taker_bid_volume_total_usd: str = "0"
    taker_ask_volume_24h_usd: str = "0"
    taker_ask_volume_7d_usd: str = "0"
    taker_ask_volume_30d_usd: str = "0"
    taker_ask_volume_total_usd: str = "0"
    taker_bid_ask_imbalance_24h_pct: str = "0"
    taker_bid_ask_imbalance_7d_pct: str = "0"
    taker_bid_ask_imbalance_30d_pct: str = "0"
    taker_bid_ask_imbalance_total_pct: str = "0"
    volume_share_24h_pct: str = "0"
    symbol: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "DepositTokenVolumeMetrics":
        return DepositTokenVolumeMetrics(
            deposit_asset=d.get("deposit_asset", ""),
            symbol=d.get("symbol"),
            volume_24h_usd=str(d.get("volume_24h_usd", "0")),
            volume_7d_usd=str(d.get("volume_7d_usd", "0")),
            volume_30d_usd=str(d.get("volume_30d_usd", "0")),
            volume_total_usd=str(d.get("volume_total_usd", "0")),
            taker_bid_volume_24h_usd=str(d.get("taker_bid_volume_24h_usd", "0")),
            taker_bid_volume_7d_usd=str(d.get("taker_bid_volume_7d_usd", "0")),
            taker_bid_volume_30d_usd=str(d.get("taker_bid_volume_30d_usd", "0")),
            taker_bid_volume_total_usd=str(d.get("taker_bid_volume_total_usd", "0")),
            taker_ask_volume_24h_usd=str(d.get("taker_ask_volume_24h_usd", "0")),
            taker_ask_volume_7d_usd=str(d.get("taker_ask_volume_7d_usd", "0")),
            taker_ask_volume_30d_usd=str(d.get("taker_ask_volume_30d_usd", "0")),
            taker_ask_volume_total_usd=str(d.get("taker_ask_volume_total_usd", "0")),
            taker_bid_ask_imbalance_24h_pct=str(
                d.get("taker_bid_ask_imbalance_24h_pct", "0")
            ),
            taker_bid_ask_imbalance_7d_pct=str(
                d.get("taker_bid_ask_imbalance_7d_pct", "0")
            ),
            taker_bid_ask_imbalance_30d_pct=str(
                d.get("taker_bid_ask_imbalance_30d_pct", "0")
            ),
            taker_bid_ask_imbalance_total_pct=str(
                d.get("taker_bid_ask_imbalance_total_pct", "0")
            ),
            volume_share_24h_pct=str(d.get("volume_share_24h_pct", "0")),
        )


@dataclass
class DepositTokensMetrics:
    """Envelope for /api/metrics/deposit-tokens."""

    deposit_tokens: list[DepositTokenVolumeMetrics] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "DepositTokensMetrics":
        return DepositTokensMetrics(
            deposit_tokens=[
                DepositTokenVolumeMetrics.from_dict(x)
                for x in d.get("deposit_tokens", [])
            ],
        )


@dataclass
class DepositTokenVolumeHistoryToken:
    """Token legend/summary entry in /api/metrics/deposit-tokens/volume-history."""

    rank: int = 0
    deposit_asset: str = ""
    volume_total_usd: str = "0"
    symbol: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "DepositTokenVolumeHistoryToken":
        return DepositTokenVolumeHistoryToken(
            rank=int(d.get("rank", 0)),
            deposit_asset=d.get("deposit_asset", ""),
            symbol=d.get("symbol"),
            volume_total_usd=str(d.get("volume_total_usd", "0")),
        )


@dataclass
class DepositTokenVolumeHistoryPointToken:
    """Per-token stacked-bar entry for a daily deposit-token history point."""

    deposit_asset: str = ""
    volume_usd: str = "0"
    symbol: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "DepositTokenVolumeHistoryPointToken":
        return DepositTokenVolumeHistoryPointToken(
            deposit_asset=d.get("deposit_asset", ""),
            symbol=d.get("symbol"),
            volume_usd=str(d.get("volume_usd", "0")),
        )


@dataclass
class DepositTokenVolumeHistoryPoint:
    """Daily point in /api/metrics/deposit-tokens/volume-history."""

    bucket_start: int = 0
    bucket_start_date: str = ""
    total_volume_usd: str = "0"
    cumulative_volume_usd: str = "0"
    deposit_token_volumes: list[DepositTokenVolumeHistoryPointToken] = field(
        default_factory=list
    )

    @staticmethod
    def from_dict(d: dict) -> "DepositTokenVolumeHistoryPoint":
        return DepositTokenVolumeHistoryPoint(
            bucket_start=int(d.get("bucket_start", 0)),
            bucket_start_date=d.get("bucket_start_date", ""),
            total_volume_usd=str(d.get("total_volume_usd", "0")),
            cumulative_volume_usd=str(d.get("cumulative_volume_usd", "0")),
            deposit_token_volumes=[
                DepositTokenVolumeHistoryPointToken.from_dict(x)
                for x in d.get("deposit_token_volumes", [])
            ],
        )


@dataclass
class DepositTokenVolumeHistory:
    """Response of /api/metrics/deposit-tokens/volume-history."""

    timestamp: int = 0
    resolution: str = ""
    from_ms: int = 0
    to_ms: int = 0
    volume_total_usd: str = "0"
    total_days: int = 0
    deposit_tokens: list[DepositTokenVolumeHistoryToken] = field(default_factory=list)
    points: list[DepositTokenVolumeHistoryPoint] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "DepositTokenVolumeHistory":
        return DepositTokenVolumeHistory(
            timestamp=int(d.get("timestamp", 0)),
            resolution=d.get("resolution", ""),
            from_ms=int(d.get("from", 0)),
            to_ms=int(d.get("to", 0)),
            volume_total_usd=str(d.get("volume_total_usd", "0")),
            total_days=int(d.get("total_days", 0)),
            deposit_tokens=[
                DepositTokenVolumeHistoryToken.from_dict(x)
                for x in d.get("deposit_tokens", [])
            ],
            points=[
                DepositTokenVolumeHistoryPoint.from_dict(x)
                for x in d.get("points", [])
            ],
        )


@dataclass
class OpenInterestHistoryDepositAsset:
    """Deposit-asset summary entry in /api/metrics/open-interest/history."""

    rank: int = 0
    deposit_asset: str = ""
    latest_open_interest_usd: str = "0"
    max_open_interest_usd: str = "0"
    symbol: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "OpenInterestHistoryDepositAsset":
        return OpenInterestHistoryDepositAsset(
            rank=int(d.get("rank", 0)),
            deposit_asset=d.get("deposit_asset", ""),
            symbol=d.get("symbol"),
            latest_open_interest_usd=str(d.get("latest_open_interest_usd", "0")),
            max_open_interest_usd=str(d.get("max_open_interest_usd", "0")),
        )


@dataclass
class OpenInterestHistoryPointDepositAsset:
    """Per-deposit-asset entry for one daily open-interest history point."""

    deposit_asset: str = ""
    open_interest_usd: str = "0"
    symbol: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "OpenInterestHistoryPointDepositAsset":
        return OpenInterestHistoryPointDepositAsset(
            deposit_asset=d.get("deposit_asset", ""),
            symbol=d.get("symbol"),
            open_interest_usd=str(d.get("open_interest_usd", "0")),
        )


@dataclass
class OpenInterestHistoryPoint:
    """Daily point in /api/metrics/open-interest/history."""

    bucket_start: int = 0
    bucket_start_date: str = ""
    total_open_interest_usd: str = "0"
    deposit_asset_open_interest: list[OpenInterestHistoryPointDepositAsset] = field(
        default_factory=list
    )

    @staticmethod
    def from_dict(d: dict) -> "OpenInterestHistoryPoint":
        return OpenInterestHistoryPoint(
            bucket_start=int(d.get("bucket_start", 0)),
            bucket_start_date=d.get("bucket_start_date", ""),
            total_open_interest_usd=str(d.get("total_open_interest_usd", "0")),
            deposit_asset_open_interest=[
                OpenInterestHistoryPointDepositAsset.from_dict(x)
                for x in d.get("deposit_asset_open_interest", [])
            ],
        )


@dataclass
class OpenInterestHistory:
    """Response of /api/metrics/open-interest/history."""

    timestamp: int = 0
    resolution: str = ""
    from_ms: int = 0
    to_ms: int = 0
    latest_open_interest_usd: str = "0"
    total_days: int = 0
    deposit_assets: list[OpenInterestHistoryDepositAsset] = field(default_factory=list)
    points: list[OpenInterestHistoryPoint] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "OpenInterestHistory":
        return OpenInterestHistory(
            timestamp=int(d.get("timestamp", 0)),
            resolution=d.get("resolution", ""),
            from_ms=int(d.get("from", 0)),
            to_ms=int(d.get("to", 0)),
            latest_open_interest_usd=str(d.get("latest_open_interest_usd", "0")),
            total_days=int(d.get("total_days", 0)),
            deposit_assets=[
                OpenInterestHistoryDepositAsset.from_dict(x)
                for x in d.get("deposit_assets", [])
            ],
            points=[
                OpenInterestHistoryPoint.from_dict(x)
                for x in d.get("points", [])
            ],
        )


UniqueTradersHistoryScope = Literal[
    "platform",
    "market",
    "orderbook",
    "category",
    "outcome",
]


@dataclass
class UniqueTradersHistoryPoint:
    """Daily point in /api/metrics/unique-traders/history."""

    bucket_start: int = 0
    bucket_start_date: str = ""
    unique_traders: int = 0

    @staticmethod
    def from_dict(d: dict) -> "UniqueTradersHistoryPoint":
        return UniqueTradersHistoryPoint(
            bucket_start=int(d.get("bucket_start", 0)),
            bucket_start_date=d.get("bucket_start_date", ""),
            unique_traders=int(d.get("unique_traders", 0)),
        )


@dataclass
class UniqueTradersHistory:
    """Response of /api/metrics/unique-traders/history."""

    timestamp: int = 0
    resolution: str = ""
    scope: UniqueTradersHistoryScope = "platform"
    scope_key: str = "platform"
    from_ms: int = 0
    to_ms: int = 0
    latest_unique_traders: int = 0
    total_days: int = 0
    points: list[UniqueTradersHistoryPoint] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "UniqueTradersHistory":
        return UniqueTradersHistory(
            timestamp=int(d.get("timestamp", 0)),
            resolution=d.get("resolution", ""),
            scope=d.get("scope", "platform"),
            scope_key=d.get("scope_key", "platform"),
            from_ms=int(d.get("from", 0)),
            to_ms=int(d.get("to", 0)),
            latest_unique_traders=int(d.get("latest_unique_traders", 0)),
            total_days=int(d.get("total_days", 0)),
            points=[
                UniqueTradersHistoryPoint.from_dict(x)
                for x in d.get("points", [])
            ],
        )


# ─── Orderbook tickers (batch) ───────────────────────────────────────────────


@dataclass
class OrderbookTickerEntry:
    """One entry in /api/metrics/orderbooks/tickers.

    Same shape (BBO + midpoint) as the WS ``Ticker`` stream, delivered in
    batch over REST. Price fields are ``None`` when the orderbook has no
    liquidity yet.
    """

    orderbook_id: str = ""
    market_pubkey: str = ""
    outcome_index: Optional[int] = None
    outcome_name: Optional[str] = None
    outcome_name_long: Optional[str] = None
    base_deposit_asset: str = ""
    quote_deposit_asset: str = ""
    best_bid: Optional[str] = None
    best_ask: Optional[str] = None
    midpoint: Optional[str] = None
    computed_at: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderbookTickerEntry":
        def _opt_str(key: str) -> Optional[str]:
            v = d.get(key)
            return None if v is None else str(v)

        def _opt_int(key: str) -> Optional[int]:
            v = d.get(key)
            return None if v is None else int(v)

        return OrderbookTickerEntry(
            orderbook_id=str(d.get("orderbook_id", "")),
            market_pubkey=str(d.get("market_pubkey", "")),
            outcome_index=_opt_int("outcome_index"),
            outcome_name=_opt_str("outcome_name"),
            outcome_name_long=_opt_str("outcome_name_long"),
            base_deposit_asset=str(d.get("base_deposit_asset", "")),
            quote_deposit_asset=str(d.get("quote_deposit_asset", "")),
            best_bid=_opt_str("best_bid"),
            best_ask=_opt_str("best_ask"),
            midpoint=_opt_str("midpoint"),
            computed_at=_opt_str("computed_at"),
        )


@dataclass
class OrderbookTickersResponse:
    """Response of /api/metrics/orderbooks/tickers."""

    tickers: list[OrderbookTickerEntry] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "OrderbookTickersResponse":
        return OrderbookTickersResponse(
            tickers=[
                OrderbookTickerEntry.from_dict(x) for x in d.get("tickers", [])
            ],
        )


# ─── Platform ────────────────────────────────────────────────────────────────


@dataclass
class PlatformMetrics:
    """Response of /api/metrics/platform."""

    volume_24h_usd: str = "0"
    volume_7d_usd: str = "0"
    volume_30d_usd: str = "0"
    volume_total_usd: str = "0"
    taker_bid_volume_24h_usd: str = "0"
    taker_bid_volume_7d_usd: str = "0"
    taker_bid_volume_30d_usd: str = "0"
    taker_bid_volume_total_usd: str = "0"
    taker_ask_volume_24h_usd: str = "0"
    taker_ask_volume_7d_usd: str = "0"
    taker_ask_volume_30d_usd: str = "0"
    taker_ask_volume_total_usd: str = "0"
    taker_bid_ask_imbalance_24h_pct: str = "0"
    taker_bid_ask_imbalance_7d_pct: str = "0"
    taker_bid_ask_imbalance_30d_pct: str = "0"
    taker_bid_ask_imbalance_total_pct: str = "0"
    open_interest_usd: str = "0"
    fees_24h_usd: str = "0"
    fees_7d_usd: str = "0"
    fees_30d_usd: str = "0"
    unique_traders_24h: int = 0
    unique_traders_7d: int = 0
    unique_traders_30d: int = 0
    active_markets: int = 0
    active_orderbooks: int = 0
    deposit_token_volumes: list[DepositTokenVolumeMetrics] = field(default_factory=list)
    updated_at: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "PlatformMetrics":
        return PlatformMetrics(
            volume_24h_usd=str(d.get("volume_24h_usd", "0")),
            volume_7d_usd=str(d.get("volume_7d_usd", "0")),
            volume_30d_usd=str(d.get("volume_30d_usd", "0")),
            volume_total_usd=str(d.get("volume_total_usd", "0")),
            taker_bid_volume_24h_usd=str(d.get("taker_bid_volume_24h_usd", "0")),
            taker_bid_volume_7d_usd=str(d.get("taker_bid_volume_7d_usd", "0")),
            taker_bid_volume_30d_usd=str(d.get("taker_bid_volume_30d_usd", "0")),
            taker_bid_volume_total_usd=str(d.get("taker_bid_volume_total_usd", "0")),
            taker_ask_volume_24h_usd=str(d.get("taker_ask_volume_24h_usd", "0")),
            taker_ask_volume_7d_usd=str(d.get("taker_ask_volume_7d_usd", "0")),
            taker_ask_volume_30d_usd=str(d.get("taker_ask_volume_30d_usd", "0")),
            taker_ask_volume_total_usd=str(d.get("taker_ask_volume_total_usd", "0")),
            taker_bid_ask_imbalance_24h_pct=str(
                d.get("taker_bid_ask_imbalance_24h_pct", "0")
            ),
            taker_bid_ask_imbalance_7d_pct=str(
                d.get("taker_bid_ask_imbalance_7d_pct", "0")
            ),
            taker_bid_ask_imbalance_30d_pct=str(
                d.get("taker_bid_ask_imbalance_30d_pct", "0")
            ),
            taker_bid_ask_imbalance_total_pct=str(
                d.get("taker_bid_ask_imbalance_total_pct", "0")
            ),
            open_interest_usd=str(d.get("open_interest_usd", "0")),
            fees_24h_usd=str(d.get("fees_24h_usd", "0")),
            fees_7d_usd=str(d.get("fees_7d_usd", "0")),
            fees_30d_usd=str(d.get("fees_30d_usd", "0")),
            unique_traders_24h=int(d.get("unique_traders_24h", 0)),
            unique_traders_7d=int(d.get("unique_traders_7d", 0)),
            unique_traders_30d=int(d.get("unique_traders_30d", 0)),
            active_markets=int(d.get("active_markets", 0)),
            active_orderbooks=int(d.get("active_orderbooks", 0)),
            deposit_token_volumes=[
                DepositTokenVolumeMetrics.from_dict(x)
                for x in d.get("deposit_token_volumes", [])
            ],
            updated_at=d.get("updated_at"),
        )


# ─── Market summary ──────────────────────────────────────────────────────────


@dataclass
class MarketVolumeMetrics:
    """Entry in /api/metrics/markets."""

    market_pubkey: str = ""
    volume_24h_usd: str = "0"
    volume_7d_usd: str = "0"
    volume_30d_usd: str = "0"
    volume_total_usd: str = "0"
    taker_bid_volume_24h_usd: str = "0"
    taker_bid_volume_7d_usd: str = "0"
    taker_bid_volume_30d_usd: str = "0"
    taker_bid_volume_total_usd: str = "0"
    taker_ask_volume_24h_usd: str = "0"
    taker_ask_volume_7d_usd: str = "0"
    taker_ask_volume_30d_usd: str = "0"
    taker_ask_volume_total_usd: str = "0"
    taker_bid_ask_imbalance_24h_pct: str = "0"
    taker_bid_ask_imbalance_7d_pct: str = "0"
    taker_bid_ask_imbalance_30d_pct: str = "0"
    taker_bid_ask_imbalance_total_pct: str = "0"
    unique_traders_24h: int = 0
    unique_traders_7d: int = 0
    unique_traders_30d: int = 0
    category_volume_share_24h_pct: str = "0"
    platform_volume_share_24h_pct: str = "0"
    slug: Optional[str] = None
    market_name: Optional[str] = None
    category: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "MarketVolumeMetrics":
        return MarketVolumeMetrics(
            market_pubkey=d.get("market_pubkey", ""),
            slug=d.get("slug"),
            market_name=d.get("market_name"),
            category=d.get("category"),
            volume_24h_usd=str(d.get("volume_24h_usd", "0")),
            volume_7d_usd=str(d.get("volume_7d_usd", "0")),
            volume_30d_usd=str(d.get("volume_30d_usd", "0")),
            volume_total_usd=str(d.get("volume_total_usd", "0")),
            taker_bid_volume_24h_usd=str(d.get("taker_bid_volume_24h_usd", "0")),
            taker_bid_volume_7d_usd=str(d.get("taker_bid_volume_7d_usd", "0")),
            taker_bid_volume_30d_usd=str(d.get("taker_bid_volume_30d_usd", "0")),
            taker_bid_volume_total_usd=str(d.get("taker_bid_volume_total_usd", "0")),
            taker_ask_volume_24h_usd=str(d.get("taker_ask_volume_24h_usd", "0")),
            taker_ask_volume_7d_usd=str(d.get("taker_ask_volume_7d_usd", "0")),
            taker_ask_volume_30d_usd=str(d.get("taker_ask_volume_30d_usd", "0")),
            taker_ask_volume_total_usd=str(d.get("taker_ask_volume_total_usd", "0")),
            taker_bid_ask_imbalance_24h_pct=str(
                d.get("taker_bid_ask_imbalance_24h_pct", "0")
            ),
            taker_bid_ask_imbalance_7d_pct=str(
                d.get("taker_bid_ask_imbalance_7d_pct", "0")
            ),
            taker_bid_ask_imbalance_30d_pct=str(
                d.get("taker_bid_ask_imbalance_30d_pct", "0")
            ),
            taker_bid_ask_imbalance_total_pct=str(
                d.get("taker_bid_ask_imbalance_total_pct", "0")
            ),
            unique_traders_24h=int(d.get("unique_traders_24h", 0)),
            unique_traders_7d=int(d.get("unique_traders_7d", 0)),
            unique_traders_30d=int(d.get("unique_traders_30d", 0)),
            category_volume_share_24h_pct=str(
                d.get("category_volume_share_24h_pct", "0")
            ),
            platform_volume_share_24h_pct=str(
                d.get("platform_volume_share_24h_pct", "0")
            ),
        )


@dataclass
class MarketsMetrics:
    """Envelope for /api/metrics/markets."""

    markets: list[MarketVolumeMetrics] = field(default_factory=list)
    total: int = 0

    @staticmethod
    def from_dict(d: dict) -> "MarketsMetrics":
        return MarketsMetrics(
            markets=[
                MarketVolumeMetrics.from_dict(x) for x in d.get("markets", [])
            ],
            total=int(d.get("total", 0)),
        )


# ─── Outcome / orderbook breakdowns (nested in MarketDetailMetrics) ─────────


@dataclass
class OutcomeVolumeMetrics:
    outcome_index: Optional[int] = None
    outcome_name: Optional[str] = None
    outcome_name_long: Optional[str] = None
    volume_24h_usd: str = "0"
    volume_7d_usd: str = "0"
    volume_30d_usd: str = "0"
    volume_total_usd: str = "0"
    taker_bid_volume_24h_usd: str = "0"
    taker_bid_volume_7d_usd: str = "0"
    taker_bid_volume_30d_usd: str = "0"
    taker_bid_volume_total_usd: str = "0"
    taker_ask_volume_24h_usd: str = "0"
    taker_ask_volume_7d_usd: str = "0"
    taker_ask_volume_30d_usd: str = "0"
    taker_ask_volume_total_usd: str = "0"
    taker_bid_ask_imbalance_24h_pct: str = "0"
    taker_bid_ask_imbalance_7d_pct: str = "0"
    taker_bid_ask_imbalance_30d_pct: str = "0"
    taker_bid_ask_imbalance_total_pct: str = "0"
    unique_traders_24h: int = 0
    unique_traders_7d: int = 0
    unique_traders_30d: int = 0
    volume_share_24h_pct: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "OutcomeVolumeMetrics":
        return OutcomeVolumeMetrics(
            outcome_index=d.get("outcome_index"),
            outcome_name=d.get("outcome_name"),
            outcome_name_long=d.get("outcome_name_long"),
            volume_24h_usd=str(d.get("volume_24h_usd", "0")),
            volume_7d_usd=str(d.get("volume_7d_usd", "0")),
            volume_30d_usd=str(d.get("volume_30d_usd", "0")),
            volume_total_usd=str(d.get("volume_total_usd", "0")),
            taker_bid_volume_24h_usd=str(d.get("taker_bid_volume_24h_usd", "0")),
            taker_bid_volume_7d_usd=str(d.get("taker_bid_volume_7d_usd", "0")),
            taker_bid_volume_30d_usd=str(d.get("taker_bid_volume_30d_usd", "0")),
            taker_bid_volume_total_usd=str(d.get("taker_bid_volume_total_usd", "0")),
            taker_ask_volume_24h_usd=str(d.get("taker_ask_volume_24h_usd", "0")),
            taker_ask_volume_7d_usd=str(d.get("taker_ask_volume_7d_usd", "0")),
            taker_ask_volume_30d_usd=str(d.get("taker_ask_volume_30d_usd", "0")),
            taker_ask_volume_total_usd=str(d.get("taker_ask_volume_total_usd", "0")),
            taker_bid_ask_imbalance_24h_pct=str(
                d.get("taker_bid_ask_imbalance_24h_pct", "0")
            ),
            taker_bid_ask_imbalance_7d_pct=str(
                d.get("taker_bid_ask_imbalance_7d_pct", "0")
            ),
            taker_bid_ask_imbalance_30d_pct=str(
                d.get("taker_bid_ask_imbalance_30d_pct", "0")
            ),
            taker_bid_ask_imbalance_total_pct=str(
                d.get("taker_bid_ask_imbalance_total_pct", "0")
            ),
            unique_traders_24h=int(d.get("unique_traders_24h", 0)),
            unique_traders_7d=int(d.get("unique_traders_7d", 0)),
            unique_traders_30d=int(d.get("unique_traders_30d", 0)),
            volume_share_24h_pct=str(d.get("volume_share_24h_pct", "0")),
        )


@dataclass
class MarketOrderbookVolumeMetrics:
    """Per-orderbook breakdown inside MarketDetailMetrics."""

    orderbook_id: str = ""
    base_deposit_asset: str = ""
    quote_deposit_asset: str = ""
    volume_24h_usd: str = "0"
    volume_7d_usd: str = "0"
    volume_30d_usd: str = "0"
    volume_total_usd: str = "0"
    volume_24h_base: str = "0"
    volume_7d_base: str = "0"
    volume_30d_base: str = "0"
    volume_total_base: str = "0"
    volume_24h_quote: str = "0"
    volume_7d_quote: str = "0"
    volume_30d_quote: str = "0"
    volume_total_quote: str = "0"
    taker_bid_volume_24h_usd: str = "0"
    taker_bid_volume_7d_usd: str = "0"
    taker_bid_volume_30d_usd: str = "0"
    taker_bid_volume_total_usd: str = "0"
    taker_bid_volume_24h_base: str = "0"
    taker_bid_volume_7d_base: str = "0"
    taker_bid_volume_30d_base: str = "0"
    taker_bid_volume_total_base: str = "0"
    taker_bid_volume_24h_quote: str = "0"
    taker_bid_volume_7d_quote: str = "0"
    taker_bid_volume_30d_quote: str = "0"
    taker_bid_volume_total_quote: str = "0"
    taker_ask_volume_24h_usd: str = "0"
    taker_ask_volume_7d_usd: str = "0"
    taker_ask_volume_30d_usd: str = "0"
    taker_ask_volume_total_usd: str = "0"
    taker_ask_volume_24h_base: str = "0"
    taker_ask_volume_7d_base: str = "0"
    taker_ask_volume_30d_base: str = "0"
    taker_ask_volume_total_base: str = "0"
    taker_ask_volume_24h_quote: str = "0"
    taker_ask_volume_7d_quote: str = "0"
    taker_ask_volume_30d_quote: str = "0"
    taker_ask_volume_total_quote: str = "0"
    taker_bid_ask_imbalance_24h_pct: str = "0"
    taker_bid_ask_imbalance_7d_pct: str = "0"
    taker_bid_ask_imbalance_30d_pct: str = "0"
    taker_bid_ask_imbalance_total_pct: str = "0"
    volume_share_24h_pct: str = "0"
    outcome_index: Optional[int] = None
    outcome_name: Optional[str] = None
    outcome_name_long: Optional[str] = None
    base_deposit_symbol: Optional[str] = None
    quote_deposit_symbol: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "MarketOrderbookVolumeMetrics":
        def g(key: str) -> str:
            return str(d.get(key, "0"))

        return MarketOrderbookVolumeMetrics(
            orderbook_id=d.get("orderbook_id", ""),
            outcome_index=d.get("outcome_index"),
            outcome_name=d.get("outcome_name"),
            outcome_name_long=d.get("outcome_name_long"),
            base_deposit_asset=d.get("base_deposit_asset", ""),
            base_deposit_symbol=d.get("base_deposit_symbol"),
            quote_deposit_asset=d.get("quote_deposit_asset", ""),
            quote_deposit_symbol=d.get("quote_deposit_symbol"),
            volume_24h_usd=g("volume_24h_usd"),
            volume_7d_usd=g("volume_7d_usd"),
            volume_30d_usd=g("volume_30d_usd"),
            volume_total_usd=g("volume_total_usd"),
            volume_24h_base=g("volume_24h_base"),
            volume_7d_base=g("volume_7d_base"),
            volume_30d_base=g("volume_30d_base"),
            volume_total_base=g("volume_total_base"),
            volume_24h_quote=g("volume_24h_quote"),
            volume_7d_quote=g("volume_7d_quote"),
            volume_30d_quote=g("volume_30d_quote"),
            volume_total_quote=g("volume_total_quote"),
            taker_bid_volume_24h_usd=g("taker_bid_volume_24h_usd"),
            taker_bid_volume_7d_usd=g("taker_bid_volume_7d_usd"),
            taker_bid_volume_30d_usd=g("taker_bid_volume_30d_usd"),
            taker_bid_volume_total_usd=g("taker_bid_volume_total_usd"),
            taker_bid_volume_24h_base=g("taker_bid_volume_24h_base"),
            taker_bid_volume_7d_base=g("taker_bid_volume_7d_base"),
            taker_bid_volume_30d_base=g("taker_bid_volume_30d_base"),
            taker_bid_volume_total_base=g("taker_bid_volume_total_base"),
            taker_bid_volume_24h_quote=g("taker_bid_volume_24h_quote"),
            taker_bid_volume_7d_quote=g("taker_bid_volume_7d_quote"),
            taker_bid_volume_30d_quote=g("taker_bid_volume_30d_quote"),
            taker_bid_volume_total_quote=g("taker_bid_volume_total_quote"),
            taker_ask_volume_24h_usd=g("taker_ask_volume_24h_usd"),
            taker_ask_volume_7d_usd=g("taker_ask_volume_7d_usd"),
            taker_ask_volume_30d_usd=g("taker_ask_volume_30d_usd"),
            taker_ask_volume_total_usd=g("taker_ask_volume_total_usd"),
            taker_ask_volume_24h_base=g("taker_ask_volume_24h_base"),
            taker_ask_volume_7d_base=g("taker_ask_volume_7d_base"),
            taker_ask_volume_30d_base=g("taker_ask_volume_30d_base"),
            taker_ask_volume_total_base=g("taker_ask_volume_total_base"),
            taker_ask_volume_24h_quote=g("taker_ask_volume_24h_quote"),
            taker_ask_volume_7d_quote=g("taker_ask_volume_7d_quote"),
            taker_ask_volume_30d_quote=g("taker_ask_volume_30d_quote"),
            taker_ask_volume_total_quote=g("taker_ask_volume_total_quote"),
            taker_bid_ask_imbalance_24h_pct=g("taker_bid_ask_imbalance_24h_pct"),
            taker_bid_ask_imbalance_7d_pct=g("taker_bid_ask_imbalance_7d_pct"),
            taker_bid_ask_imbalance_30d_pct=g("taker_bid_ask_imbalance_30d_pct"),
            taker_bid_ask_imbalance_total_pct=g(
                "taker_bid_ask_imbalance_total_pct"
            ),
            volume_share_24h_pct=g("volume_share_24h_pct"),
        )


@dataclass
class MarketDetailMetrics:
    """Response of /api/metrics/markets/{market_pubkey}."""

    market_pubkey: str = ""
    volume_24h_usd: str = "0"
    volume_7d_usd: str = "0"
    volume_30d_usd: str = "0"
    volume_total_usd: str = "0"
    taker_bid_volume_24h_usd: str = "0"
    taker_bid_volume_7d_usd: str = "0"
    taker_bid_volume_30d_usd: str = "0"
    taker_bid_volume_total_usd: str = "0"
    taker_ask_volume_24h_usd: str = "0"
    taker_ask_volume_7d_usd: str = "0"
    taker_ask_volume_30d_usd: str = "0"
    taker_ask_volume_total_usd: str = "0"
    taker_bid_ask_imbalance_24h_pct: str = "0"
    taker_bid_ask_imbalance_7d_pct: str = "0"
    taker_bid_ask_imbalance_30d_pct: str = "0"
    taker_bid_ask_imbalance_total_pct: str = "0"
    unique_traders_24h: int = 0
    unique_traders_7d: int = 0
    unique_traders_30d: int = 0
    category_volume_share_24h_pct: str = "0"
    platform_volume_share_24h_pct: str = "0"
    outcome_volumes: list[OutcomeVolumeMetrics] = field(default_factory=list)
    orderbook_volumes: list[MarketOrderbookVolumeMetrics] = field(default_factory=list)
    deposit_token_volumes: list[DepositTokenVolumeMetrics] = field(default_factory=list)
    slug: Optional[str] = None
    market_name: Optional[str] = None
    category: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "MarketDetailMetrics":
        return MarketDetailMetrics(
            market_pubkey=d.get("market_pubkey", ""),
            slug=d.get("slug"),
            market_name=d.get("market_name"),
            category=d.get("category"),
            volume_24h_usd=str(d.get("volume_24h_usd", "0")),
            volume_7d_usd=str(d.get("volume_7d_usd", "0")),
            volume_30d_usd=str(d.get("volume_30d_usd", "0")),
            volume_total_usd=str(d.get("volume_total_usd", "0")),
            taker_bid_volume_24h_usd=str(d.get("taker_bid_volume_24h_usd", "0")),
            taker_bid_volume_7d_usd=str(d.get("taker_bid_volume_7d_usd", "0")),
            taker_bid_volume_30d_usd=str(d.get("taker_bid_volume_30d_usd", "0")),
            taker_bid_volume_total_usd=str(d.get("taker_bid_volume_total_usd", "0")),
            taker_ask_volume_24h_usd=str(d.get("taker_ask_volume_24h_usd", "0")),
            taker_ask_volume_7d_usd=str(d.get("taker_ask_volume_7d_usd", "0")),
            taker_ask_volume_30d_usd=str(d.get("taker_ask_volume_30d_usd", "0")),
            taker_ask_volume_total_usd=str(d.get("taker_ask_volume_total_usd", "0")),
            taker_bid_ask_imbalance_24h_pct=str(
                d.get("taker_bid_ask_imbalance_24h_pct", "0")
            ),
            taker_bid_ask_imbalance_7d_pct=str(
                d.get("taker_bid_ask_imbalance_7d_pct", "0")
            ),
            taker_bid_ask_imbalance_30d_pct=str(
                d.get("taker_bid_ask_imbalance_30d_pct", "0")
            ),
            taker_bid_ask_imbalance_total_pct=str(
                d.get("taker_bid_ask_imbalance_total_pct", "0")
            ),
            unique_traders_24h=int(d.get("unique_traders_24h", 0)),
            unique_traders_7d=int(d.get("unique_traders_7d", 0)),
            unique_traders_30d=int(d.get("unique_traders_30d", 0)),
            category_volume_share_24h_pct=str(
                d.get("category_volume_share_24h_pct", "0")
            ),
            platform_volume_share_24h_pct=str(
                d.get("platform_volume_share_24h_pct", "0")
            ),
            outcome_volumes=[
                OutcomeVolumeMetrics.from_dict(x)
                for x in d.get("outcome_volumes", [])
            ],
            orderbook_volumes=[
                MarketOrderbookVolumeMetrics.from_dict(x)
                for x in d.get("orderbook_volumes", [])
            ],
            deposit_token_volumes=[
                DepositTokenVolumeMetrics.from_dict(x)
                for x in d.get("deposit_token_volumes", [])
            ],
        )


# ─── Orderbook ───────────────────────────────────────────────────────────────


@dataclass
class OrderbookVolumeMetrics:
    """Response of /api/metrics/orderbooks/{orderbook_id}."""

    orderbook_id: str = ""
    market_pubkey: str = ""
    base_deposit_asset: str = ""
    quote_deposit_asset: str = ""
    volume_24h_usd: str = "0"
    volume_7d_usd: str = "0"
    volume_30d_usd: str = "0"
    volume_total_usd: str = "0"
    volume_24h_base: str = "0"
    volume_7d_base: str = "0"
    volume_30d_base: str = "0"
    volume_total_base: str = "0"
    volume_24h_quote: str = "0"
    volume_7d_quote: str = "0"
    volume_30d_quote: str = "0"
    volume_total_quote: str = "0"
    taker_bid_volume_24h_usd: str = "0"
    taker_bid_volume_7d_usd: str = "0"
    taker_bid_volume_30d_usd: str = "0"
    taker_bid_volume_total_usd: str = "0"
    taker_bid_volume_24h_base: str = "0"
    taker_bid_volume_7d_base: str = "0"
    taker_bid_volume_30d_base: str = "0"
    taker_bid_volume_total_base: str = "0"
    taker_bid_volume_24h_quote: str = "0"
    taker_bid_volume_7d_quote: str = "0"
    taker_bid_volume_30d_quote: str = "0"
    taker_bid_volume_total_quote: str = "0"
    taker_ask_volume_24h_usd: str = "0"
    taker_ask_volume_7d_usd: str = "0"
    taker_ask_volume_30d_usd: str = "0"
    taker_ask_volume_total_usd: str = "0"
    taker_ask_volume_24h_base: str = "0"
    taker_ask_volume_7d_base: str = "0"
    taker_ask_volume_30d_base: str = "0"
    taker_ask_volume_total_base: str = "0"
    taker_ask_volume_24h_quote: str = "0"
    taker_ask_volume_7d_quote: str = "0"
    taker_ask_volume_30d_quote: str = "0"
    taker_ask_volume_total_quote: str = "0"
    taker_bid_ask_imbalance_24h_pct: str = "0"
    taker_bid_ask_imbalance_7d_pct: str = "0"
    taker_bid_ask_imbalance_30d_pct: str = "0"
    taker_bid_ask_imbalance_total_pct: str = "0"
    unique_traders_24h: int = 0
    unique_traders_7d: int = 0
    unique_traders_30d: int = 0
    market_volume_share_24h_pct: str = "0"
    outcome_index: Optional[int] = None
    outcome_name: Optional[str] = None
    outcome_name_long: Optional[str] = None
    base_deposit_symbol: Optional[str] = None
    quote_deposit_symbol: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderbookVolumeMetrics":
        def g(key: str) -> str:
            return str(d.get(key, "0"))

        return OrderbookVolumeMetrics(
            orderbook_id=d.get("orderbook_id", ""),
            market_pubkey=d.get("market_pubkey", ""),
            outcome_index=d.get("outcome_index"),
            outcome_name=d.get("outcome_name"),
            outcome_name_long=d.get("outcome_name_long"),
            base_deposit_asset=d.get("base_deposit_asset", ""),
            base_deposit_symbol=d.get("base_deposit_symbol"),
            quote_deposit_asset=d.get("quote_deposit_asset", ""),
            quote_deposit_symbol=d.get("quote_deposit_symbol"),
            volume_24h_usd=g("volume_24h_usd"),
            volume_7d_usd=g("volume_7d_usd"),
            volume_30d_usd=g("volume_30d_usd"),
            volume_total_usd=g("volume_total_usd"),
            volume_24h_base=g("volume_24h_base"),
            volume_7d_base=g("volume_7d_base"),
            volume_30d_base=g("volume_30d_base"),
            volume_total_base=g("volume_total_base"),
            volume_24h_quote=g("volume_24h_quote"),
            volume_7d_quote=g("volume_7d_quote"),
            volume_30d_quote=g("volume_30d_quote"),
            volume_total_quote=g("volume_total_quote"),
            taker_bid_volume_24h_usd=g("taker_bid_volume_24h_usd"),
            taker_bid_volume_7d_usd=g("taker_bid_volume_7d_usd"),
            taker_bid_volume_30d_usd=g("taker_bid_volume_30d_usd"),
            taker_bid_volume_total_usd=g("taker_bid_volume_total_usd"),
            taker_bid_volume_24h_base=g("taker_bid_volume_24h_base"),
            taker_bid_volume_7d_base=g("taker_bid_volume_7d_base"),
            taker_bid_volume_30d_base=g("taker_bid_volume_30d_base"),
            taker_bid_volume_total_base=g("taker_bid_volume_total_base"),
            taker_bid_volume_24h_quote=g("taker_bid_volume_24h_quote"),
            taker_bid_volume_7d_quote=g("taker_bid_volume_7d_quote"),
            taker_bid_volume_30d_quote=g("taker_bid_volume_30d_quote"),
            taker_bid_volume_total_quote=g("taker_bid_volume_total_quote"),
            taker_ask_volume_24h_usd=g("taker_ask_volume_24h_usd"),
            taker_ask_volume_7d_usd=g("taker_ask_volume_7d_usd"),
            taker_ask_volume_30d_usd=g("taker_ask_volume_30d_usd"),
            taker_ask_volume_total_usd=g("taker_ask_volume_total_usd"),
            taker_ask_volume_24h_base=g("taker_ask_volume_24h_base"),
            taker_ask_volume_7d_base=g("taker_ask_volume_7d_base"),
            taker_ask_volume_30d_base=g("taker_ask_volume_30d_base"),
            taker_ask_volume_total_base=g("taker_ask_volume_total_base"),
            taker_ask_volume_24h_quote=g("taker_ask_volume_24h_quote"),
            taker_ask_volume_7d_quote=g("taker_ask_volume_7d_quote"),
            taker_ask_volume_30d_quote=g("taker_ask_volume_30d_quote"),
            taker_ask_volume_total_quote=g("taker_ask_volume_total_quote"),
            taker_bid_ask_imbalance_24h_pct=g("taker_bid_ask_imbalance_24h_pct"),
            taker_bid_ask_imbalance_7d_pct=g("taker_bid_ask_imbalance_7d_pct"),
            taker_bid_ask_imbalance_30d_pct=g("taker_bid_ask_imbalance_30d_pct"),
            taker_bid_ask_imbalance_total_pct=g(
                "taker_bid_ask_imbalance_total_pct"
            ),
            unique_traders_24h=int(d.get("unique_traders_24h", 0)),
            unique_traders_7d=int(d.get("unique_traders_7d", 0)),
            unique_traders_30d=int(d.get("unique_traders_30d", 0)),
            market_volume_share_24h_pct=g("market_volume_share_24h_pct"),
        )


# ─── Category ────────────────────────────────────────────────────────────────


@dataclass
class CategoryVolumeMetrics:
    """Entry in /api/metrics/categories and response of /api/metrics/categories/{category}."""

    category: str = ""
    volume_24h_usd: str = "0"
    volume_7d_usd: str = "0"
    volume_30d_usd: str = "0"
    volume_total_usd: str = "0"
    taker_bid_volume_24h_usd: str = "0"
    taker_bid_volume_7d_usd: str = "0"
    taker_bid_volume_30d_usd: str = "0"
    taker_bid_volume_total_usd: str = "0"
    taker_ask_volume_24h_usd: str = "0"
    taker_ask_volume_7d_usd: str = "0"
    taker_ask_volume_30d_usd: str = "0"
    taker_ask_volume_total_usd: str = "0"
    taker_bid_ask_imbalance_24h_pct: str = "0"
    taker_bid_ask_imbalance_7d_pct: str = "0"
    taker_bid_ask_imbalance_30d_pct: str = "0"
    taker_bid_ask_imbalance_total_pct: str = "0"
    unique_traders_24h: int = 0
    unique_traders_7d: int = 0
    unique_traders_30d: int = 0
    platform_volume_share_24h_pct: str = "0"
    deposit_token_volumes: list[DepositTokenVolumeMetrics] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "CategoryVolumeMetrics":
        def g(key: str) -> str:
            return str(d.get(key, "0"))

        return CategoryVolumeMetrics(
            category=d.get("category", ""),
            volume_24h_usd=g("volume_24h_usd"),
            volume_7d_usd=g("volume_7d_usd"),
            volume_30d_usd=g("volume_30d_usd"),
            volume_total_usd=g("volume_total_usd"),
            taker_bid_volume_24h_usd=g("taker_bid_volume_24h_usd"),
            taker_bid_volume_7d_usd=g("taker_bid_volume_7d_usd"),
            taker_bid_volume_30d_usd=g("taker_bid_volume_30d_usd"),
            taker_bid_volume_total_usd=g("taker_bid_volume_total_usd"),
            taker_ask_volume_24h_usd=g("taker_ask_volume_24h_usd"),
            taker_ask_volume_7d_usd=g("taker_ask_volume_7d_usd"),
            taker_ask_volume_30d_usd=g("taker_ask_volume_30d_usd"),
            taker_ask_volume_total_usd=g("taker_ask_volume_total_usd"),
            taker_bid_ask_imbalance_24h_pct=g("taker_bid_ask_imbalance_24h_pct"),
            taker_bid_ask_imbalance_7d_pct=g("taker_bid_ask_imbalance_7d_pct"),
            taker_bid_ask_imbalance_30d_pct=g("taker_bid_ask_imbalance_30d_pct"),
            taker_bid_ask_imbalance_total_pct=g(
                "taker_bid_ask_imbalance_total_pct"
            ),
            unique_traders_24h=int(d.get("unique_traders_24h", 0)),
            unique_traders_7d=int(d.get("unique_traders_7d", 0)),
            unique_traders_30d=int(d.get("unique_traders_30d", 0)),
            platform_volume_share_24h_pct=g("platform_volume_share_24h_pct"),
            deposit_token_volumes=[
                DepositTokenVolumeMetrics.from_dict(x)
                for x in d.get("deposit_token_volumes", [])
            ],
        )


@dataclass
class CategoriesMetrics:
    """Envelope for /api/metrics/categories."""

    categories: list[CategoryVolumeMetrics] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "CategoriesMetrics":
        return CategoriesMetrics(
            categories=[
                CategoryVolumeMetrics.from_dict(x)
                for x in d.get("categories", [])
            ],
        )


# ─── Leaderboard ─────────────────────────────────────────────────────────────


@dataclass
class LeaderboardEntry:
    """Entry in /api/metrics/leaderboard/markets."""

    rank: int = 0
    market_pubkey: str = ""
    volume_24h_usd: str = "0"
    category_volume_share_24h_pct: str = "0"
    platform_volume_share_24h_pct: str = "0"
    slug: Optional[str] = None
    market_name: Optional[str] = None
    category: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "LeaderboardEntry":
        return LeaderboardEntry(
            rank=int(d.get("rank", 0)),
            market_pubkey=d.get("market_pubkey", ""),
            slug=d.get("slug"),
            market_name=d.get("market_name"),
            category=d.get("category"),
            volume_24h_usd=str(d.get("volume_24h_usd", "0")),
            category_volume_share_24h_pct=str(
                d.get("category_volume_share_24h_pct", "0")
            ),
            platform_volume_share_24h_pct=str(
                d.get("platform_volume_share_24h_pct", "0")
            ),
        )


@dataclass
class Leaderboard:
    """Envelope for /api/metrics/leaderboard/markets."""

    entries: list[LeaderboardEntry] = field(default_factory=list)
    period: str = ""

    @staticmethod
    def from_dict(d: dict) -> "Leaderboard":
        return Leaderboard(
            entries=[LeaderboardEntry.from_dict(x) for x in d.get("entries", [])],
            period=d.get("period", ""),
        )


# ─── History ─────────────────────────────────────────────────────────────────


@dataclass
class HistoryPoint:
    """Bucket in /api/metrics/history/{scope}/{scope_key}."""

    bucket_start: int = 0
    volume_usd: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "HistoryPoint":
        return HistoryPoint(
            bucket_start=int(d.get("bucket_start", 0)),
            volume_usd=str(d.get("volume_usd", "0")),
        )


@dataclass
class MetricsHistory:
    """Response of /api/metrics/history/{scope}/{scope_key}."""

    scope: str = ""
    scope_key: str = ""
    resolution: str = ""
    points: list[HistoryPoint] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "MetricsHistory":
        return MetricsHistory(
            scope=d.get("scope", ""),
            scope_key=d.get("scope_key", ""),
            resolution=d.get("resolution", ""),
            points=[HistoryPoint.from_dict(x) for x in d.get("points", [])],
        )


# ─── Queries ─────────────────────────────────────────────────────────────────


@dataclass
class MetricsHistoryQuery:
    """Query for /api/metrics/history/{scope}/{scope_key}."""

    resolution: str = "1h"
    from_ms: Optional[int] = None
    to_ms: Optional[int] = None
    limit: Optional[int] = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {"resolution": self.resolution}
        # Backend handler query is `from: Option<i64>, to: Option<i64>, limit: usize`.
        if self.from_ms is not None:
            params["from"] = str(self.from_ms)
        if self.to_ms is not None:
            params["to"] = str(self.to_ms)
        if self.limit is not None:
            params["limit"] = str(self.limit)
        return params


@dataclass
class DepositTokenVolumeHistoryQuery:
    """Query for /api/metrics/deposit-tokens/volume-history."""

    from_ms: Optional[int] = None
    to_ms: Optional[int] = None
    limit: Optional[int] = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {}
        if self.from_ms is not None:
            params["from"] = str(self.from_ms)
        if self.to_ms is not None:
            params["to"] = str(self.to_ms)
        if self.limit is not None:
            params["limit"] = str(self.limit)
        return params


@dataclass
class OpenInterestHistoryQuery:
    """Query for /api/metrics/open-interest/history."""

    from_ms: Optional[int] = None
    to_ms: Optional[int] = None
    limit: Optional[int] = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {}
        if self.from_ms is not None:
            params["from"] = str(self.from_ms)
        if self.to_ms is not None:
            params["to"] = str(self.to_ms)
        if self.limit is not None:
            params["limit"] = str(self.limit)
        return params


@dataclass
class UniqueTradersHistoryQuery:
    """Query for /api/metrics/unique-traders/history."""

    scope: Optional[UniqueTradersHistoryScope] = None
    scope_key: Optional[str] = None
    from_ms: Optional[int] = None
    to_ms: Optional[int] = None
    limit: Optional[int] = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {}
        if self.scope is not None:
            params["scope"] = self.scope
        if self.scope_key is not None:
            params["scope_key"] = self.scope_key
        if self.from_ms is not None:
            params["from"] = str(self.from_ms)
        if self.to_ms is not None:
            params["to"] = str(self.to_ms)
        if self.limit is not None:
            params["limit"] = str(self.limit)
        return params


@dataclass
class UserMetrics:
    """Per-wallet trading + referral aggregates.

    Response shape of ``metrics().user()``, ``metrics().user_with_cookies()``,
    and ``metrics().user_by_wallet()``.
    """

    wallet_address: str
    total_outcomes_traded: int
    total_volume_usd: str
    total_referrals_used: int

    @classmethod
    def from_dict(cls, data: dict) -> "UserMetrics":
        return cls(
            wallet_address=data.get("wallet_address", ""),
            total_outcomes_traded=int(data.get("total_outcomes_traded", 0)),
            total_volume_usd=str(data.get("total_volume_usd", "0")),
            total_referrals_used=int(data.get("total_referrals_used", 0)),
        )


__all__ = [
    "DepositTokenVolumeMetrics",
    "DepositTokensMetrics",
    "DepositTokenVolumeHistoryToken",
    "DepositTokenVolumeHistoryPointToken",
    "DepositTokenVolumeHistoryPoint",
    "DepositTokenVolumeHistory",
    "DepositTokenVolumeHistoryQuery",
    "OpenInterestHistoryDepositAsset",
    "OpenInterestHistoryPointDepositAsset",
    "OpenInterestHistoryPoint",
    "OpenInterestHistory",
    "OpenInterestHistoryQuery",
    "UniqueTradersHistoryScope",
    "UniqueTradersHistoryPoint",
    "UniqueTradersHistory",
    "UniqueTradersHistoryQuery",
    "PlatformMetrics",
    "MarketVolumeMetrics",
    "MarketsMetrics",
    "OutcomeVolumeMetrics",
    "MarketOrderbookVolumeMetrics",
    "MarketDetailMetrics",
    "OrderbookVolumeMetrics",
    "CategoryVolumeMetrics",
    "CategoriesMetrics",
    "LeaderboardEntry",
    "Leaderboard",
    "HistoryPoint",
    "MetricsHistory",
    "MetricsHistoryQuery",
    "UserMetrics",
]
