"""Positions sub-client — portfolio, position queries, PDA helpers, tx builders, and on-chain ops."""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING

from solders.instruction import Instruction
from solders.pubkey import Pubkey

from .wire import PositionsResponseWire, MarketPositionsResponseWire
from ...program.accounts import deserialize_position
from ...program.instructions import (
    build_deposit_to_global_instruction,
    build_extend_position_tokens_instruction,
    build_global_to_market_deposit_instruction,
    build_init_position_tokens_instruction,
    build_redeem_winnings_instruction,
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

    # ── On-chain transaction builders ────────────────────────────────────

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
