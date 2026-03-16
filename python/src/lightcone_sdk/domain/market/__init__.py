"""Market domain types."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

from ..orderbook import OrderBookPair


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
    icon_url: str = ""


@dataclass
class ConditionalToken:
    mint: str
    outcome_index: int
    outcome: str = ""
    deposit_asset: str = ""
    deposit_symbol: str = ""
    name: str = ""
    symbol: str = ""
    description: Optional[str] = None
    decimals: int = 6
    icon_url: str = ""


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
    icon_url: str = ""


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
    icon_url: str = ""
    name: str = ""


@dataclass
class Market:
    """Rich market domain type."""
    id: int
    pubkey: str
    name: str
    banner_image_url: str = ""
    icon_url: str = ""
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
    conditional_tokens: list[ConditionalToken] = field(default_factory=list)
    outcomes: list[Outcome] = field(default_factory=list)
    orderbook_pairs: list[OrderBookPair] = field(default_factory=list)
    orderbook_ids: list[str] = field(default_factory=list)
    token_metadata: dict[str, TokenMetadata] = field(default_factory=dict)


@dataclass
class MarketsResult:
    markets: list[Market] = field(default_factory=list)
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
    "ValidatedTokens",
    "TokenMetadata",
    "OrderBookPair",
    "Market",
    "MarketsResult",
    "MarketValidationError",
]
