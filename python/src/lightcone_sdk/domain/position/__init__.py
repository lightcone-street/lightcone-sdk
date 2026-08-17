"""Position domain types."""

import re
from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum
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
    """Exact SPL balance plus display metadata for one mint.

    ``idle`` retains mint-specific decimal precision. Icon URLs are nullable
    because balance data remains valid when metadata is unavailable.
    """

    mint: str
    idle: str = "0"
    symbol: str = ""
    name: str = ""
    icon_url_low: str | None = None
    icon_url_medium: str | None = None
    icon_url_high: str | None = None


@dataclass
class DepositTokenBalancesSnapshot:
    """Complete authenticated wallet snapshot with native SOL kept separate.

    ``balances`` is mandatory and contains only SPL mints. Native SOL uses
    canonical non-negative text with exactly nine fractional digits. Use
    :meth:`from_dict` at the REST boundary so incomplete payloads fail closed.
    """

    context_slot: int
    native_sol_balance: str
    balances: dict[str, DepositTokenBalance] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: dict[str, object]) -> "DepositTokenBalancesSnapshot":
        """Strictly decode a complete REST snapshot or raise :class:`TypeError`."""
        raw_balances = _parse_deposit_token_balances(data.get("balances"))
        native_sol_balance = _require_native_sol_balance(data, "deposit-token snapshot")
        return cls(
            context_slot=_require_context_slot(data),
            native_sol_balance=native_sol_balance,
            balances=raw_balances,
        )


class WalletDepositBalanceStatus(str, Enum):
    """Recoverable stream conditions that retain the last accepted balances."""

    #: The backend is restoring its watcher and will publish a replacement.
    RECONNECTING = "reconnecting"
    #: Balance values remain usable while token metadata refresh is unavailable.
    METADATA_UNAVAILABLE = "metadata_unavailable"


@dataclass
class WalletDepositBalanceSnapshot:
    """Complete wallet baseline that replaces all SPL and native state.

    The cross-component slot may trail an earlier component update because the
    backend reports the lower slot valid for the complete snapshot.
    """

    event_type: str
    wallet_address: str
    context_slot: int
    native_sol_balance: str
    balances: dict[str, DepositTokenBalance] = field(default_factory=dict)


@dataclass
class WalletDepositBalanceUpdate:
    """Absolute one-mint replacement; explicit zero removes the mint from state."""

    event_type: str
    wallet_address: str
    context_slot: int
    balance: DepositTokenBalance


@dataclass
class WalletNativeSolBalanceUpdate:
    """Absolute canonical nine-decimal native SOL replacement, never a delta."""

    event_type: str
    wallet_address: str
    context_slot: int
    native_sol_balance: str


@dataclass
class WalletDepositBalanceStatusEvent:
    """Non-mutating wallet diagnostic with a machine-readable backend code."""

    event_type: str
    wallet_address: str
    status: WalletDepositBalanceStatus
    code: str


# Nested discriminated payload of the outer ``wallet_deposit_balances`` channel.
WalletDepositBalancesEvent = (
    WalletDepositBalanceSnapshot
    | WalletDepositBalanceUpdate
    | WalletNativeSolBalanceUpdate
    | WalletDepositBalanceStatusEvent
)


def wallet_deposit_balances_event_from_dict(
    data: dict[str, object],
) -> WalletDepositBalancesEvent:
    """Decode one strict nested wallet event by its exact ``event_type``.

    Required fields, known statuses, exact native SOL, and nested balance shape
    are validated. Unknown variants or malformed fields raise :class:`TypeError`.
    """
    event_type = _require_string(data, "event_type", "wallet balance event")
    wallet_address = _require_string(data, "wallet_address", "wallet balance event")
    if event_type == "wallet_deposit_balance_snapshot":
        native = _require_native_sol_balance(data, "wallet balance snapshot")
        return WalletDepositBalanceSnapshot(
            event_type=event_type,
            wallet_address=wallet_address,
            context_slot=_require_context_slot(data),
            balances=_parse_deposit_token_balances(data.get("balances")),
            native_sol_balance=native,
        )
    if event_type == "wallet_deposit_balance_update":
        return WalletDepositBalanceUpdate(
            event_type=event_type,
            wallet_address=wallet_address,
            context_slot=_require_context_slot(data),
            balance=_parse_deposit_token_balance(data.get("balance")),
        )
    if event_type == "wallet_native_sol_balance_update":
        return WalletNativeSolBalanceUpdate(
            event_type=event_type,
            wallet_address=wallet_address,
            context_slot=_require_context_slot(data),
            native_sol_balance=_require_native_sol_balance(data, "native SOL update"),
        )
    if event_type == "wallet_deposit_balance_status":
        raw_status = _require_string(data, "status", "wallet balance status")
        try:
            status = WalletDepositBalanceStatus(raw_status)
        except ValueError as error:
            raise TypeError(f"unknown wallet balance status: {raw_status}") from error
        return WalletDepositBalanceStatusEvent(
            event_type=event_type,
            wallet_address=wallet_address,
            status=status,
            code=_require_string(data, "code", "wallet balance status"),
        )
    raise TypeError(f"unknown wallet balance event_type: {event_type}")


def _parse_deposit_token_balances(value: object) -> dict[str, DepositTokenBalance]:
    """Require a complete object map; absence never synthesizes an empty snapshot."""
    if not isinstance(value, dict):
        raise TypeError("deposit-token snapshot balances must be an object")
    return {
        str(mint): _parse_deposit_token_balance(balance)
        for mint, balance in value.items()
    }


def _parse_deposit_token_balance(value: object) -> DepositTokenBalance:
    """Require exact balance fields while allowing only icon metadata to be null."""
    if not isinstance(value, dict):
        raise TypeError("deposit-token snapshot balance entries must be objects")
    data = value
    return DepositTokenBalance(
        mint=_require_string(data, "mint", "deposit-token balance"),
        idle=_require_string(data, "idle", "deposit-token balance"),
        symbol=_require_string(data, "symbol", "deposit-token balance"),
        name=_require_string(data, "name", "deposit-token balance"),
        icon_url_low=_optional_string(data, "icon_url_low"),
        icon_url_medium=_optional_string(data, "icon_url_medium"),
        icon_url_high=_optional_string(data, "icon_url_high"),
    )


def _require_context_slot(data: dict[str, object]) -> int:
    """Require a non-negative exact integer slot, rejecting booleans and floats."""
    value = data.get("context_slot")
    if type(value) is not int or value < 0:
        raise TypeError("wallet balance context_slot must be a non-negative integer")
    return value


def _require_string(data: dict, field_name: str, context: str) -> str:
    value = data.get(field_name)
    if not isinstance(value, str):
        raise TypeError(f"{context} {field_name} must be a string")
    return value


def _require_native_sol_balance(data: dict, context: str) -> str:
    """Require canonical lamport text without signs, exponents, or rounding."""
    value = _require_string(data, "native_sol_balance", context)
    if re.fullmatch(r"(?:0|[1-9][0-9]*)\.[0-9]{9}", value) is None:
        raise TypeError(
            f"{context} native_sol_balance must have exactly nine decimal places"
        )
    return value


def _optional_string(data: dict, field_name: str) -> str | None:
    value = data.get(field_name)
    if value is not None and not isinstance(value, str):
        raise TypeError(f"deposit-token balance {field_name} must be a string")
    return value


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
from .state import (  # noqa: E402
    WRAPPED_SOL_MINT,
    WalletDepositBalancesApplyResult,
    WalletDepositBalancesState,
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
    "WalletDepositBalanceStatus",
    "WalletDepositBalanceSnapshot",
    "WalletDepositBalanceUpdate",
    "WalletNativeSolBalanceUpdate",
    "WalletDepositBalanceStatusEvent",
    "WalletDepositBalancesEvent",
    "wallet_deposit_balances_event_from_dict",
    "WalletDepositBalancesApplyResult",
    "WalletDepositBalancesState",
    "WRAPPED_SOL_MINT",
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
