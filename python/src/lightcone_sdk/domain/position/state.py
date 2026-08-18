"""App-owned external-wallet deposit balance state."""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal, InvalidOperation
from enum import Enum

from ...shared.scaling import ExactDecimal, exact_scaled_integer
from . import (
    DepositTokenBalance,
    DepositTokenBalancesSnapshot,
    WalletDepositBalancesEvent,
    WalletDepositBalanceSnapshot,
    WalletDepositBalanceStatusEvent,
    WalletDepositBalanceUpdate,
    WalletNativeSolBalanceUpdate,
)

#: Canonical Tokenkeg WSOL mint used by aggregation and conversion preflight.
WRAPPED_SOL_MINT = "So11111111111111111111111111111111111111112"


class WalletDepositBalancesApplyResult(str, Enum):
    """Whether an event was accepted by or rejected by its lifecycle guard."""

    #: A complete snapshot or matching update was applied; values may be unchanged.
    APPLIED = "applied"
    #: Status, pre-baseline, or wrong-wallet events do not change state.
    IGNORED = "ignored"
    #: A matching event carried an invalid balance and was not applied.
    REJECTED = "rejected"


@dataclass
class WalletDepositBalancesState:
    """Mutable application-owned native SOL and mint-keyed SPL state.

    A new instance is uninitialized. A complete REST or WebSocket snapshot
    establishes its wallet baseline; component events then require that wallet
    and carry absolute values. ``context_slot`` records the latest accepted
    component observation rather than a globally monotonic stream version.
    Mapping containers are copied, but mutable balance objects are retained by
    reference; treat payload entries as immutable after applying them.
    """

    wallet_address: str | None = None
    context_slot: int | None = None
    balances: dict[str, DepositTokenBalance] = field(default_factory=dict)
    native_sol_balance: str | None = None

    def apply_rest_snapshot(
        self, wallet_address: str, snapshot: DepositTokenBalancesSnapshot
    ) -> WalletDepositBalancesApplyResult:
        """Initialize or wholesale-replace state without comparing prior slots."""
        self._replace(
            wallet_address,
            snapshot.context_slot,
            snapshot.balances,
            snapshot.native_sol_balance,
        )
        return WalletDepositBalancesApplyResult.APPLIED

    def apply_event(
        self, event: WalletDepositBalancesEvent
    ) -> WalletDepositBalancesApplyResult:
        """Apply authoritative snapshots, absolute updates, and lifecycle guards.

        Complete snapshots always replace. Matching component updates replace one
        value, zero SPL removes its mint, and status or wrong-wallet events are
        ignored without stale-slot filtering.
        """
        if isinstance(event, WalletDepositBalanceSnapshot):
            # Complete snapshots are authoritative even after a higher
            # component event because their slot is the lower component slot.
            self._replace(
                event.wallet_address,
                event.context_slot,
                event.balances,
                event.native_sol_balance,
            )
            return WalletDepositBalancesApplyResult.APPLIED
        if isinstance(event, WalletDepositBalanceStatusEvent):
            return WalletDepositBalancesApplyResult.IGNORED
        if not self._matches_initialized_wallet(event.wallet_address):
            return WalletDepositBalancesApplyResult.IGNORED
        if isinstance(event, WalletDepositBalanceUpdate):
            try:
                is_zero = _is_zero_token_amount(event.balance.idle)
            except ValueError:
                return WalletDepositBalancesApplyResult.REJECTED
            if is_zero:
                self.balances.pop(event.balance.mint, None)
            else:
                self.balances[event.balance.mint] = event.balance
            self.context_slot = event.context_slot
            return WalletDepositBalancesApplyResult.APPLIED
        if isinstance(event, WalletNativeSolBalanceUpdate):
            self.native_sol_balance = event.native_sol_balance
            self.context_slot = event.context_slot
            return WalletDepositBalancesApplyResult.APPLIED
        return WalletDepositBalancesApplyResult.IGNORED

    def combined_sol_balance(self) -> str:
        """Return exact native plus canonical WSOL with nine fractional digits.

        Arithmetic is arbitrary-width and does not merge the stored assets.
        Uninitialized or malformed cached values raise scaling errors.
        """
        native = self.native_sol_lamports()
        wrapped = self.balances.get(WRAPPED_SOL_MINT)
        wrapped_lamports = (
            exact_scaled_integer(wrapped.idle, 9) if wrapped is not None else 0
        )
        return _format_lamports(native + wrapped_lamports)

    def native_sol_lamports(self) -> int:
        """Scale cached native SOL exactly; transaction range is checked elsewhere."""
        if self.native_sol_balance is None:
            raise ValueError("wallet balance state is not initialized")
        return exact_scaled_integer(self.native_sol_balance, 9)

    def has_positive_wsol(self) -> bool:
        """Validate and test only the canonical WSOL idle amount."""
        wrapped = self.balances.get(WRAPPED_SOL_MINT)
        return wrapped is not None and exact_scaled_integer(wrapped.idle, 9) > 0

    def _matches_initialized_wallet(self, wallet_address: str) -> bool:
        """Prevent component events from crossing wallet or baseline boundaries."""
        return (
            self.wallet_address == wallet_address
            and self.context_slot is not None
            and self.native_sol_balance is not None
        )

    def _replace(
        self,
        wallet_address: str,
        context_slot: int,
        balances: dict[str, DepositTokenBalance],
        native_sol_balance: str,
    ) -> None:
        """Replace from authority, copying only the mapping container.

        Balance objects are retained by reference and must be treated as immutable.
        Complete authority is allowed to switch the state to another wallet.
        """
        self.wallet_address = wallet_address
        self.context_slot = context_slot
        self.balances = dict(balances)
        self.native_sol_balance = native_sol_balance


def sol_amount_to_lamports(value: ExactDecimal) -> int:
    """Scale exact SOL without rounding and enforce Solana's unsigned ``u64`` cap.

    Floats, negatives, and excess precision are rejected by the exact scaler.
    Positivity is an operation-level requirement enforced by ``wrap_sol``.
    """
    lamports = exact_scaled_integer(value, 9)
    if lamports > 2**64 - 1:
        raise ValueError("SOL amount exceeds the transaction u64 range")
    return lamports


def _format_lamports(value: int) -> str:
    """Format arbitrary non-negative lamports canonically without floating point."""
    whole, fraction = divmod(value, 1_000_000_000)
    return f"{whole}.{fraction:09d}"


def _is_zero_token_amount(value: str) -> bool:
    """Detect explicit zero without imposing SOL's nine-decimal scale on SPL tokens."""
    try:
        amount = Decimal(value)
    except InvalidOperation as error:
        raise ValueError(f"invalid deposit-token balance: {value}") from error
    if not amount.is_finite() or amount.is_signed():
        raise ValueError(f"invalid deposit-token balance: {value}")
    return amount.is_zero()
