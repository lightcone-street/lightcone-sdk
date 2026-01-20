"""Main client for the Lightcone SDK."""

from typing import List, Optional

from solana.rpc.async_api import AsyncClient
from solders.hash import Hash
from solders.instruction import Instruction
from solders.keypair import Keypair
from solders.message import Message
from solders.pubkey import Pubkey
from solders.transaction import Transaction

from .accounts import (
    deserialize_exchange,
    deserialize_market,
    deserialize_order_status,
    deserialize_position,
    deserialize_user_nonce,
)
from .constants import PROGRAM_ID
from .ed25519 import (
    build_ed25519_batch_verify_instruction,
    build_ed25519_verify_instruction_for_order,
)
from .errors import AccountNotFoundError
from .instructions import (
    build_activate_market_instruction,
    build_add_deposit_mint_instruction,
    build_cancel_order_instruction,
    build_create_market_instruction_with_id,
    build_increment_nonce_instruction,
    build_initialize_instruction,
    build_match_orders_multi_instruction,
    build_merge_complete_set_instruction,
    build_mint_complete_set_instruction,
    build_redeem_winnings_instruction,
    build_set_operator_instruction,
    build_set_paused_instruction,
    build_settle_market_instruction,
    build_withdraw_from_position_instruction,
)
from .orders import (
    create_ask_order,
    create_bid_order,
    create_signed_ask_order,
    create_signed_bid_order,
    hash_order,
    sign_order,
)
from .pda import (
    get_all_conditional_mints,
    get_conditional_mint_pda,
    get_exchange_pda,
    get_market_pda,
    get_order_status_pda,
    get_position_pda,
    get_user_nonce_pda,
)
from .types import (
    ActivateMarketParams,
    AddDepositMintParams,
    AskOrderParams,
    BidOrderParams,
    Exchange,
    FullOrder,
    MakerFill,
    Market,
    MergeCompleteSetParams,
    MintCompleteSetParams,
    OrderStatus,
    Position,
    RedeemWinningsParams,
    SettleMarketParams,
    WithdrawFromPositionParams,
)
from .utils import derive_condition_id


class LightconePinocchioClient:
    """Async client for interacting with the Lightcone program."""

    def __init__(
        self,
        connection: AsyncClient,
        program_id: Pubkey = PROGRAM_ID,
    ):
        """Initialize the client.

        Args:
            connection: Solana RPC async client
            program_id: Lightcone program ID (defaults to mainnet)
        """
        self.connection = connection
        self.program_id = program_id

    # =========================================================================
    # Account Fetchers
    # =========================================================================

    async def get_exchange(self) -> Exchange:
        """Fetch and deserialize the exchange account."""
        exchange_pda, _ = get_exchange_pda(self.program_id)
        response = await self.connection.get_account_info(exchange_pda)

        if response.value is None:
            raise AccountNotFoundError(str(exchange_pda))

        return deserialize_exchange(response.value.data)

    async def get_market(self, market_id: int) -> Market:
        """Fetch and deserialize a market account."""
        market_pda, _ = get_market_pda(market_id, self.program_id)
        response = await self.connection.get_account_info(market_pda)

        if response.value is None:
            raise AccountNotFoundError(str(market_pda))

        return deserialize_market(response.value.data)

    async def get_market_by_address(self, market_address: Pubkey) -> Market:
        """Fetch and deserialize a market account by its address."""
        response = await self.connection.get_account_info(market_address)

        if response.value is None:
            raise AccountNotFoundError(str(market_address))

        return deserialize_market(response.value.data)

    async def get_position(
        self, owner: Pubkey, market: Pubkey
    ) -> Optional[Position]:
        """Fetch and deserialize a position account, or None if it doesn't exist."""
        position_pda, _ = get_position_pda(owner, market, self.program_id)
        response = await self.connection.get_account_info(position_pda)

        if response.value is None:
            return None

        return deserialize_position(response.value.data)

    async def get_order_status(self, order_hash: bytes) -> Optional[OrderStatus]:
        """Fetch and deserialize an order status account, or None if it doesn't exist."""
        order_status_pda, _ = get_order_status_pda(order_hash, self.program_id)
        response = await self.connection.get_account_info(order_status_pda)

        if response.value is None:
            return None

        return deserialize_order_status(response.value.data)

    async def get_user_nonce(self, user: Pubkey) -> int:
        """Get the current nonce for a user. Returns 0 if the account doesn't exist."""
        user_nonce_pda, _ = get_user_nonce_pda(user, self.program_id)
        response = await self.connection.get_account_info(user_nonce_pda)

        if response.value is None:
            return 0

        user_nonce = deserialize_user_nonce(response.value.data)
        return user_nonce.nonce

    async def get_next_nonce(self, user: Pubkey) -> int:
        """Get the next available nonce for a user (current nonce + 1)."""
        current = await self.get_user_nonce(user)
        return current + 1

    async def get_next_market_id(self) -> int:
        """Get the next market ID (current market_count)."""
        exchange = await self.get_exchange()
        return exchange.market_count

    # =========================================================================
    # Transaction Builders
    # =========================================================================

    async def initialize(self, authority: Pubkey) -> Transaction:
        """Build an initialize transaction."""
        ix = build_initialize_instruction(authority, self.program_id)
        return await self._build_transaction([ix])

    async def create_market(
        self,
        authority: Pubkey,
        num_outcomes: int,
        oracle: Pubkey,
        question_id: bytes,
    ) -> Transaction:
        """Build a create_market transaction."""
        market_id = await self.get_next_market_id()
        ix = build_create_market_instruction_with_id(
            authority=authority,
            market_id=market_id,
            num_outcomes=num_outcomes,
            oracle=oracle,
            question_id=question_id,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def add_deposit_mint(
        self, params: AddDepositMintParams, num_outcomes: int
    ) -> Transaction:
        """Build an add_deposit_mint transaction."""
        ix = build_add_deposit_mint_instruction(
            payer=params.payer,
            market=params.market,
            deposit_mint=params.deposit_mint,
            outcome_metadata=params.outcome_metadata,
            num_outcomes=num_outcomes,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def mint_complete_set(
        self, params: MintCompleteSetParams, num_outcomes: int
    ) -> Transaction:
        """Build a mint_complete_set transaction."""
        ix = build_mint_complete_set_instruction(
            user=params.user,
            market=params.market,
            deposit_mint=params.deposit_mint,
            amount=params.amount,
            num_outcomes=num_outcomes,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def merge_complete_set(
        self, params: MergeCompleteSetParams, num_outcomes: int
    ) -> Transaction:
        """Build a merge_complete_set transaction."""
        ix = build_merge_complete_set_instruction(
            user=params.user,
            market=params.market,
            deposit_mint=params.deposit_mint,
            amount=params.amount,
            num_outcomes=num_outcomes,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def cancel_order(self, maker: Pubkey, order: FullOrder) -> Transaction:
        """Build a cancel_order transaction."""
        ix = build_cancel_order_instruction(maker, order, self.program_id)
        return await self._build_transaction([ix])

    async def increment_nonce(self, user: Pubkey) -> Transaction:
        """Build an increment_nonce transaction."""
        ix = build_increment_nonce_instruction(user, self.program_id)
        return await self._build_transaction([ix])

    async def settle_market(self, params: SettleMarketParams) -> Transaction:
        """Build a settle_market transaction."""
        ix = build_settle_market_instruction(
            oracle=params.oracle,
            market=params.market,
            winning_outcome=params.winning_outcome,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def redeem_winnings(
        self, params: RedeemWinningsParams, winning_outcome: int
    ) -> Transaction:
        """Build a redeem_winnings transaction."""
        ix = build_redeem_winnings_instruction(
            user=params.user,
            market=params.market,
            deposit_mint=params.deposit_mint,
            winning_outcome=winning_outcome,
            amount=params.amount,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def set_paused(self, authority: Pubkey, paused: bool) -> Transaction:
        """Build a set_paused transaction."""
        ix = build_set_paused_instruction(authority, paused, self.program_id)
        return await self._build_transaction([ix])

    async def set_operator(
        self, authority: Pubkey, new_operator: Pubkey
    ) -> Transaction:
        """Build a set_operator transaction."""
        ix = build_set_operator_instruction(
            authority, new_operator, self.program_id
        )
        return await self._build_transaction([ix])

    async def withdraw_from_position(
        self, params: WithdrawFromPositionParams, is_token_2022: bool = True
    ) -> Transaction:
        """Build a withdraw_from_position transaction."""
        ix = build_withdraw_from_position_instruction(
            user=params.user,
            position=params.position,
            mint=params.mint,
            amount=params.amount,
            is_token_2022=is_token_2022,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def activate_market(self, params: ActivateMarketParams) -> Transaction:
        """Build an activate_market transaction."""
        ix = build_activate_market_instruction(
            authority=params.authority,
            market=params.market,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def match_orders_multi(
        self,
        operator: Pubkey,
        market: Pubkey,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        taker_order: FullOrder,
        maker_fills: List[MakerFill],
    ) -> Transaction:
        """Build a match_orders_multi transaction (without Ed25519 verify)."""
        ix = build_match_orders_multi_instruction(
            operator=operator,
            market=market,
            base_mint=base_mint,
            quote_mint=quote_mint,
            taker_order=taker_order,
            maker_fills=maker_fills,
            program_id=self.program_id,
        )
        return await self._build_transaction([ix])

    async def match_orders_multi_with_verify(
        self,
        operator: Pubkey,
        market: Pubkey,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        taker_order: FullOrder,
        maker_fills: List[MakerFill],
    ) -> Transaction:
        """Build a match_orders_multi transaction with Ed25519 signature verification.

        This includes an Ed25519 verify instruction before the match instruction.
        """
        # Build Ed25519 verify instructions for all orders
        all_orders = [taker_order] + [mf.order for mf in maker_fills]
        ed25519_ix = build_ed25519_batch_verify_instruction(all_orders)

        # Build match instruction
        match_ix = build_match_orders_multi_instruction(
            operator=operator,
            market=market,
            base_mint=base_mint,
            quote_mint=quote_mint,
            taker_order=taker_order,
            maker_fills=maker_fills,
            program_id=self.program_id,
        )

        return await self._build_transaction([ed25519_ix, match_ix])

    # =========================================================================
    # Order Helpers
    # =========================================================================

    def create_bid_order(self, params: BidOrderParams) -> FullOrder:
        """Create an unsigned bid order."""
        return create_bid_order(params)

    def create_ask_order(self, params: AskOrderParams) -> FullOrder:
        """Create an unsigned ask order."""
        return create_ask_order(params)

    def create_signed_bid_order(
        self, params: BidOrderParams, keypair: Keypair
    ) -> FullOrder:
        """Create and sign a bid order."""
        return create_signed_bid_order(params, keypair)

    def create_signed_ask_order(
        self, params: AskOrderParams, keypair: Keypair
    ) -> FullOrder:
        """Create and sign an ask order."""
        return create_signed_ask_order(params, keypair)

    def hash_order(self, order: FullOrder) -> bytes:
        """Compute the keccak256 hash of an order."""
        return hash_order(order)

    def sign_order(self, order: FullOrder, keypair: Keypair) -> bytes:
        """Sign an order with a keypair."""
        return sign_order(order, keypair)

    # =========================================================================
    # Utility Methods
    # =========================================================================

    def derive_condition_id(
        self, oracle: Pubkey, question_id: bytes, num_outcomes: int
    ) -> bytes:
        """Derive the condition ID for a market."""
        return derive_condition_id(oracle, question_id, num_outcomes)

    def get_conditional_mints(
        self, market: Pubkey, deposit_mint: Pubkey, num_outcomes: int
    ) -> List[Pubkey]:
        """Get all conditional mint addresses for a market."""
        return get_all_conditional_mints(
            market, deposit_mint, num_outcomes, self.program_id
        )

    def get_exchange_address(self) -> Pubkey:
        """Get the exchange PDA address."""
        pda, _ = get_exchange_pda(self.program_id)
        return pda

    def get_market_address(self, market_id: int) -> Pubkey:
        """Get a market PDA address."""
        pda, _ = get_market_pda(market_id, self.program_id)
        return pda

    def get_position_address(self, owner: Pubkey, market: Pubkey) -> Pubkey:
        """Get a position PDA address."""
        pda, _ = get_position_pda(owner, market, self.program_id)
        return pda

    def get_order_status_address(self, order_hash: bytes) -> Pubkey:
        """Get an order status PDA address."""
        pda, _ = get_order_status_pda(order_hash, self.program_id)
        return pda

    def get_user_nonce_address(self, user: Pubkey) -> Pubkey:
        """Get a user nonce PDA address."""
        pda, _ = get_user_nonce_pda(user, self.program_id)
        return pda

    # =========================================================================
    # Internal Helpers
    # =========================================================================

    async def _build_transaction(
        self, instructions: List[Instruction]
    ) -> Transaction:
        """Build a transaction with the given instructions."""
        # Get recent blockhash
        response = await self.connection.get_latest_blockhash()
        blockhash = response.value.blockhash

        # Build message and transaction
        message = Message.new_with_blockhash(
            instructions,
            None,  # No payer specified - caller must sign
            blockhash,
        )

        return Transaction.new_unsigned(message)

    async def _get_blockhash(self) -> Hash:
        """Get the latest blockhash."""
        response = await self.connection.get_latest_blockhash()
        return response.value.blockhash
