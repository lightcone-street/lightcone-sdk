"""Market domain types."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class Status(str, Enum):
    PENDING = "pending"
    ACTIVE = "active"
    RESOLVED = "resolved"
    CANCELLED = "cancelled"


@dataclass
class Outcome:
    name: str
    index: int
    mint: Optional[str] = None
    symbol: Optional[str] = None


@dataclass
class ConditionalToken:
    mint: str
    outcome_index: int
    name: Optional[str] = None
    symbol: Optional[str] = None
    uri: Optional[str] = None
    decimals: int = 6


@dataclass
class DepositAsset:
    mint: str
    symbol: Optional[str] = None
    name: Optional[str] = None
    decimals: int = 6
    icon_url: Optional[str] = None


@dataclass
class ValidatedTokens:
    conditional_tokens: list[ConditionalToken] = field(default_factory=list)
    deposit_assets: list[DepositAsset] = field(default_factory=list)


@dataclass
class TokenMetadata:
    mint: str
    name: Optional[str] = None
    symbol: Optional[str] = None
    decimals: int = 6
    icon_url: Optional[str] = None


@dataclass
class OrderBookPairSummary:
    id: str
    base_token: str
    quote_token: str
    outcome_index: int
    tick_size: Optional[str] = None
    active: bool = True


@dataclass
class Market:
    """Rich market domain type."""
    id: str
    pubkey: str
    name: str
    slug: Optional[str] = None
    description: Optional[str] = None
    status: Status = Status.PENDING
    volume: Optional[str] = None
    outcomes: list[Outcome] = field(default_factory=list)
    conditional_tokens: list[ConditionalToken] = field(default_factory=list)
    deposit_assets: list[DepositAsset] = field(default_factory=list)
    orderbook_pairs: list[OrderBookPairSummary] = field(default_factory=list)
    token_metadata: list[TokenMetadata] = field(default_factory=list)
    icon_url: Optional[str] = None
    category: Optional[str] = None
    featured: bool = False
    created_at: Optional[str] = None
    resolved_at: Optional[str] = None
    winning_outcome: Optional[int] = None


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
    "OrderBookPairSummary",
    "Market",
    "MarketValidationError",
]
