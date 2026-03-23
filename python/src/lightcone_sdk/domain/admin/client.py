"""Admin sub-client — metadata, referral management, and on-chain admin operations."""

from __future__ import annotations

from typing import TYPE_CHECKING

from solders.instruction import Instruction
from solders.pubkey import Pubkey
from solders.transaction import Transaction

from . import (
    AdminEnvelope,
    AllocateCodesResponse,
    CreateNotificationResponse,
    DismissNotificationResponse,
    RevokeResponse,
    UnifiedMetadataResponse,
    UnrevokeResponse,
    WhitelistResponse,
)
from ...program.instructions import (
    build_activate_market_instruction,
    build_add_deposit_mint_instruction,
    build_create_market_instruction,
    build_create_orderbook_instruction,
    build_deposit_and_swap_instruction,
    build_initialize_instruction,
    build_match_orders_multi_instruction,
    build_set_authority_instruction,
    build_set_operator_instruction,
    build_set_paused_instruction,
    build_settle_market_instruction,
    build_whitelist_deposit_token_instruction,
)
from ...program.types import (
    ActivateMarketParams,
    AddDepositMintParams,
    CreateOrderbookParams,
    DepositAndSwapParams,
    MatchOrdersMultiParams,
    SetAuthorityParams,
    SettleMarketParams,
    WhitelistDepositTokenParams,
)

if TYPE_CHECKING:
    from ...client import LightconeClient


class Admin:
    """Admin operations sub-client."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    # ── HTTP methods ─────────────────────────────────────────────────────

    async def upsert_metadata(self, envelope: AdminEnvelope) -> UnifiedMetadataResponse:
        """Upsert market/token metadata."""
        data = await self._client._http.post("/api/admin/metadata", envelope.to_dict())
        return UnifiedMetadataResponse.from_dict(data)

    async def allocate_codes(self, envelope: AdminEnvelope) -> AllocateCodesResponse:
        """Allocate referral codes."""
        data = await self._client._http.post("/api/admin/referral/allocate", envelope.to_dict())
        return AllocateCodesResponse.from_dict(data)

    async def whitelist(self, envelope: AdminEnvelope) -> WhitelistResponse:
        """Whitelist wallet addresses."""
        data = await self._client._http.post("/api/admin/referral/whitelist", envelope.to_dict())
        return WhitelistResponse.from_dict(data)

    async def revoke(self, envelope: AdminEnvelope) -> RevokeResponse:
        """Revoke access."""
        data = await self._client._http.post("/api/admin/referral/revoke", envelope.to_dict())
        return RevokeResponse.from_dict(data)

    async def unrevoke(self, envelope: AdminEnvelope) -> UnrevokeResponse:
        """Unrevoke access."""
        data = await self._client._http.post("/api/admin/referral/unrevoke", envelope.to_dict())
        return UnrevokeResponse.from_dict(data)

    async def create_notification(self, envelope: AdminEnvelope) -> CreateNotificationResponse:
        """Create a notification."""
        data = await self._client._http.post("/api/admin/notifications", envelope.to_dict())
        return CreateNotificationResponse.from_dict(data)

    async def dismiss_notification(
        self,
        envelope: AdminEnvelope,
    ) -> DismissNotificationResponse:
        """Dismiss a notification."""
        data = await self._client._http.post("/api/admin/notifications/dismiss", envelope.to_dict())
        return DismissNotificationResponse.from_dict(data)

    # ── On-chain instruction builders ────────────────────────────────────

    def initialize_ix(self, authority: Pubkey) -> Instruction:
        """Build Initialize instruction."""
        return build_initialize_instruction(authority, self._client.program_id)

    async def create_market_ix(
        self,
        authority: Pubkey,
        num_outcomes: int,
        oracle: Pubkey,
        question_id: bytes,
    ) -> Instruction:
        """Build CreateMarket instruction.

        Async because it fetches the next market ID from on-chain state.
        """
        market_id = await self._client.markets().next_id()
        return build_create_market_instruction(
            authority=authority,
            market_id=market_id,
            num_outcomes=num_outcomes,
            oracle=oracle,
            question_id=question_id,
            program_id=self._client.program_id,
        )

    def add_deposit_mint_ix(
        self,
        params: AddDepositMintParams,
        market: Pubkey,
        num_outcomes: int,
    ) -> Instruction:
        """Build AddDepositMint instruction."""
        return build_add_deposit_mint_instruction(
            authority=params.authority,
            market=market,
            deposit_mint=params.deposit_mint,
            outcome_metadata=params.outcome_metadata,
            num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def activate_market_ix(self, params: ActivateMarketParams) -> Instruction:
        """Build ActivateMarket instruction."""
        return build_activate_market_instruction(
            authority=params.authority,
            market_id=params.market_id,
            program_id=self._client.program_id,
        )

    def settle_market_ix(self, params: SettleMarketParams) -> Instruction:
        """Build SettleMarket instruction."""
        return build_settle_market_instruction(
            oracle=params.oracle,
            market_id=params.market_id,
            winning_outcome=params.winning_outcome,
            program_id=self._client.program_id,
        )

    def set_paused_ix(self, authority: Pubkey, paused: bool) -> Instruction:
        """Build SetPaused instruction."""
        return build_set_paused_instruction(authority, paused, self._client.program_id)

    def set_operator_ix(self, authority: Pubkey, new_operator: Pubkey) -> Instruction:
        """Build SetOperator instruction."""
        return build_set_operator_instruction(
            authority, new_operator, self._client.program_id
        )

    def set_authority_ix(self, params: SetAuthorityParams) -> Instruction:
        """Build SetAuthority instruction."""
        return build_set_authority_instruction(
            current_authority=params.current_authority,
            new_authority=params.new_authority,
            program_id=self._client.program_id,
        )

    def whitelist_deposit_token_ix(self, params: WhitelistDepositTokenParams) -> Instruction:
        """Build WhitelistDepositToken instruction."""
        return build_whitelist_deposit_token_instruction(
            authority=params.authority,
            mint=params.mint,
            program_id=self._client.program_id,
        )

    def create_orderbook_ix(self, params: CreateOrderbookParams) -> Instruction:
        """Build CreateOrderbook instruction."""
        return build_create_orderbook_instruction(
            payer=params.authority,
            market=params.market,
            mint_a=params.mint_a,
            mint_b=params.mint_b,
            recent_slot=params.recent_slot,
            program_id=self._client.program_id,
        )

    def match_orders_multi_ix(self, params: MatchOrdersMultiParams) -> Instruction:
        """Build MatchOrdersMulti instruction."""
        return build_match_orders_multi_instruction(
            operator=params.operator,
            market=params.market,
            base_mint=params.base_mint,
            quote_mint=params.quote_mint,
            taker_order=params.taker_order,
            maker_orders=params.maker_orders,
            maker_fill_amounts=params.maker_fill_amounts,
            taker_fill_amounts=params.taker_fill_amounts,
            full_fill_bitmask=params.full_fill_bitmask,
            program_id=self._client.program_id,
        )

    def deposit_and_swap_ix(self, params: DepositAndSwapParams) -> Instruction:
        """Build DepositAndSwap instruction."""
        return build_deposit_and_swap_instruction(
            operator=params.operator,
            market=params.market,
            base_mint=params.base_mint,
            quote_mint=params.quote_mint,
            taker_order=params.taker_order,
            taker_is_full_fill=params.taker_is_full_fill,
            taker_is_deposit=params.taker_is_deposit,
            taker_deposit_mint=params.taker_deposit_mint,
            num_outcomes=params.num_outcomes,
            makers=params.makers,
            program_id=self._client.program_id,
        )

    # ── On-chain transaction builders ────────────────────────────────────

    def initialize_tx(self, authority: Pubkey) -> Transaction:
        """Build Initialize transaction."""
        ix = self.initialize_ix(authority)
        return Transaction.new_with_payer([ix], authority)

    async def create_market_tx(
        self,
        authority: Pubkey,
        num_outcomes: int,
        oracle: Pubkey,
        question_id: bytes,
    ) -> Transaction:
        """Build CreateMarket transaction.

        Async because it fetches the next market ID from on-chain state.
        """
        ix = await self.create_market_ix(authority, num_outcomes, oracle, question_id)
        return Transaction.new_with_payer([ix], authority)

    def add_deposit_mint_tx(
        self,
        params: AddDepositMintParams,
        market: Pubkey,
        num_outcomes: int,
    ) -> Transaction:
        """Build AddDepositMint transaction."""
        ix = self.add_deposit_mint_ix(params, market, num_outcomes)
        return Transaction.new_with_payer([ix], params.authority)

    def activate_market_tx(self, params: ActivateMarketParams) -> Transaction:
        """Build ActivateMarket transaction."""
        ix = self.activate_market_ix(params)
        return Transaction.new_with_payer([ix], params.authority)

    def settle_market_tx(self, params: SettleMarketParams) -> Transaction:
        """Build SettleMarket transaction."""
        ix = self.settle_market_ix(params)
        return Transaction.new_with_payer([ix], params.oracle)

    def set_paused_tx(self, authority: Pubkey, paused: bool) -> Transaction:
        """Build SetPaused transaction."""
        ix = self.set_paused_ix(authority, paused)
        return Transaction.new_with_payer([ix], authority)

    def set_operator_tx(self, authority: Pubkey, new_operator: Pubkey) -> Transaction:
        """Build SetOperator transaction."""
        ix = self.set_operator_ix(authority, new_operator)
        return Transaction.new_with_payer([ix], authority)

    def set_authority_tx(self, params: SetAuthorityParams) -> Transaction:
        """Build SetAuthority transaction."""
        ix = self.set_authority_ix(params)
        return Transaction.new_with_payer([ix], params.current_authority)

    def whitelist_deposit_token_tx(self, params: WhitelistDepositTokenParams) -> Transaction:
        """Build WhitelistDepositToken transaction."""
        ix = self.whitelist_deposit_token_ix(params)
        return Transaction.new_with_payer([ix], params.authority)

    def create_orderbook_tx(self, params: CreateOrderbookParams) -> Transaction:
        """Build CreateOrderbook transaction."""
        ix = self.create_orderbook_ix(params)
        return Transaction.new_with_payer([ix], params.authority)

    def match_orders_multi_tx(self, params: MatchOrdersMultiParams) -> Transaction:
        """Build MatchOrdersMulti transaction."""
        ix = self.match_orders_multi_ix(params)
        return Transaction.new_with_payer([ix], params.operator)

    def deposit_and_swap_tx(self, params: DepositAndSwapParams) -> Transaction:
        """Build DepositAndSwap transaction."""
        ix = self.deposit_and_swap_ix(params)
        return Transaction.new_with_payer([ix], params.operator)
