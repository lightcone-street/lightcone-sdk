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
    condition_id: int = 0
    condition_name: str = ""
    token_mint: str = ""
    amount: int = 0
    usd_value: str = "0"


@dataclass
class Position:
    """Position in a single market."""
    event_pubkey: str
    event_name: str = ""
    event_img_src: str = ""
    outcomes: list[PositionOutcome] = field(default_factory=list)
    total_value: str = "0"
    created_at: Optional[str] = None


@dataclass
class WalletHolding:
    token_mint: str
    symbol: str = ""
    amount: int = 0
    decimals: int = 6
    usd_value: str = "0"
    img_src: str = ""


@dataclass
class DepositAssetMetadata:
    mint: str
    symbol: str = ""
    name: str = ""
    icon_url: str = ""
    decimals: int = 6
    value: str = "0"


@dataclass
class DepositTokenBalance:
    mint: str
    amount: int = 0
    idle: int = 0
    symbol: str = ""
    name: str = ""
    icon_url: str = ""


@dataclass
class Portfolio:
    """User's full portfolio."""
    user_address: str
    wallet_holdings: list[WalletHolding] = field(default_factory=list)
    positions: list[Position] = field(default_factory=list)
    total_wallet_value: str = "0"
    total_positions_value: str = "0"


@dataclass
class TokenBalanceComputedBase:
    mint: str
    value: str = "0"
    size: str = "0"
    price: str = "0"


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
