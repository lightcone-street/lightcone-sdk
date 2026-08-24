"""App-owned external-wallet deposit balance state."""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal, InvalidOperation
from enum import Enum

from ...error import SdkError
from ...shared.scaling import exact_scaled_integer
from . import (
    DepositTokenBalance,
    DepositTokenBalancesSnapshot,
    WalletDepositBalancesEvent,
    WalletDepositBalanceSnapshot,
    WalletDepositBalanceStatusEvent,
    WalletDepositBalanceUpdate,
    WalletNativeSolBalanceUpdate,
)

#: Canonical WSOL mint under Solana's legacy SPL Token Program (Tokenkeg),
#: deliberately pinned away from Token-2022 for one protocol ATA authority.
WRAPPED_SOL_MINT = "So11111111111111111111111111111111111111112"

#: Unsponsored native reserve floor in lamports when canonical ATA creation is required.
SOL_RESERVE_WITH_ACCOUNT_CREATION_LAMPORTS = 3_500_000
#: Unsponsored native reserve floor in lamports when canonical WSOL already exists.
SOL_RESERVE_WITH_EXISTING_ACCOUNT_LAMPORTS = 1_000_000
#: Largest exact non-negative lamport value accepted by Solana u64 fields.
MAX_SOLANA_LAMPORTS = 2**64 - 1


@dataclass(frozen=True)
class SolBalanceComponents:
    """Exact lamport components behind the single displayed SOL asset."""

    #: Lamports held by the Trading Wallet system account.
    native_lamports: int
    #: Token amount in the Trading Wallet's persistent canonical WSOL ATA.
    canonical_wsol_lamports: int

    @property
    def displayed_lamports(self) -> int:
        """Return native plus canonical WSOL without floating-point conversion."""
        return self.native_lamports + self.canonical_wsol_lamports


@dataclass(frozen=True)
class SolActionCosts:
    """Store live chain costs used to derive transaction funding requirements.

    ``reserve_lamports`` applies the ordinary action safety floor.
    :meth:`SolBalanceAvailability.from_unwrap_all_costs` instead rejects account
    creation, upfront rent, and sponsorship before using only ``fee_lamports`` as
    the unwrap-all reserve.
    """

    #: Live getFeeForMessage result, in lamports.
    fee_lamports: int
    #: Rent funded up front, even when a temporary account refunds it later.
    upfront_rent_lamports: int
    #: Whether this transaction creates the persistent canonical WSOL ATA.
    creates_canonical_wsol_account: bool
    #: Exact public sponsorship capability supplied by the caller.
    sponsored: bool

    @property
    def reserve_lamports(self) -> int:
        """Reserve live costs or the matching unsponsored safety floor."""
        for label, value in (
            ("transaction fee", self.fee_lamports),
            ("upfront rent", self.upfront_rent_lamports),
        ):
            if (
                isinstance(value, bool)
                or not isinstance(value, int)
                or value < 0
                or value > MAX_SOLANA_LAMPORTS
            ):
                raise SdkError(
                    f"{label} must fit the non-negative u64 lamport range"
                )
        live_costs = self.fee_lamports + self.upfront_rent_lamports
        if live_costs > MAX_SOLANA_LAMPORTS:
            raise SdkError("combined transaction costs must fit u64 lamports")
        if self.sponsored:
            return 0
        floor = (
            SOL_RESERVE_WITH_ACCOUNT_CREATION_LAMPORTS
            if self.creates_canonical_wsol_account
            else SOL_RESERVE_WITH_EXISTING_ACCOUNT_LAMPORTS
        )
        return max(live_costs, floor)


@dataclass(frozen=True)
class SolBalanceAvailability:
    """Action-specific displayed, reserved, and spendable SOL values."""

    #: Separately authoritative native and canonical WSOL balances.
    components: SolBalanceComponents
    #: Sum of both components in lamports, before reserve.
    displayed_lamports: int
    #: Native lamports withheld for an ordinary safety floor or exact unwrap fee.
    reserve_lamports: int
    #: Displayed lamports available to this action after reserve.
    spendable_lamports: int

    @classmethod
    def from_costs(
        cls, components: SolBalanceComponents, costs: SolActionCosts
    ) -> SolBalanceAvailability:
        """Derive availability and require native SOL to fund the reserve."""
        for label, value in (
            ("native SOL", components.native_lamports),
            ("canonical WSOL", components.canonical_wsol_lamports),
        ):
            if (
                isinstance(value, bool)
                or not isinstance(value, int)
                or value < 0
                or value > MAX_SOLANA_LAMPORTS
            ):
                raise SdkError(
                    f"{label} must fit the non-negative u64 lamport range"
                )
        reserve = costs.reserve_lamports
        displayed = components.displayed_lamports
        if displayed > MAX_SOLANA_LAMPORTS:
            raise SdkError("displayed SOL exceeds the transaction u64 range")
        if components.native_lamports < reserve:
            raise SdkError(
                "native SOL balance cannot fund the required "
                f"{reserve} lamport transaction reserve"
            )
        return cls(
            components=components,
            displayed_lamports=displayed,
            reserve_lamports=reserve,
            spendable_lamports=displayed - reserve,
        )

    @classmethod
    def from_unwrap_all_costs(
        cls, components: SolBalanceComponents, costs: SolActionCosts
    ) -> SolBalanceAvailability:
        """Return unwrap-all availability with the live fee as its entire reserve.

        This method rejects costs that include sponsorship, account creation, or
        upfront rent. It rejects components and fees outside Solana's unsigned
        64-bit lamport range. It rejects an overflowing displayed-balance sum.
        Native SOL must fund the fee without relying on lamports that a later
        ``CloseAccount`` instruction may transfer. The ordinary persistent-account
        floor does not apply because unwrap-all removes that account.
        """
        if (
            isinstance(costs.upfront_rent_lamports, bool)
            or not isinstance(costs.upfront_rent_lamports, int)
            or costs.upfront_rent_lamports != 0
        ):
            raise SdkError("unwrap-all costs require zero upfront rent")
        if costs.creates_canonical_wsol_account is not False:
            raise SdkError("unwrap-all costs must not create canonical WSOL")
        if costs.sponsored is not False:
            raise SdkError("unwrap-all costs must be unsponsored")
        fee_lamports = costs.fee_lamports
        for label, value in (
            ("native SOL", components.native_lamports),
            ("canonical WSOL", components.canonical_wsol_lamports),
            ("transaction fee", fee_lamports),
        ):
            if (
                isinstance(value, bool)
                or not isinstance(value, int)
                or value < 0
                or value > MAX_SOLANA_LAMPORTS
            ):
                raise SdkError(f"{label} must fit the non-negative u64 lamport range")
        displayed = components.displayed_lamports
        if displayed > MAX_SOLANA_LAMPORTS:
            raise SdkError("displayed SOL exceeds the transaction u64 range")
        if components.native_lamports < fee_lamports:
            raise SdkError(
                "native SOL balance cannot fund the required "
                f"{fee_lamports} lamport unwrap-all transaction fee"
            )
        return cls(
            components=components,
            displayed_lamports=displayed,
            reserve_lamports=fee_lamports,
            spendable_lamports=displayed - fee_lamports,
        )


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

    #: Wallet owning the current complete baseline, or None before initialization.
    wallet_address: str | None = None
    #: Slot of the last accepted observation, not a globally monotonic version.
    context_slot: int | None = None
    #: Sparse mint-keyed SPL balances; native SOL is never inserted here.
    balances: dict[str, DepositTokenBalance] = field(default_factory=dict)
    #: Exact nine-decimal native SOL text, or None before initialization.
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

    def sol_components(self) -> SolBalanceComponents:
        """Return exact native and canonical WSOL transaction components."""
        try:
            native_lamports = self.native_sol_lamports()
            canonical_wsol_lamports = self.canonical_wsol_lamports()
        except ValueError as error:
            raise SdkError(f"invalid SOL balance component: {error}") from error
        if (
            native_lamports > MAX_SOLANA_LAMPORTS
            or canonical_wsol_lamports > MAX_SOLANA_LAMPORTS
        ):
            raise SdkError("SOL component exceeds the transaction u64 range")
        return SolBalanceComponents(native_lamports, canonical_wsol_lamports)

    def native_sol_lamports(self) -> int:
        """Scale cached native SOL exactly; transaction range is checked elsewhere."""
        if self.native_sol_balance is None:
            raise ValueError("wallet balance state is not initialized")
        return int(exact_scaled_integer(self.native_sol_balance, 9))

    def canonical_wsol_lamports(self) -> int:
        """Scale the canonical WSOL idle balance exactly to lamports."""
        wrapped = self.balances.get(WRAPPED_SOL_MINT)
        return exact_scaled_integer(wrapped.idle, 9) if wrapped is not None else 0

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
