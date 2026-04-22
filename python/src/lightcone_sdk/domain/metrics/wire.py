"""Metrics wire types - mirror the backend's dto::metrics shapes.

Decimal-bearing fields are stored as `str` to match the existing SDK
convention (wire types in ``order/wire.py``, ``market/wire.py`` etc. also
use ``str`` for Decimal-valued fields). Consumers who need numeric math
can wrap a field in ``decimal.Decimal`` themselves.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional


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


__all__ = [
    "DepositTokenVolumeMetrics",
    "DepositTokensMetrics",
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
]
