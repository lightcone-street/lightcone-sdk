"""Positions sub-client — portfolio, position queries, PDA helpers, ix/tx builders, and on-chain ops."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

from solders.instruction import Instruction
from solders.pubkey import Pubkey
from solders.system_program import TransferParams, transfer
from solders.transaction import Transaction
from spl.token.constants import TOKEN_PROGRAM_ID, WRAPPED_SOL_MINT
from spl.token.instructions import (
    CloseAccountParams,
    SyncNativeParams,
    close_account,
    create_idempotent_associated_token_account,
    get_associated_token_address,
    sync_native,
)

from ...error import SdkError
from ...program.accounts import deserialize_position
from ...program.instructions import (
    build_close_position_alt_instruction,
    build_close_position_token_accounts_instruction,
    build_deposit_to_global_instruction,
    build_deposit_to_global_instruction_with_alt,
    build_extend_position_tokens_instruction,
    build_global_to_market_deposit_instruction,
    build_init_position_tokens_instruction,
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
from ...rpc import require_connection
from ...shared.scaling import ExactDecimal, ScalingError
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
from .state import WalletDepositBalancesState, sol_amount_to_lamports
from .wire import MarketPositionsResponseWire, PositionsResponseWire

if TYPE_CHECKING:
    from ...client import LightconeClient


class Positions:
    """Position operations sub-client."""

    def __init__(self, client: "LightconeClient"):
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
        self, min_context_slot: Optional[int] = None
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
        min_context_slot: Optional[int],
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

    async def wrap_sol(
        self, amount: ExactDecimal, state: WalletDepositBalancesState
    ) -> str:
        """Wrap exact SOL into the authenticated wallet's canonical WSOL ATA.

        The amount must be positive, exactly representable at nine decimals, fit
        Solana's ``u64`` range, and not exceed cached native SOL. Live credentials
        must match initialized state, and the configured signing strategy must
        control that wallet. Maintained create/transfer/sync builders are submitted
        through confirmation; the returned string is the transaction signature and
        cached state is never mutated. A confirmation error does not prove rollback;
        refresh authoritative state before retrying.
        """
        wallet = self._conversion_wallet(state)
        try:
            lamports = sol_amount_to_lamports(amount)
            native_lamports = state.native_sol_lamports()
        except (ScalingError, ValueError) as error:
            raise SdkError(f"invalid SOL amount: {error}") from error
        if lamports <= 0:
            raise SdkError("wrap amount must be greater than zero")
        # Do not guess a fee or ATA-rent reserve from stale client state; an
        # equal-balance wrap is valid preflight and the chain remains authoritative.
        if lamports > native_lamports:
            raise SdkError("wrap amount exceeds cached native SOL balance")

        account = get_associated_token_address(wallet, WRAPPED_SOL_MINT)
        transaction = Transaction.new_with_payer(
            [
                create_idempotent_associated_token_account(
                    wallet, wallet, WRAPPED_SOL_MINT, TOKEN_PROGRAM_ID
                ),
                transfer(
                    TransferParams(
                        from_pubkey=wallet,
                        to_pubkey=account,
                        lamports=lamports,
                    )
                ),
                sync_native(
                    SyncNativeParams(program_id=TOKEN_PROGRAM_ID, account=account)
                ),
            ],
            wallet,
        )
        return await self._client.sign_and_submit_tx_confirmed(transaction)

    async def unwrap_wsol(self, state: WalletDepositBalancesState) -> str:
        """Fully unwrap the authenticated wallet's canonical Tokenkeg WSOL ATA.

        Live matching credentials, a signing strategy controlling that wallet, and
        positive cached WSOL are required. Closing credits the complete token balance
        plus rent to the wallet; partial unwrap is unsupported. The method returns a
        confirmed transaction signature and leaves cached state unchanged. A
        confirmation error does not prove the account stayed open; refresh
        authoritative state before retrying.
        """
        wallet = self._conversion_wallet(state)
        try:
            has_wsol = state.has_positive_wsol()
        except ScalingError as error:
            raise SdkError(f"invalid canonical WSOL balance: {error}") from error
        if not has_wsol:
            raise SdkError("canonical WSOL balance must be greater than zero")

        account = get_associated_token_address(wallet, WRAPPED_SOL_MINT)
        transaction = Transaction.new_with_payer(
            [
                close_account(
                    CloseAccountParams(
                        program_id=TOKEN_PROGRAM_ID,
                        account=account,
                        dest=wallet,
                        owner=wallet,
                    )
                )
            ],
            wallet,
        )
        return await self._client.sign_and_submit_tx_confirmed(transaction)

    def _conversion_wallet(self, state: WalletDepositBalancesState) -> Pubkey:
        """Validate the cached identity/state signing boundary.

        Matching proves which wallet may sign against the cached preflight; it
        does not claim that the balance snapshot is still fresh on-chain.
        """
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
            return Pubkey.from_string(credentials.wallet_address)
        except ValueError as error:
            raise SdkError(f"authenticated wallet is invalid: {error}") from error

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

    async def get_onchain(self, owner: Pubkey, market: Pubkey) -> Optional[Position]:
        """Fetch a Position account (returns None if not found)."""
        conn = require_connection(self._client)
        addr = self.pda(owner, market)
        response = await conn.get_account_info(addr)
        if response.value is None:
            return None
        return deserialize_position(response.value.data)
