"""Position domain types."""

from dataclasses import dataclass, field
from typing import Optional, Union


@dataclass
class DepositAssetType:
    """Token type for deposit assets."""
    kind: str = "deposit_asset"


@dataclass
class ConditionalTokenType:
    """Token type for conditional tokens with associated data."""
    kind: str = "conditional_token"
    orderbook_id: str = ""
    market_pubkey: str = ""
    outcome_index: int = 0


TokenBalanceTokenType = Union[DepositAssetType, ConditionalTokenType]


@dataclass
class TokenBalance:
    mint: str
    idle: str = "0"
    on_book: str = "0"
    token_type: TokenBalanceTokenType = field(default_factory=DepositAssetType)


@dataclass
class PositionOutcome:
    condition_id: int = 0
    condition_name: str = ""
    token_mint: str = ""
    amount: str = "0"
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
    amount: str = "0"
    decimals: int = 6
    usd_value: str = "0"
    img_src: str = ""


@dataclass
class DepositAssetMetadata:
    symbol: str = ""
    name: str = ""
    icon_url: str = ""


@dataclass
class DepositTokenBalance:
    mint: str
    idle: str = "0"
    symbol: str = ""
    name: str = ""
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""


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
    value: str = "0"
    size: str = "0"
    price: str = "0"


from .builders import (  # noqa: E402
    DepositBuilder,
    DepositToGlobalBuilder,
    ExtendPositionTokensBuilder,
    GlobalToMarketDepositBuilder,
    InitPositionTokensBuilder,
    MergeBuilder,
    RedeemWinningsBuilder,
    WithdrawBuilder,
    WithdrawFromGlobalBuilder,
    WithdrawFromPositionBuilder,
)
from .wire import (  # noqa: E402
    GlobalDeposit,
    MarketPositionsResponseWire as MarketPositionsResponse,
    PositionsResponseWire as PositionsResponse,
)


__all__ = [
    "DepositBuilder",
    "DepositToGlobalBuilder",
    "ExtendPositionTokensBuilder",
    "GlobalToMarketDepositBuilder",
    "InitPositionTokensBuilder",
    "MergeBuilder",
    "RedeemWinningsBuilder",
    "WithdrawBuilder",
    "WithdrawFromGlobalBuilder",
    "WithdrawFromPositionBuilder",
    "DepositAssetType",
    "ConditionalTokenType",
    "TokenBalanceTokenType",
    "TokenBalance",
    "PositionOutcome",
    "Position",
    "WalletHolding",
    "DepositAssetMetadata",
    "DepositTokenBalance",
    "Portfolio",
    "TokenBalanceComputedBase",
    "GlobalDeposit",
    "PositionsResponse",
    "MarketPositionsResponse",
]
