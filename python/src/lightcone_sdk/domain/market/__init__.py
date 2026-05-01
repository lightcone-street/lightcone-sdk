"""Market domain types."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional, Protocol, TypeVar

from ..orderbook import OrderBookPair


class _HasSymbol(Protocol):
    symbol: str


def token_display_priority(token: _HasSymbol) -> int:
    """Display priority for sorting: lower values come first.

    BTC/WBTC tie at 0, ETH/WETH tie at 1, SOL at 2; everything else falls
    to the alphabetical tail.
    """
    match token.symbol:
        case "BTC" | "WBTC":
            return 0
        case "ETH" | "WETH":
            return 1
        case "SOL":
            return 2
        case _:
            return 255


_T = TypeVar("_T", bound=_HasSymbol)


def sort_by_display_priority(tokens: list[_T]) -> list[_T]:
    """Return a new list ordered by display priority then alphabetical by symbol."""
    return sorted(tokens, key=lambda token: (token_display_priority(token), token.symbol))


class Status(str, Enum):
    PENDING = "pending"
    ACTIVE = "active"
    RESOLVED = "resolved"
    CANCELLED = "cancelled"

    def as_str(self) -> str:
        return self.value

    @staticmethod
    def from_str(s: str) -> "Status":
        try:
            return Status(s.lower())
        except ValueError:
            return Status.PENDING


@dataclass
class Outcome:
    index: int
    name: str
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""


@dataclass
class ConditionalToken:
    pubkey: str
    outcome_index: int
    id: int = 0
    outcome: str = ""
    deposit_asset: str = ""
    deposit_symbol: str = ""
    name: str = ""
    symbol: str = ""
    description: Optional[str] = None
    decimals: int = 6
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""


@dataclass
class DepositAsset:
    id: int = 0
    market_pda: str = ""
    deposit_asset: str = ""
    num_outcomes: int = 0
    name: str = ""
    symbol: str = ""
    description: Optional[str] = None
    decimals: int = 6
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""


@dataclass
class DepositAssetPair:
    """A base/quote pairing of two :class:`DepositAsset` instances.

    Populated on :attr:`Market.deposit_asset_pairs` during wire→domain
    conversion; one entry per unique ``(base.deposit_asset, quote.deposit_asset)``
    combination across the market's orderbook pairs.
    """
    id: str
    base: DepositAsset
    quote: DepositAsset

    @property
    def symbol(self) -> str:
        """Display symbol delegated to the base deposit asset — enables
        `sort_by_display_priority` to treat pairs like tokens."""
        return self.base.symbol


@dataclass
class GlobalDepositAsset:
    """A globally whitelisted deposit asset (platform-scoped, not market-bound).

    Distinct from ``DepositAsset``, which is bound to a specific market.
    """
    id: int = 0
    deposit_asset: str = ""
    name: str = ""
    symbol: str = ""
    description: Optional[str] = None
    decimals: int = 6
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""
    whitelist_index: int = 0
    active: bool = False


@dataclass
class ValidatedTokens:
    token: Optional[DepositAsset] = None
    conditionals: list[ConditionalToken] = field(default_factory=list)
    metadata: dict[str, "TokenMetadata"] = field(default_factory=dict)


@dataclass
class TokenMetadata:
    pubkey: str = ""
    symbol: str = ""
    decimals: int = 6
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""
    name: str = ""


@dataclass
class Market:
    """Rich market domain type."""
    id: int
    pubkey: str
    name: str
    banner_image_url_low: str = ""
    banner_image_url_medium: str = ""
    banner_image_url_high: str = ""
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""
    featured_rank: Optional[int] = None
    volume: str = "0"
    slug: str = ""
    status: Status = Status.PENDING
    created_at: Optional[str] = None
    activated_at: Optional[str] = None
    settled_at: Optional[str] = None
    winning_outcome: Optional[int] = None
    description: str = ""
    definition: str = ""
    category: Optional[str] = None
    tags: list[str] = field(default_factory=list)
    deposit_assets: list[DepositAsset] = field(default_factory=list)
    deposit_asset_pairs: list[DepositAssetPair] = field(default_factory=list)
    conditional_tokens: list[ConditionalToken] = field(default_factory=list)
    outcomes: list[Outcome] = field(default_factory=list)
    orderbook_pairs: list[OrderBookPair] = field(default_factory=list)
    orderbook_ids: list[str] = field(default_factory=list)
    token_metadata: dict[str, TokenMetadata] = field(default_factory=dict)


@dataclass
class MarketsResult:
    markets: list[Market] = field(default_factory=list)
    validation_errors: list[str] = field(default_factory=list)


@dataclass
class GlobalDepositAssetsResult:
    """Result of fetching the global deposit asset whitelist.

    Assets that fail validation are skipped and their errors are reported
    separately.
    """
    assets: list[GlobalDepositAsset] = field(default_factory=list)
    validation_errors: list[str] = field(default_factory=list)


class MarketValidationError(Exception):
    def __init__(self, message: str, details: Optional[list[str]] = None):
        super().__init__(message)
        self.details = details or []


__all__ = [
    "Status",
    "Outcome",
    "ConditionalToken",
    "DepositAsset",
    "DepositAssetPair",
    "GlobalDepositAsset",
    "ValidatedTokens",
    "TokenMetadata",
    "OrderBookPair",
    "Market",
    "MarketsResult",
    "GlobalDepositAssetsResult",
    "MarketValidationError",
    "token_display_priority",
    "sort_by_display_priority",
]
