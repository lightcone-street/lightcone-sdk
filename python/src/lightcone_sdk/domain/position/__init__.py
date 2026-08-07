"""Position domain types."""

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Optional, Union

from ..order import UserMarketBalance, UserOutcomeBalance


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
    short_symbol: str = ""
    name: str = ""
    deposit_asset: str = ""
    icon_url_low: str = ""
    icon_url_medium: str = ""
    icon_url_high: str = ""
    description: Optional[str] = None
    decimals: int = 0


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
class DepositTokenBalancesSnapshot:
    context_slot: int
    balances: dict[str, DepositTokenBalance] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: dict[str, object]) -> "DepositTokenBalancesSnapshot":
        raw_balances = data.get("balances", {})
        if not isinstance(raw_balances, dict):
            raise TypeError("deposit-token snapshot balances must be an object")
        if not all(isinstance(balance, dict) for balance in raw_balances.values()):
            raise TypeError("deposit-token snapshot balance entries must be objects")
        return cls(
            context_slot=int(data["context_slot"]),
            balances={
                str(mint): DepositTokenBalance(**balance)
                for mint, balance in raw_balances.items()
            },
        )


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


@dataclass
class ConditionalBalanceDelta:
    """An incremental change to a user's balance for one conditional token."""

    market_pubkey: str = ""
    orderbook_id: Optional[str] = None
    outcome_index: int = 0
    conditional_token: str = ""
    idle: str = "0"
    on_book: str = "0"

    def total(self) -> str:
        """Full-precision sum of idle + on-book (mirrors the ``balance`` field)."""
        return str(Decimal(self.idle) + Decimal(self.on_book))

    def is_zero(self) -> bool:
        """True when the delta holds nothing idle and nothing resting on the book."""
        return not (
            Decimal(self.idle) > Decimal(0) or Decimal(self.on_book) > Decimal(0)
        )

    def into_token_balance(self) -> TokenBalance:
        return TokenBalance(
            mint=self.conditional_token,
            idle=self.idle,
            on_book=self.on_book,
            token_type=ConditionalTokenType(
                orderbook_id=self.orderbook_id or "",
                market_pubkey=self.market_pubkey,
                outcome_index=self.outcome_index,
            ),
        )

    def into_user_outcome_balance(self) -> UserOutcomeBalance:
        return UserOutcomeBalance(
            outcome_index=self.outcome_index,
            conditional_token=self.conditional_token,
            balance=self.total(),
            balance_idle=self.idle,
            balance_on_book=self.on_book,
        )


ConditionalTokenBalanceIndex = dict[str, UserOutcomeBalance]
DepositAssetBalanceIndex = dict[str, ConditionalTokenBalanceIndex]


class UserMarketBalanceIndex:
    """Nested index of a user's conditional-token balances, keyed
    ``market -> deposit_asset -> conditional_token``. Zero balances are dropped
    when building from wire records.
    """

    def __init__(self) -> None:
        self._inner: dict[str, DepositAssetBalanceIndex] = {}

    def get(self, market_pubkey: str) -> Optional[DepositAssetBalanceIndex]:
        return self._inner.get(market_pubkey)

    def insert(
        self, market_pubkey: str, market_entry: DepositAssetBalanceIndex
    ) -> None:
        self._inner[market_pubkey] = market_entry

    def extend(self, other: "UserMarketBalanceIndex") -> None:
        for market_pubkey, market_entry in other._inner.items():
            self._inner.setdefault(market_pubkey, {}).update(market_entry)

    def remove(self, market_pubkey: str) -> None:
        self._inner.pop(market_pubkey, None)

    def inner(self) -> dict[str, DepositAssetBalanceIndex]:
        return self._inner

    def market_pubkeys(self) -> list[str]:
        return sorted(self._inner)

    def is_empty(self) -> bool:
        return not self._inner

    @classmethod
    def from_user_market_balance(
        cls, market_balance: UserMarketBalance
    ) -> Optional["UserMarketBalanceIndex"]:
        market_entry: DepositAssetBalanceIndex = {}

        for deposit_asset_balance in market_balance.deposit_assets:
            outcomes: ConditionalTokenBalanceIndex = {}
            for outcome in deposit_asset_balance.outcomes:
                if not outcome.is_zero():
                    outcomes[outcome.conditional_token] = outcome
            if outcomes:
                market_entry[deposit_asset_balance.deposit_asset] = outcomes

        if not market_entry:
            return None

        index = cls()
        index.insert(market_balance.market_pubkey, market_entry)
        return index

    @classmethod
    def from_user_market_balances(
        cls, market_balances: list[UserMarketBalance]
    ) -> "UserMarketBalanceIndex":
        index = cls()
        for market_balance in market_balances:
            market_index = cls.from_user_market_balance(market_balance)
            if market_index is not None:
                index.extend(market_index)
        return index


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
    "DepositTokenBalancesSnapshot",
    "Portfolio",
    "TokenBalanceComputedBase",
    "ConditionalBalanceDelta",
    "ConditionalTokenBalanceIndex",
    "DepositAssetBalanceIndex",
    "UserMarketBalanceIndex",
    "GlobalDeposit",
    "PositionsResponse",
    "MarketPositionsResponse",
]
