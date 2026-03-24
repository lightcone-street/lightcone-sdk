"""Positions sub-client — portfolio, position queries, PDA helpers, ix/tx builders, and on-chain ops."""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING

from solders.instruction import Instruction
from solders.pubkey import Pubkey
from solders.transaction import Transaction

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
from .wire import PositionsResponseWire, MarketPositionsResponseWire
from ...program.accounts import deserialize_position
from ...program.instructions import (
    build_deposit_to_global_instruction,
    build_extend_position_tokens_instruction,
    build_global_to_market_deposit_instruction,
    build_init_position_tokens_instruction,
    build_redeem_winnings_instruction,
    build_withdraw_from_global_instruction,
    build_withdraw_from_position_instruction,
)
from ...program.pda import get_position_pda
from ...program.types import (
    DepositToGlobalParams,
    ExtendPositionTokensParams,
    GlobalToMarketDepositParams,
    InitPositionTokensParams,
    Position as OnchainPosition,
    RedeemWinningsParams,
    WithdrawFromGlobalParams,
    WithdrawFromPositionParams,
)
from ...rpc import require_connection

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

    # ── On-chain instruction builders ────────────────────────────────────

    def redeem_winnings_ix(
        self, params: RedeemWinningsParams, winning_outcome: int
    ) -> Instruction:
        """Build RedeemWinnings instruction."""
        return build_redeem_winnings_instruction(
            user=params.user,
            market=params.market,
            deposit_mint=params.deposit_mint,
            winning_outcome=winning_outcome,
            amount=params.amount,
            program_id=self._client.program_id,
        )

    def withdraw_from_position_ix(
        self, params: WithdrawFromPositionParams, is_token_2022: bool = True
    ) -> Instruction:
        """Build WithdrawFromPosition instruction."""
        return build_withdraw_from_position_instruction(
            user=params.user,
            market=params.market,
            mint=params.mint,
            amount=params.amount,
            outcome_index=params.outcome_index,
            is_token_2022=is_token_2022,
            program_id=self._client.program_id,
        )

    def init_position_tokens_ix(
        self, params: InitPositionTokensParams, num_outcomes: int
    ) -> Instruction:
        """Build InitPositionTokens instruction."""
        return build_init_position_tokens_instruction(
            params, num_outcomes, self._client.program_id
        )

    def extend_position_tokens_ix(
        self, params: ExtendPositionTokensParams, num_outcomes: int
    ) -> Instruction:
        """Build ExtendPositionTokens instruction."""
        return build_extend_position_tokens_instruction(
            params, num_outcomes, self._client.program_id
        )

    def deposit_to_global_ix(self, params: DepositToGlobalParams) -> Instruction:
        """Build DepositToGlobal instruction."""
        return build_deposit_to_global_instruction(
            user=params.user,
            mint=params.mint,
            amount=params.amount,
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
        self, params: RedeemWinningsParams, winning_outcome: int
    ) -> Transaction:
        """Build RedeemWinnings transaction."""
        ix = self.redeem_winnings_ix(params, winning_outcome)
        return Transaction.new_with_payer([ix], params.user)

    def withdraw_from_position_tx(
        self, params: WithdrawFromPositionParams, is_token_2022: bool = True
    ) -> Transaction:
        """Build WithdrawFromPosition transaction."""
        ix = self.withdraw_from_position_ix(params, is_token_2022)
        return Transaction.new_with_payer([ix], params.user)

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
        return Transaction.new_with_payer([ix], params.payer)

    def deposit_to_global_tx(self, params: DepositToGlobalParams) -> Transaction:
        """Build DepositToGlobal transaction."""
        ix = self.deposit_to_global_ix(params)
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
        """Create a withdraw-from-position builder."""
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

    async def get_onchain(
        self, owner: Pubkey, market: Pubkey
    ) -> Optional[OnchainPosition]:
        """Fetch a Position account (returns None if not found)."""
        conn = require_connection(self._client)
        addr = self.pda(owner, market)
        response = await conn.get_account_info(addr)
        if response.value is None:
            return None
        return deserialize_position(response.value.data)
