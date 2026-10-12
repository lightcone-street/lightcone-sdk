"""Position domain types."""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum

from ...shared.price import add_decimals


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


TokenBalanceTokenType = DepositAssetType | ConditionalTokenType


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
    created_at: str | None = None


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
    description: str | None = None
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
    def from_dict(cls, data: dict[str, object]) -> DepositTokenBalancesSnapshot:
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
    #: A status this SDK version does not know; balances are unchanged.
    UNKNOWN = "unknown"

    @classmethod
    def from_wire(cls, value: str) -> WalletDepositBalanceStatus:
        for member in cls:
            if member is not cls.UNKNOWN and member.value == value:
                return member
        return cls.UNKNOWN


@dataclass
class WalletDepositBalanceSnapshot:
    """Complete wallet baseline that replaces all SPL and native state.

    The cross-balance slot may trail an earlier individual balance update because
    the backend reports the lower slot valid for the complete snapshot.
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

    Required fields, exact native SOL, and nested balance shape are validated.
    A status this SDK version does not know decodes as
    :attr:`WalletDepositBalanceStatus.UNKNOWN`. Unknown variants or malformed
    fields raise :class:`TypeError`.
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
        return WalletDepositBalanceStatusEvent(
            event_type=event_type,
            wallet_address=wallet_address,
            status=WalletDepositBalanceStatus.from_wire(raw_status),
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
    orderbook_id: str | None = None
    outcome_index: int = 0
    conditional_token: str = ""
    idle: str = "0"
    on_book: str = "0"

    def total(self) -> str:
        """Exact sum of idle + on-book (mirrors the ``balance`` field), as plain
        decimal text at the larger of the two scales."""
        return format(add_decimals(Decimal(self.idle), Decimal(self.on_book)), "f")

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


from .builders import (  # noqa: E402
    DepositBuilder,
    DepositToGlobalBuilder,
    GlobalToMarketDepositBuilder,
    InitPositionTokensBuilder,
    MergeBuilder,
    RedeemWinningsBuilder,
    WithdrawBuilder,
    WithdrawFromGlobalBuilder,
    WithdrawFromPositionBuilder,
)
from .client import (  # noqa: E402
    SolActionKind,
    SolActionPlan,
    SolBalanceDelta,
    native_withdraw_seed,
)
from .state import (  # noqa: E402
    SOL_RESERVE_WITH_ACCOUNT_CREATION_LAMPORTS,
    SOL_RESERVE_WITH_EXISTING_ACCOUNT_LAMPORTS,
    WRAPPED_SOL_MINT,
    SolActionCosts,
    SolBalanceAvailability,
    SolBalanceBreakdown,
    WalletDepositBalancesApplyResult,
    WalletDepositBalancesState,
)
from .wire import (  # noqa: E402
    FundingAccount,
    FundingUpdate,
    MarketPositionsResponse,
    ObservationState,
    PositionsResponse,
)

__all__ = [
    "DepositBuilder",
    "DepositToGlobalBuilder",
    "GlobalToMarketDepositBuilder",
    "InitPositionTokensBuilder",
    "MergeBuilder",
    "RedeemWinningsBuilder",
    "WithdrawBuilder",
    "WithdrawFromGlobalBuilder",
    "WithdrawFromPositionBuilder",
    "SolActionKind",
    "SolActionPlan",
    "SolBalanceDelta",
    "native_withdraw_seed",
    "SolActionCosts",
    "SolBalanceAvailability",
    "SolBalanceBreakdown",
    "SOL_RESERVE_WITH_ACCOUNT_CREATION_LAMPORTS",
    "SOL_RESERVE_WITH_EXISTING_ACCOUNT_LAMPORTS",
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
    "FundingAccount",
    "FundingUpdate",
    "ObservationState",
    "PositionsResponse",
    "MarketPositionsResponse",
]
