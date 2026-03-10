"""Position domain types."""

from dataclasses import dataclass, field
from typing import Optional


class TokenBalanceTokenType:
    DEPOSIT_ASSET = "deposit_asset"
    CONDITIONAL_TOKEN = "conditional_token"


@dataclass
class TokenBalance:
    mint: str
    idle: int = 0
    on_book: int = 0
    token_type: str = TokenBalanceTokenType.CONDITIONAL_TOKEN


@dataclass
class PositionOutcome:
    condition_id: Optional[str] = None
    name: Optional[str] = None
    mint: Optional[str] = None
    amount: int = 0
    usd_value: Optional[str] = None


@dataclass
class Position:
    """Position in a single market."""
    event_pubkey: str
    event_name: Optional[str] = None
    outcomes: list[PositionOutcome] = field(default_factory=list)
    total_value: Optional[str] = None


@dataclass
class WalletHolding:
    token_mint: str
    symbol: Optional[str] = None
    amount: int = 0
    decimals: int = 6
    usd_value: Optional[str] = None


@dataclass
class DepositAssetMetadata:
    mint: str
    symbol: Optional[str] = None
    decimals: int = 6


@dataclass
class DepositTokenBalance:
    mint: str
    amount: int = 0
    symbol: Optional[str] = None


@dataclass
class Portfolio:
    """User's full portfolio."""
    user_address: str
    wallet_holdings: list[WalletHolding] = field(default_factory=list)
    positions: list[Position] = field(default_factory=list)
    total_wallet_value: Optional[str] = None
    total_positions_value: Optional[str] = None


@dataclass
class TokenBalanceComputedBase:
    mint: str
    idle: int = 0
    on_book: int = 0


__all__ = [
    "TokenBalanceTokenType",
    "TokenBalance",
    "PositionOutcome",
    "Position",
    "WalletHolding",
    "DepositAssetMetadata",
    "DepositTokenBalance",
    "Portfolio",
    "TokenBalanceComputedBase",
]
