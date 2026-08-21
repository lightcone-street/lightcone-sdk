"""Positions sub-client — portfolio, position queries, PDA helpers, ix/tx builders, and on-chain ops."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from enum import Enum
from typing import TYPE_CHECKING

from solders.hash import Hash
from solders.instruction import Instruction
from solders.pubkey import Pubkey
from solders.system_program import (
    CreateAccountWithSeedParams,
    TransferParams,
    create_account_with_seed,
)
from solders.system_program import (
    transfer as system_transfer,
)
from solders.transaction import Transaction
from spl.token.constants import TOKEN_PROGRAM_ID, WRAPPED_SOL_MINT
from spl.token.instructions import (
    CloseAccountParams,
    InitializeAccount3Params,
    SyncNativeParams,
    close_account,
    create_idempotent_associated_token_account,
    get_associated_token_address,
    initialize_account3,
    sync_native,
)
from spl.token.instructions import (
    TransferParams as TokenTransferParams,
)
from spl.token.instructions import (
    transfer as token_transfer,
)

from ...error import SdkError
from ...program.accounts import deserialize_position
from ...program.instructions import (
    build_close_position_alt_instruction,
    build_close_position_token_accounts_instruction,
    build_deposit_instruction,
    build_deposit_to_global_instruction,
    build_deposit_to_global_instruction_with_alt,
    build_extend_position_tokens_instruction,
    build_global_to_market_deposit_instruction,
    build_init_position_tokens_instruction,
    build_merge_instruction,
    build_redeem_winnings_instruction,
    build_withdraw_conditional_from_position_instruction,
    build_withdraw_from_global_instruction,
)
from ...program.pda import get_position_pda
from ...program.types import (
    ClosePositionAltParams,
    ClosePositionTokenAccountsParams,
    DepositToGlobalAltContext,
    DepositToGlobalParams,
    ExtendPositionTokensParams,
    GlobalToMarketDepositParams,
    InitPositionTokensParams,
    Position,
    RedeemWinningsParams,
    WithdrawConditionalFromPositionParams,
    WithdrawFromGlobalParams,
    WithdrawFromPositionParams,
)
from ...program.utils import validate_outcome_count, validate_outcome_index
from ...rpc import require_connection
from ..market import Market
from . import DepositTokenBalancesSnapshot
from .builders import (
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
from .state import (
    SolActionCosts,
    SolBalanceAvailability,
    SolBalanceComponents,
    WalletDepositBalancesState,
)
from .wire import MarketPositionsResponseWire, PositionsResponseWire

if TYPE_CHECKING:
    from ...client import LightconeClient
    from ...rpc import Rpc


#: Byte allocation for a legacy SPL Token Program (Tokenkeg) account.
TOKEN_ACCOUNT_SPACE = 165
#: Largest exact lamport amount accepted by Solana transaction instructions.
MAX_U64 = 2**64 - 1


def _require_sol_action_amount(amount_lamports: int, action: str) -> None:
    """Reject non-positive or non-u64 lamport amounts before any RPC side effect."""
    if isinstance(amount_lamports, bool) or not isinstance(amount_lamports, int):
        raise SdkError(f"{action} amount must be an integer number of lamports")
    if amount_lamports <= 0:
        raise SdkError(f"{action} amount must be greater than zero")
    if amount_lamports > MAX_U64:
        raise SdkError(f"{action} amount must fit u64")


def _require_unsponsored_plan(sponsored: bool) -> None:
    """Reject sponsorship until a concrete sponsor owns fees and account rent."""
    if sponsored:
        raise SdkError("sponsored SOL action planning is not supported")


@dataclass(frozen=True)
class SolComponentDelta:
    """Expected changes to separately authoritative native and canonical balances."""

    #: System-account change in lamports, including unsponsored costs.
    native_lamports: int
    #: Persistent canonical WSOL ATA change in lamports.
    canonical_wsol_lamports: int


class SolActionKind(str, Enum):
    """SOL-aware operation represented by an action plan."""

    #: Mint a complete conditional-token set, wrapping only a WSOL shortfall.
    SPLIT = "split"
    #: Burn a complete set and retain returned collateral as canonical WSOL.
    MERGE = "merge"
    #: Redeem winning tokens and retain returned collateral as canonical WSOL.
    REDEEM = "redeem"
    #: Deliver exact native lamports, converting canonical WSOL only if needed.
    NATIVE_WITHDRAW = "native_withdraw"


@dataclass(frozen=True)
class SolActionPlan:
    """Unsigned transaction and exact preflight facts authorizing one SOL action."""

    #: Operation whose balance semantics produced this plan.
    kind: SolActionKind
    #: Fee-prepared message that submission must preserve exactly.
    transaction: Transaction
    #: Live fee/rent observations and explicit sponsorship capability.
    costs: SolActionCosts
    #: Component totals after action-specific native reserve.
    availability: SolBalanceAvailability
    #: Component-wise projection that does not replace authoritative state.
    expected_delta: SolComponentDelta


def native_withdraw_seed(
    recent_blockhash: Hash,
    wallet: Pubkey,
    recipient: Pubkey,
    amount_lamports: int,
    attempt: int,
) -> str:
    """Derive the exact bounded temporary-account seed shared by all SDKs.

    SHA-256 receives the ASCII domain ``lightcone:wsol-withdraw:v1``, one zero
    byte, raw 32-byte blockhash, wallet, and recipient keys, the amount as
    unsigned eight-byte big-endian lamports, then the one-byte attempt. This
    exact order is shared by all three SDKs. The first 16 digest bytes become 32
    lowercase hexadecimal ASCII characters to satisfy Solana's seed limit.
    """
    if not 0 <= amount_lamports <= MAX_U64:
        raise SdkError("withdraw amount must fit u64")
    if not 0 <= attempt <= 255:
        raise SdkError("temporary WSOL seed attempt must fit u8")
    preimage = b"".join(
        [
            b"lightcone:wsol-withdraw:v1",
            b"\x00",
            bytes(recent_blockhash),
            bytes(wallet),
            bytes(recipient),
            amount_lamports.to_bytes(8, "big"),
            bytes([attempt]),
        ]
    )
    return hashlib.sha256(preimage).digest()[:16].hex()


class Positions:
    """Position operations sub-client."""

    def __init__(self, client: LightconeClient):
        """Bind position operations to one client's auth, RPC, and program authority."""
        self._client = client

    # ── PDA helpers ──────────────────────────────────────────────────────

    def pda(self, owner: Pubkey, market: Pubkey) -> Pubkey:
        """Get the Position PDA."""
        addr, _ = get_position_pda(owner, market, self._client.program_id)
        return addr

    # ── HTTP methods ─────────────────────────────────────────────────────

    async def get(self, user_pubkey: str) -> PositionsResponseWire:
        """Get all positions for a user."""
        data = await self._client._http.get(f"/api/users/{user_pubkey}/positions")
        return PositionsResponseWire.from_dict(data)

    async def get_for_market(
        self,
        user_pubkey: str,
        market_pubkey: str,
    ) -> MarketPositionsResponseWire:
        """Get positions in a specific market."""
        data = await self._client._http.get(
            f"/api/users/{user_pubkey}/markets/{market_pubkey}/positions"
        )
        return MarketPositionsResponseWire.from_dict(data)

    async def positions(self) -> PositionsResponseWire:
        """Get all conditional-token positions for the authenticated user.

        Wallet is resolved server-side from the ``cookie_header`` cookie; no
        parameter required. Same response shape as ``get(wallet)``.

        GET /api/users/positions
        """
        data = await self._client._http.get("/api/users/positions")
        return PositionsResponseWire.from_dict(data)

    async def positions_with_cookies(self, cookie_header: str) -> PositionsResponseWire:
        """Same as :meth:`positions`, with an explicit per-call ``cookie_header``.

        Intended for server-side cookie forwarding (SSR / server functions)
        where the per-request browser cookie can't propagate to the SDK's
        process-wide cookie store. The token is used only for this call and
        never written back to the shared store.
        """
        data = await self._client._http.get_with_cookies(
            "/api/users/positions",
            cookie_header=cookie_header,
        )
        return PositionsResponseWire.from_dict(data)

    async def positions_for_market(
        self, market_pubkey: str
    ) -> MarketPositionsResponseWire:
        """Get the authenticated user's positions in a specific market.

        Wallet is resolved server-side from the ``cookie_header`` cookie.

        GET /api/users/markets/{market_pubkey}/positions
        """
        data = await self._client._http.get(
            f"/api/users/markets/{market_pubkey}/positions"
        )
        return MarketPositionsResponseWire.from_dict(data)

    async def positions_for_market_with_cookies(
        self, market_pubkey: str, cookie_header: str
    ) -> MarketPositionsResponseWire:
        """Same as :meth:`positions_for_market`, with an explicit per-call ``cookie_header``.

        For server-side cookie forwarding (SSR / route handlers).
        """
        data = await self._client._http.get_with_cookies(
            f"/api/users/markets/{market_pubkey}/positions",
            cookie_header=cookie_header,
        )
        return MarketPositionsResponseWire.from_dict(data)

    async def deposit_token_balances(
        self, min_context_slot: int | None = None
    ) -> DepositTokenBalancesSnapshot:
        """Fetch a complete authenticated SPL and native-SOL snapshot.

        ``min_context_slot`` lower-bounds the complete cross-component view.
        Native SOL is canonical nine-decimal text outside the SPL map.
        """
        params = (
            {"min_context_slot": str(min_context_slot)}
            if min_context_slot is not None
            else None
        )
        data = await self._client._http.get(
            "/api/users/deposit-token-balances",
            params=params,
        )
        return DepositTokenBalancesSnapshot.from_dict(data)

    async def deposit_token_balances_with_cookies(
        self,
        min_context_slot: int | None,
        cookie_header: str,
    ) -> DepositTokenBalancesSnapshot:
        """Fetch the complete snapshot with an explicit per-call cookie.

        Intended for server-side cookie forwarding (SSR / server functions)
        where the per-request browser cookie can't propagate to the SDK's
        process-wide cookie store. The token is used only for this call and
        never written back to the shared store. Snapshot and minimum-slot
        semantics match :meth:`deposit_token_balances`.
        """
        params = (
            {"min_context_slot": str(min_context_slot)}
            if min_context_slot is not None
            else None
        )
        data = await self._client._http.get_with_cookies(
            "/api/users/deposit-token-balances",
            cookie_header=cookie_header,
            params=params,
        )
        return DepositTokenBalancesSnapshot.from_dict(data)

    async def plan_sol_split(
        self,
        market: Market,
        amount_lamports: int,
        state: WalletDepositBalancesState,
        sponsored: bool,
    ) -> SolActionPlan:
        """Plan one atomic split using canonical WSOL before wrapping a shortfall.

        Amounts and live costs are lamports. Account, fee, and rent reads fail
        closed, and sponsored planning is rejected until a sponsor owns costs.
        """
        _require_unsponsored_plan(sponsored)
        _require_sol_action_amount(amount_lamports, "split")
        wallet = self._planning_wallet(state)
        components = state.sol_components()
        rpc = self._client.rpc()
        canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)
        canonical_exists = await rpc.account_exists(canonical)
        if components.canonical_wsol_lamports > 0 and not canonical_exists:
            raise SdkError(
                "canonical WSOL balance is positive but its account is unavailable"
            )
        shortfall = max(amount_lamports - components.canonical_wsol_lamports, 0)
        rent = (
            0
            if canonical_exists
            else await rpc.minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_SPACE)
        )
        instructions = []
        if not canonical_exists:
            instructions.append(self._create_canonical_wsol_account(wallet))
        if shortfall > 0:
            instructions.extend(
                [
                    system_transfer(
                        TransferParams(
                            from_pubkey=wallet,
                            to_pubkey=canonical,
                            lamports=shortfall,
                        )
                    ),
                    sync_native(
                        SyncNativeParams(program_id=TOKEN_PROGRAM_ID, account=canonical)
                    ),
                ]
            )
        instructions.append(
            build_deposit_instruction(
                user=wallet,
                market=Pubkey.from_string(market.pubkey),
                deposit_mint=WRAPPED_SOL_MINT,
                amount=amount_lamports,
                num_outcomes=market.num_outcomes,
                program_id=self._client.program_id,
            )
        )
        transaction = Transaction.new_with_payer(instructions, wallet)
        fee = await rpc.prepare_and_estimate_transaction_fee(transaction)
        costs = SolActionCosts(fee, rent, not canonical_exists, sponsored)
        availability = SolBalanceAvailability.from_costs(components, costs)
        if amount_lamports > availability.spendable_lamports:
            raise SdkError(
                "split amount exceeds spendable SOL after transaction reserve"
            )
        if shortfall + availability.reserve_lamports > components.native_lamports:
            raise SdkError(
                "native SOL cannot fund the wrap shortfall and transaction reserve"
            )
        wallet_costs = 0 if sponsored else fee + rent
        return SolActionPlan(
            SolActionKind.SPLIT,
            transaction,
            costs,
            availability,
            SolComponentDelta(-shortfall - wallet_costs, shortfall - amount_lamports),
        )

    async def plan_sol_merge(
        self,
        market: Market,
        amount_lamports: int,
        state: WalletDepositBalancesState,
        sponsored: bool,
    ) -> SolActionPlan:
        """Plan a merge that retains returned WSOL in the canonical ATA.

        The fee-prepared transaction does not mutate cached state; callers
        refresh authority after confirmed submission.
        """
        _require_unsponsored_plan(sponsored)
        _require_sol_action_amount(amount_lamports, "merge")
        wallet = self._planning_wallet(state)
        rpc, components, exists, rent = await self._receive_plan_context(wallet, state)
        instructions = [] if exists else [self._create_canonical_wsol_account(wallet)]
        instructions.append(
            build_merge_instruction(
                user=wallet,
                market=Pubkey.from_string(market.pubkey),
                deposit_mint=WRAPPED_SOL_MINT,
                amount=amount_lamports,
                num_outcomes=market.num_outcomes,
                program_id=self._client.program_id,
            )
        )
        return await self._finish_receive_plan(
            SolActionKind.MERGE,
            amount_lamports,
            Transaction.new_with_payer(instructions, wallet),
            rpc,
            components,
            rent,
            not exists,
            sponsored,
        )

    async def plan_sol_redeem(
        self,
        market: Pubkey,
        amount_lamports: int,
        outcome_index: int,
        num_outcomes: int,
        state: WalletDepositBalancesState,
        sponsored: bool,
    ) -> SolActionPlan:
        """Plan a redemption that retains returned WSOL in the canonical ATA.

        ``amount_lamports`` is exact collateral scale; ``outcome_index`` is
        validated against the supplied authoritative ``num_outcomes``.
        """
        _require_unsponsored_plan(sponsored)
        _require_sol_action_amount(amount_lamports, "redeem")
        validate_outcome_count(num_outcomes)
        validate_outcome_index(outcome_index, num_outcomes)
        wallet = self._planning_wallet(state)
        rpc, components, exists, rent = await self._receive_plan_context(wallet, state)
        instructions = [] if exists else [self._create_canonical_wsol_account(wallet)]
        instructions.append(
            build_redeem_winnings_instruction(
                user=wallet,
                market=market,
                deposit_mint=WRAPPED_SOL_MINT,
                outcome_index=outcome_index,
                amount=amount_lamports,
                program_id=self._client.program_id,
            )
        )
        return await self._finish_receive_plan(
            SolActionKind.REDEEM,
            amount_lamports,
            Transaction.new_with_payer(instructions, wallet),
            rpc,
            components,
            rent,
            not exists,
            sponsored,
        )

    async def plan_native_sol_withdrawal(
        self,
        recipient: Pubkey,
        amount_lamports: int,
        state: WalletDepositBalancesState,
        sponsored: bool,
    ) -> SolActionPlan:
        """Plan exact native SOL delivery without closing the canonical ATA.

        Native funds are preferred. A shortfall passes through a bounded seeded
        Tokenkeg account whose rent returns on close. Account, rent, and fee
        authority fail closed. At most eight blockhash-scoped candidates bound
        RPC latency while making accidental exhaustion negligible. The returned
        transaction is fee-prepared.
        """
        _require_unsponsored_plan(sponsored)
        _require_sol_action_amount(amount_lamports, "withdraw")
        wallet = self._planning_wallet(state)
        components = state.sol_components()
        rpc = self._client.rpc()
        direct = Transaction.new_with_payer(
            [
                system_transfer(
                    TransferParams(
                        from_pubkey=wallet,
                        to_pubkey=recipient,
                        lamports=amount_lamports,
                    )
                )
            ],
            wallet,
        )
        direct_fee = await rpc.prepare_and_estimate_transaction_fee(direct)
        direct_costs = SolActionCosts(direct_fee, 0, False, sponsored)
        direct_availability = SolBalanceAvailability.from_costs(
            components, direct_costs
        )
        if amount_lamports > direct_availability.spendable_lamports:
            raise SdkError(
                "withdraw amount exceeds spendable SOL after transaction reserve"
            )
        if (
            components.native_lamports
            >= amount_lamports + direct_availability.reserve_lamports
        ):
            return SolActionPlan(
                SolActionKind.NATIVE_WITHDRAW,
                direct,
                direct_costs,
                direct_availability,
                SolComponentDelta(
                    -amount_lamports - (0 if sponsored else direct_fee), 0
                ),
            )

        canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)
        if not await rpc.account_exists(canonical):
            raise SdkError("canonical WSOL is required for this native withdrawal")
        temporary_rent = await rpc.minimum_balance_for_rent_exemption(
            TOKEN_ACCOUNT_SPACE
        )
        blockhash = await rpc.get_latest_blockhash()
        selected: tuple[str, Pubkey] | None = None
        # Bound account-existence RPCs; blockhash plus attempt makes eight collisions remote.
        for attempt in range(8):
            seed = native_withdraw_seed(
                blockhash, wallet, recipient, amount_lamports, attempt
            )
            candidate = Pubkey.create_with_seed(wallet, seed, TOKEN_PROGRAM_ID)
            if not await rpc.account_exists(candidate):
                selected = seed, candidate
                break
        if selected is None:
            raise SdkError("temporary WSOL seed attempts are exhausted")
        seed, temporary = selected

        transaction = self._build_temporary_native_withdrawal(
            wallet,
            recipient,
            amount_lamports,
            1,
            temporary_rent,
            seed,
            temporary,
        )
        transaction.partial_sign([], blockhash)
        initial_fee = await rpc.estimate_prepared_transaction_fee(transaction)
        initial_costs = SolActionCosts(initial_fee, temporary_rent, False, sponsored)
        initial_availability = SolBalanceAvailability.from_costs(
            components, initial_costs
        )
        initial_required = amount_lamports + initial_availability.reserve_lamports
        if initial_required < components.native_lamports:
            raise SdkError("invalid temporary withdrawal requirement")
        initial_transfer = initial_required - components.native_lamports
        transaction = self._build_temporary_native_withdrawal(
            wallet,
            recipient,
            amount_lamports,
            initial_transfer,
            temporary_rent,
            seed,
            temporary,
        )
        transaction.partial_sign([], blockhash)
        final_fee = await rpc.estimate_prepared_transaction_fee(transaction)
        costs = SolActionCosts(final_fee, temporary_rent, False, sponsored)
        availability = SolBalanceAvailability.from_costs(components, costs)
        final_required = amount_lamports + availability.reserve_lamports
        if final_required < components.native_lamports:
            raise SdkError("invalid temporary withdrawal requirement")
        canonical_transfer = final_required - components.native_lamports
        if canonical_transfer > components.canonical_wsol_lamports:
            raise SdkError("canonical WSOL cannot fund the native withdrawal shortfall")
        if canonical_transfer != initial_transfer:
            transaction = self._build_temporary_native_withdrawal(
                wallet,
                recipient,
                amount_lamports,
                canonical_transfer,
                temporary_rent,
                seed,
                temporary,
            )
            transaction.partial_sign([], blockhash)
            stable_fee = await rpc.estimate_prepared_transaction_fee(transaction)
            if stable_fee != final_fee:
                raise SdkError(
                    "transaction fee changed while rebuilding native withdrawal"
                )
        return SolActionPlan(
            SolActionKind.NATIVE_WITHDRAW,
            transaction,
            costs,
            availability,
            SolComponentDelta(
                canonical_transfer - amount_lamports - (0 if sponsored else final_fee),
                -canonical_transfer,
            ),
        )

    def _planning_wallet(self, state: WalletDepositBalancesState) -> Pubkey:
        """Validate the cached identity boundary before planning a transaction."""
        credentials = self._client.auth().credentials()
        if credentials is None:
            raise SdkError("authenticated credentials are required")
        if not credentials.is_authenticated():
            raise SdkError("authenticated credentials have expired")
        if (
            state.wallet_address is None
            or state.context_slot is None
            or state.native_sol_balance is None
        ):
            raise SdkError("wallet balance state is not initialized")
        if state.wallet_address != credentials.wallet_address:
            raise SdkError("authenticated wallet does not match wallet balance state")
        try:
            wallet = Pubkey.from_string(credentials.wallet_address)
        except ValueError as error:
            raise SdkError(f"authenticated wallet is invalid: {error}") from error
        strategy = self._client._require_signing_strategy()
        signing_address = strategy.controlled_wallet_address()
        if signing_address is None:
            raise SdkError("signing strategy wallet identity is required")
        try:
            signing_wallet = Pubkey.from_string(signing_address)
        except (TypeError, ValueError) as error:
            raise SdkError(f"signing strategy wallet is invalid: {error}") from error
        if signing_wallet != wallet:
            raise SdkError("signing strategy does not control authenticated wallet")
        return wallet

    @staticmethod
    def _create_canonical_wsol_account(wallet: Pubkey) -> Instruction:
        """Build idempotent creation of the persistent Tokenkeg WSOL ATA.

        Tokenkeg is Solana's legacy SPL Token Program. Canonical native-mint ATA
        derivation is pinned to it rather than Token-2022 across the protocol.
        """
        return create_idempotent_associated_token_account(
            wallet, wallet, WRAPPED_SOL_MINT, TOKEN_PROGRAM_ID
        )

    async def _receive_plan_context(
        self, wallet: Pubkey, state: WalletDepositBalancesState
    ) -> tuple[Rpc, SolBalanceComponents, bool, int]:
        """Read canonical-account existence and upfront rent for receive plans."""
        rpc = self._client.rpc()
        canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)
        exists = await rpc.account_exists(canonical)
        components = state.sol_components()
        if components.canonical_wsol_lamports > 0 and not exists:
            raise SdkError(
                "canonical WSOL balance is positive but its account is unavailable"
            )
        rent = (
            0
            if exists
            else await rpc.minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_SPACE)
        )
        return rpc, components, exists, rent

    async def _finish_receive_plan(
        self,
        kind: SolActionKind,
        amount_lamports: int,
        transaction: Transaction,
        rpc: Rpc,
        components: SolBalanceComponents,
        rent: int,
        creates_canonical_account: bool,
        sponsored: bool,
    ) -> SolActionPlan:
        """Finish merge/redeem planning with live fee authority and deltas."""
        fee = await rpc.prepare_and_estimate_transaction_fee(transaction)
        costs = SolActionCosts(fee, rent, creates_canonical_account, sponsored)
        availability = SolBalanceAvailability.from_costs(components, costs)
        wallet_costs = 0 if sponsored else fee + rent
        return SolActionPlan(
            kind,
            transaction,
            costs,
            availability,
            SolComponentDelta(-wallet_costs, amount_lamports),
        )

    @staticmethod
    def _build_temporary_native_withdrawal(
        wallet: Pubkey,
        recipient: Pubkey,
        amount_lamports: int,
        canonical_transfer: int,
        temporary_rent: int,
        seed: str,
        temporary: Pubkey,
    ) -> Transaction:
        """Build the sole WSOL-to-native path without closing canonical authority.

        The temporary Tokenkeg account is initialized, funded, and closed back
        to the wallet before the exact recipient transfer in one transaction.
        """
        canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)
        return Transaction.new_with_payer(
            [
                create_account_with_seed(
                    CreateAccountWithSeedParams(
                        from_pubkey=wallet,
                        to_pubkey=temporary,
                        base=wallet,
                        seed=seed,
                        lamports=temporary_rent,
                        space=TOKEN_ACCOUNT_SPACE,
                        owner=TOKEN_PROGRAM_ID,
                    )
                ),
                initialize_account3(
                    InitializeAccount3Params(
                        program_id=TOKEN_PROGRAM_ID,
                        account=temporary,
                        mint=WRAPPED_SOL_MINT,
                        owner=wallet,
                    )
                ),
                token_transfer(
                    TokenTransferParams(
                        program_id=TOKEN_PROGRAM_ID,
                        source=canonical,
                        dest=temporary,
                        owner=wallet,
                        amount=canonical_transfer,
                        signers=[],
                    )
                ),
                close_account(
                    CloseAccountParams(
                        program_id=TOKEN_PROGRAM_ID,
                        account=temporary,
                        dest=wallet,
                        owner=wallet,
                    )
                ),
                system_transfer(
                    TransferParams(
                        from_pubkey=wallet,
                        to_pubkey=recipient,
                        lamports=amount_lamports,
                    )
                ),
            ],
            wallet,
        )

    # ── On-chain instruction builders ────────────────────────────────────

    def redeem_winnings_ix(
        self, params: RedeemWinningsParams, outcome_index: int
    ) -> Instruction:
        """Build RedeemWinnings instruction."""
        return build_redeem_winnings_instruction(
            user=params.user,
            market=params.market,
            deposit_mint=params.deposit_mint,
            outcome_index=outcome_index,
            amount=params.amount,
            program_id=self._client.program_id,
        )

    def withdraw_conditional_from_position_ix(
        self, params: WithdrawConditionalFromPositionParams
    ) -> Instruction:
        """Build conditional-token withdrawal from a position instruction."""
        return build_withdraw_conditional_from_position_instruction(
            user=params.user,
            market=params.market,
            deposit_mint=params.deposit_mint,
            amount=params.amount,
            outcome_index=params.outcome_index,
            program_id=self._client.program_id,
        )

    def withdraw_from_position_ix(
        self, params: WithdrawFromPositionParams
    ) -> Instruction:
        """Compatibility wrapper for conditional-token position withdrawal."""
        return self.withdraw_conditional_from_position_ix(params)

    def init_position_tokens_ix(
        self, params: InitPositionTokensParams, num_outcomes: int
    ) -> Instruction:
        """Build InitPositionTokens instruction."""
        return build_init_position_tokens_instruction(
            payer=params.payer,
            user=params.user,
            market=params.market,
            deposit_mints=params.deposit_mints,
            num_outcomes=num_outcomes,
            recent_slot=params.recent_slot,
            program_id=self._client.program_id,
        )

    def extend_position_tokens_ix(
        self, params: ExtendPositionTokensParams, num_outcomes: int
    ) -> Instruction:
        """Build ExtendPositionTokens instruction."""
        return build_extend_position_tokens_instruction(
            operator=params.operator,
            user=params.user,
            market=params.market,
            lookup_table=params.lookup_table,
            deposit_mints=params.deposit_mints,
            num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def close_position_alt_ix(self, params: ClosePositionAltParams) -> Instruction:
        """Build ClosePositionAlt instruction."""
        return build_close_position_alt_instruction(params, self._client.program_id)

    def close_position_token_accounts_ix(
        self,
        params: ClosePositionTokenAccountsParams,
        num_outcomes: int,
    ) -> Instruction:
        """Build ClosePositionTokenAccounts instruction."""
        return build_close_position_token_accounts_instruction(
            params,
            num_outcomes,
            self._client.program_id,
        )

    def deposit_to_global_ix(self, params: DepositToGlobalParams) -> Instruction:
        """Build DepositToGlobal instruction."""
        return build_deposit_to_global_instruction(
            user=params.user,
            mint=params.mint,
            amount=params.amount,
            program_id=self._client.program_id,
        )

    def deposit_to_global_ix_with_alt(
        self,
        params: DepositToGlobalParams,
        alt_context: DepositToGlobalAltContext,
    ) -> Instruction:
        """Build DepositToGlobal instruction with user-deposit ALT accounts."""
        return build_deposit_to_global_instruction_with_alt(
            user=params.user,
            mint=params.mint,
            amount=params.amount,
            alt_context=alt_context,
            program_id=self._client.program_id,
        )

    def global_to_market_deposit_ix(
        self, params: GlobalToMarketDepositParams, num_outcomes: int
    ) -> Instruction:
        """Build GlobalToMarketDeposit instruction."""
        return build_global_to_market_deposit_instruction(
            user=params.user,
            market=params.market,
            deposit_mint=params.deposit_mint,
            amount=params.amount,
            num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def withdraw_from_global_ix(self, params: WithdrawFromGlobalParams) -> Instruction:
        """Build WithdrawFromGlobal instruction."""
        return build_withdraw_from_global_instruction(
            user=params.user,
            mint=params.mint,
            amount=params.amount,
            program_id=self._client.program_id,
        )

    # ── On-chain transaction builders ────────────────────────────────────

    def redeem_winnings_tx(
        self, params: RedeemWinningsParams, outcome_index: int
    ) -> Transaction:
        """Build RedeemWinnings transaction."""
        ix = self.redeem_winnings_ix(params, outcome_index)
        return Transaction.new_with_payer([ix], params.user)

    def withdraw_conditional_from_position_tx(
        self, params: WithdrawConditionalFromPositionParams
    ) -> Transaction:
        """Build conditional-token withdrawal from a position transaction."""
        ix = self.withdraw_conditional_from_position_ix(params)
        return Transaction.new_with_payer([ix], params.user)

    def withdraw_from_position_tx(
        self, params: WithdrawFromPositionParams
    ) -> Transaction:
        """Compatibility wrapper for conditional-token position withdrawal."""
        return self.withdraw_conditional_from_position_tx(params)

    def init_position_tokens_tx(
        self, params: InitPositionTokensParams, num_outcomes: int
    ) -> Transaction:
        """Build InitPositionTokens transaction."""
        ix = self.init_position_tokens_ix(params, num_outcomes)
        return Transaction.new_with_payer([ix], params.payer)

    def extend_position_tokens_tx(
        self, params: ExtendPositionTokensParams, num_outcomes: int
    ) -> Transaction:
        """Build ExtendPositionTokens transaction."""
        ix = self.extend_position_tokens_ix(params, num_outcomes)
        return Transaction.new_with_payer([ix], params.operator)

    def close_position_alt_tx(self, params: ClosePositionAltParams) -> Transaction:
        """Build ClosePositionAlt transaction."""
        ix = self.close_position_alt_ix(params)
        return Transaction.new_with_payer([ix], params.operator)

    def close_position_token_accounts_tx(
        self,
        params: ClosePositionTokenAccountsParams,
        num_outcomes: int,
    ) -> Transaction:
        """Build ClosePositionTokenAccounts transaction."""
        ix = self.close_position_token_accounts_ix(params, num_outcomes)
        return Transaction.new_with_payer([ix], params.operator)

    def deposit_to_global_tx(self, params: DepositToGlobalParams) -> Transaction:
        """Build DepositToGlobal transaction."""
        ix = self.deposit_to_global_ix(params)
        return Transaction.new_with_payer([ix], params.user)

    def deposit_to_global_tx_with_alt(
        self,
        params: DepositToGlobalParams,
        alt_context: DepositToGlobalAltContext,
    ) -> Transaction:
        """Build DepositToGlobal transaction with user-deposit ALT accounts."""
        ix = self.deposit_to_global_ix_with_alt(params, alt_context)
        return Transaction.new_with_payer([ix], params.user)

    def global_to_market_deposit_tx(
        self, params: GlobalToMarketDepositParams, num_outcomes: int
    ) -> Transaction:
        """Build GlobalToMarketDeposit transaction."""
        ix = self.global_to_market_deposit_ix(params, num_outcomes)
        return Transaction.new_with_payer([ix], params.user)

    def withdraw_from_global_tx(self, params: WithdrawFromGlobalParams) -> Transaction:
        """Build WithdrawFromGlobal transaction."""
        ix = self.withdraw_from_global_ix(params)
        return Transaction.new_with_payer([ix], params.user)

    # ── Builder factories ────────────────────────────────────────────────

    def deposit(self) -> DepositBuilder:
        """Create a deposit builder pre-seeded with the client's deposit source.

        Use ``.build_ix()`` or ``.build_tx()`` to produce the final instruction/transaction.
        """
        return DepositBuilder(self._client, self._client.deposit_source)

    def withdraw(self) -> WithdrawBuilder:
        """Create a withdraw builder pre-seeded with the client's deposit source.

        Use ``.build_ix()`` or ``.build_tx()`` to produce the final instruction/transaction.
        """
        return WithdrawBuilder(self._client, self._client.deposit_source)

    def merge(self) -> MergeBuilder:
        """Create a merge builder for burning conditional tokens and releasing collateral."""
        return MergeBuilder(self._client)

    def redeem_winnings(self) -> RedeemWinningsBuilder:
        """Create a redeem winnings builder."""
        return RedeemWinningsBuilder(self._client)

    def withdraw_from_position(self) -> WithdrawFromPositionBuilder:
        """Create a builder that requires ``.num_outcomes(...)`` before building."""
        return WithdrawFromPositionBuilder(self._client)

    def withdraw_conditional_from_position(self) -> WithdrawFromPositionBuilder:
        """Create a builder that requires ``.num_outcomes(...)`` before building."""
        return WithdrawFromPositionBuilder(self._client)

    def init_position_tokens(self) -> InitPositionTokensBuilder:
        """Create an init-position-tokens builder."""
        return InitPositionTokensBuilder(self._client)

    def extend_position_tokens(self) -> ExtendPositionTokensBuilder:
        """Create an extend-position-tokens builder."""
        return ExtendPositionTokensBuilder(self._client)

    def deposit_to_global(self) -> DepositToGlobalBuilder:
        """Create a deposit-to-global builder."""
        return DepositToGlobalBuilder(self._client)

    def withdraw_from_global(self) -> WithdrawFromGlobalBuilder:
        """Create a withdraw-from-global builder."""
        return WithdrawFromGlobalBuilder(self._client)

    def global_to_market_deposit(self) -> GlobalToMarketDepositBuilder:
        """Create a global-to-market deposit builder."""
        return GlobalToMarketDepositBuilder(self._client)

    # ── On-chain account fetchers (require connection) ───────────────────

    async def get_onchain(self, owner: Pubkey, market: Pubkey) -> Position | None:
        """Fetch a Position account (returns None if not found)."""
        conn = require_connection(self._client)
        addr = self.pda(owner, market)
        response = await conn.get_account_info(addr)
        if response.value is None:
            return None
        return deserialize_position(response.value.data)
