"""Fluent builders for deposit, withdraw, and position operations.

Created via factory methods on ``client.positions()``.
"""

from __future__ import annotations

from typing import Optional, List, TYPE_CHECKING

from solders.instruction import Instruction
from solders.message import Message
from solders.pubkey import Pubkey
from solders.transaction import Transaction

from ...error import SdkError, MissingMarketContext
from ...program.errors import MissingFieldError
from ...shared.types import DepositSource
from ...program.types import (
    DepositToGlobalParams,
    ExtendPositionTokensParams,
    GlobalToMarketDepositParams,
    InitPositionTokensParams,
    MergeCompleteSetParams,
    MintCompleteSetParams,
    RedeemWinningsParams,
    WithdrawFromGlobalParams,
    WithdrawFromPositionParams,
)
from ...program.instructions import (
    build_deposit_to_global_instruction,
    build_extend_position_tokens_instruction,
    build_global_to_market_deposit_instruction,
    build_init_position_tokens_instruction,
    build_merge_complete_set_instruction,
    build_mint_complete_set_instruction,
    build_redeem_winnings_instruction,
    build_withdraw_from_global_instruction,
    build_withdraw_from_position_instruction,
)

if TYPE_CHECKING:
    from ...client import LightconeClient
    from ...domain.market import Market


# ─── DepositBuilder ─────────────────────────────────────────────────────────


class DepositBuilder:
    """Fluent builder for unified deposit operations.

    Dispatches based on deposit source:
    - **Global**: ``deposit_to_global`` — wallet -> global pool
    - **Market**: ``mint_complete_set`` — wallet -> market, mints conditional tokens

    Created via ``client.positions().deposit()``.
    """

    def __init__(self, client: "LightconeClient", deposit_source: DepositSource):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None
        self._market: Optional["Market"] = None
        self._deposit_source: Optional[DepositSource] = deposit_source

    def user(self, user: Pubkey) -> "DepositBuilder":
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> "DepositBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "DepositBuilder":
        self._amount = amount
        return self

    def market(self, market: "Market") -> "DepositBuilder":
        """Set the market reference (required when deposit source is ``Market``)."""
        self._market = market
        return self

    def deposit_source(self, source: DepositSource) -> "DepositBuilder":
        self._deposit_source = source
        return self

    def with_market_deposit_source(self, market: "Market") -> "DepositBuilder":
        """Set deposit source to ``Market`` and provide the required market reference."""
        self._deposit_source = DepositSource.MARKET
        self._market = market
        return self

    def with_global_deposit_source(self) -> "DepositBuilder":
        self._deposit_source = DepositSource.GLOBAL
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        mint = self._mint
        if mint is None:
            raise MissingFieldError("mint")
        amount = self._amount
        if amount is None:
            raise MissingFieldError("amount")

        source = self._client.resolve_deposit_source(self._deposit_source)
        program_id = self._client.program_id

        if source == DepositSource.GLOBAL:
            return build_deposit_to_global_instruction(
                user=user, mint=mint, amount=amount, program_id=program_id,
            )
        else:  # Market -> mint_complete_set
            market = self._market
            if market is None:
                raise MissingMarketContext("market is required for Market deposit source")
            market_pubkey = Pubkey.from_string(market.pubkey)
            num_outcomes = len(market.outcomes)
            return build_mint_complete_set_instruction(
                user=user, market=market_pubkey, deposit_mint=mint,
                amount=amount, num_outcomes=num_outcomes, program_id=program_id,
            )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        ix = self.build_ix()
        return Transaction.new_unsigned(Message.new_with_payer([ix], user))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the deposit transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


# ─── WithdrawBuilder ────────────────────────────────────────────────────────


class WithdrawBuilder:
    """Fluent builder for unified withdraw operations.

    Dispatches based on deposit source:
    - **Global**: ``withdraw_from_global`` — global pool -> wallet
    - **Market**: ``merge_complete_set`` — burns conditional tokens -> wallet collateral

    Created via ``client.positions().withdraw()``.
    """

    def __init__(self, client: "LightconeClient", deposit_source: DepositSource):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None
        self._market: Optional["Market"] = None
        self._deposit_source: Optional[DepositSource] = deposit_source

    def user(self, user: Pubkey) -> "WithdrawBuilder":
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> "WithdrawBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "WithdrawBuilder":
        self._amount = amount
        return self

    def market(self, market: "Market") -> "WithdrawBuilder":
        """Set the market reference (required when deposit source is ``Market``)."""
        self._market = market
        return self

    def deposit_source(self, source: DepositSource) -> "WithdrawBuilder":
        self._deposit_source = source
        return self

    def with_market_deposit_source(self, market: "Market") -> "WithdrawBuilder":
        """Set deposit source to ``Market`` and provide the required market reference."""
        self._deposit_source = DepositSource.MARKET
        self._market = market
        return self

    def with_global_deposit_source(self) -> "WithdrawBuilder":
        self._deposit_source = DepositSource.GLOBAL
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        mint = self._mint
        if mint is None:
            raise MissingFieldError("mint")
        amount = self._amount
        if amount is None:
            raise MissingFieldError("amount")

        source = self._client.resolve_deposit_source(self._deposit_source)
        program_id = self._client.program_id

        if source == DepositSource.GLOBAL:
            return build_withdraw_from_global_instruction(
                user=user, mint=mint, amount=amount, program_id=program_id,
            )
        else:  # Market -> merge_complete_set
            market = self._market
            if market is None:
                raise MissingMarketContext("market is required for Market withdrawal")
            market_pubkey = Pubkey.from_string(market.pubkey)
            num_outcomes = len(market.outcomes)
            return build_merge_complete_set_instruction(
                user=user, market=market_pubkey, deposit_mint=mint,
                amount=amount, num_outcomes=num_outcomes, program_id=program_id,
            )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        ix = self.build_ix()
        return Transaction.new_unsigned(Message.new_with_payer([ix], user))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the withdraw transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


# ─── RedeemWinningsBuilder ──────────────────────────────────────────────────


class RedeemWinningsBuilder:
    """Fluent builder for redeem winnings operations."""

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None
        self._winning_outcome: Optional[int] = None

    def user(self, user: Pubkey) -> "RedeemWinningsBuilder":
        self._user = user
        return self

    def market(self, market: Pubkey) -> "RedeemWinningsBuilder":
        self._market = market
        return self

    def mint(self, mint: Pubkey) -> "RedeemWinningsBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "RedeemWinningsBuilder":
        self._amount = amount
        return self

    def winning_outcome(self, winning_outcome: int) -> "RedeemWinningsBuilder":
        self._winning_outcome = winning_outcome
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        market = self._market
        if market is None:
            raise MissingFieldError("market")
        mint = self._mint
        if mint is None:
            raise MissingFieldError("mint")
        amount = self._amount
        if amount is None:
            raise MissingFieldError("amount")
        winning_outcome = self._winning_outcome
        if winning_outcome is None:
            raise MissingFieldError("winning_outcome")
        return build_redeem_winnings_instruction(
            user=user, market=market, deposit_mint=mint,
            winning_outcome=winning_outcome, amount=amount,
            program_id=self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        return Transaction.new_unsigned(Message.new_with_payer([self.build_ix()], user))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the redeem winnings transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


# ─── WithdrawFromPositionBuilder ────────────────────────────────────────────


class WithdrawFromPositionBuilder:
    """Fluent builder for withdraw-from-position operations."""

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None
        self._outcome_index: Optional[int] = None
        self._is_token_2022: bool = False

    def user(self, user: Pubkey) -> "WithdrawFromPositionBuilder":
        self._user = user
        return self

    def market(self, market: Pubkey) -> "WithdrawFromPositionBuilder":
        self._market = market
        return self

    def mint(self, mint: Pubkey) -> "WithdrawFromPositionBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "WithdrawFromPositionBuilder":
        self._amount = amount
        return self

    def outcome_index(self, outcome_index: int) -> "WithdrawFromPositionBuilder":
        self._outcome_index = outcome_index
        return self

    def token_2022(self, is_token_2022: bool) -> "WithdrawFromPositionBuilder":
        self._is_token_2022 = is_token_2022
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        market = self._market
        if market is None:
            raise MissingFieldError("market")
        mint = self._mint
        if mint is None:
            raise MissingFieldError("mint")
        amount = self._amount
        if amount is None:
            raise MissingFieldError("amount")
        outcome_index = self._outcome_index
        if outcome_index is None:
            raise MissingFieldError("outcome_index")
        return build_withdraw_from_position_instruction(
            user=user, market=market, mint=mint, amount=amount,
            outcome_index=outcome_index, is_token_2022=self._is_token_2022,
            program_id=self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        return Transaction.new_unsigned(Message.new_with_payer([self.build_ix()], user))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the withdraw-from-position transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


# ─── InitPositionTokensBuilder ──────────────────────────────────────────────


class InitPositionTokensBuilder:
    """Fluent builder for init-position-tokens operations."""

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._payer: Optional[Pubkey] = None
        self._user: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._deposit_mints: Optional[List[Pubkey]] = None
        self._recent_slot: Optional[int] = None
        self._num_outcomes: Optional[int] = None

    def payer(self, payer: Pubkey) -> "InitPositionTokensBuilder":
        self._payer = payer
        return self

    def user(self, user: Pubkey) -> "InitPositionTokensBuilder":
        self._user = user
        return self

    def market(self, market: Pubkey) -> "InitPositionTokensBuilder":
        self._market = market
        return self

    def deposit_mints(self, deposit_mints: List[Pubkey]) -> "InitPositionTokensBuilder":
        self._deposit_mints = deposit_mints
        return self

    def recent_slot(self, recent_slot: int) -> "InitPositionTokensBuilder":
        self._recent_slot = recent_slot
        return self

    def num_outcomes(self, num_outcomes: int) -> "InitPositionTokensBuilder":
        self._num_outcomes = num_outcomes
        return self

    def build_ix(self) -> Instruction:
        payer = self._payer
        if payer is None:
            raise MissingFieldError("payer")
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        market = self._market
        if market is None:
            raise MissingFieldError("market")
        deposit_mints = self._deposit_mints
        if deposit_mints is None:
            raise MissingFieldError("deposit_mints")
        recent_slot = self._recent_slot
        if recent_slot is None:
            raise MissingFieldError("recent_slot")
        num_outcomes = self._num_outcomes
        if num_outcomes is None:
            raise MissingFieldError("num_outcomes")
        return build_init_position_tokens_instruction(
            InitPositionTokensParams(
                payer=payer, user=user, market=market,
                deposit_mints=deposit_mints, recent_slot=recent_slot,
            ),
            num_outcomes, self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        payer = self._payer
        if payer is None:
            raise MissingFieldError("payer")
        return Transaction.new_unsigned(Message.new_with_payer([self.build_ix()], payer))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the init-position-tokens transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


# ─── ExtendPositionTokensBuilder ────────────────────────────────────────────


class ExtendPositionTokensBuilder:
    """Fluent builder for extend-position-tokens operations."""

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._payer: Optional[Pubkey] = None
        self._user: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._lookup_table: Optional[Pubkey] = None
        self._deposit_mints: Optional[List[Pubkey]] = None
        self._num_outcomes: Optional[int] = None

    def payer(self, payer: Pubkey) -> "ExtendPositionTokensBuilder":
        self._payer = payer
        return self

    def user(self, user: Pubkey) -> "ExtendPositionTokensBuilder":
        self._user = user
        return self

    def market(self, market: Pubkey) -> "ExtendPositionTokensBuilder":
        self._market = market
        return self

    def lookup_table(self, lookup_table: Pubkey) -> "ExtendPositionTokensBuilder":
        self._lookup_table = lookup_table
        return self

    def deposit_mints(self, deposit_mints: List[Pubkey]) -> "ExtendPositionTokensBuilder":
        self._deposit_mints = deposit_mints
        return self

    def num_outcomes(self, num_outcomes: int) -> "ExtendPositionTokensBuilder":
        self._num_outcomes = num_outcomes
        return self

    def build_ix(self) -> Instruction:
        payer = self._payer
        if payer is None:
            raise MissingFieldError("payer")
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        market = self._market
        if market is None:
            raise MissingFieldError("market")
        lookup_table = self._lookup_table
        if lookup_table is None:
            raise MissingFieldError("lookup_table")
        deposit_mints = self._deposit_mints
        if deposit_mints is None:
            raise MissingFieldError("deposit_mints")
        num_outcomes = self._num_outcomes
        if num_outcomes is None:
            raise MissingFieldError("num_outcomes")
        return build_extend_position_tokens_instruction(
            ExtendPositionTokensParams(
                payer=payer, user=user, market=market,
                lookup_table=lookup_table, deposit_mints=deposit_mints,
            ),
            num_outcomes, self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        payer = self._payer
        if payer is None:
            raise MissingFieldError("payer")
        return Transaction.new_unsigned(Message.new_with_payer([self.build_ix()], payer))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the extend-position-tokens transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


# ─── DepositToGlobalBuilder ─────────────────────────────────────────────────


class DepositToGlobalBuilder:
    """Fluent builder for deposit-to-global operations."""

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None

    def user(self, user: Pubkey) -> "DepositToGlobalBuilder":
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> "DepositToGlobalBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "DepositToGlobalBuilder":
        self._amount = amount
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        mint = self._mint
        if mint is None:
            raise MissingFieldError("mint")
        amount = self._amount
        if amount is None:
            raise MissingFieldError("amount")
        return build_deposit_to_global_instruction(
            user=user, mint=mint, amount=amount, program_id=self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        return Transaction.new_unsigned(Message.new_with_payer([self.build_ix()], user))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the deposit-to-global transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


# ─── WithdrawFromGlobalBuilder ──────────────────────────────────────────────


class WithdrawFromGlobalBuilder:
    """Fluent builder for withdraw-from-global operations."""

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None

    def user(self, user: Pubkey) -> "WithdrawFromGlobalBuilder":
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> "WithdrawFromGlobalBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "WithdrawFromGlobalBuilder":
        self._amount = amount
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        mint = self._mint
        if mint is None:
            raise MissingFieldError("mint")
        amount = self._amount
        if amount is None:
            raise MissingFieldError("amount")
        return build_withdraw_from_global_instruction(
            user=user, mint=mint, amount=amount, program_id=self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        return Transaction.new_unsigned(Message.new_with_payer([self.build_ix()], user))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the withdraw-from-global transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


# ─── GlobalToMarketDepositBuilder ───────────────────────────────────────────


class GlobalToMarketDepositBuilder:
    """Fluent builder for global-to-market deposit operations."""

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None
        self._num_outcomes: Optional[int] = None

    def user(self, user: Pubkey) -> "GlobalToMarketDepositBuilder":
        self._user = user
        return self

    def market(self, market: Pubkey) -> "GlobalToMarketDepositBuilder":
        self._market = market
        return self

    def mint(self, mint: Pubkey) -> "GlobalToMarketDepositBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "GlobalToMarketDepositBuilder":
        self._amount = amount
        return self

    def num_outcomes(self, num_outcomes: int) -> "GlobalToMarketDepositBuilder":
        self._num_outcomes = num_outcomes
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        market = self._market
        if market is None:
            raise MissingFieldError("market")
        mint = self._mint
        if mint is None:
            raise MissingFieldError("mint")
        amount = self._amount
        if amount is None:
            raise MissingFieldError("amount")
        num_outcomes = self._num_outcomes
        if num_outcomes is None:
            raise MissingFieldError("num_outcomes")
        return build_global_to_market_deposit_instruction(
            user=user, market=market, deposit_mint=mint,
            amount=amount, num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise MissingFieldError("user")
        return Transaction.new_unsigned(Message.new_with_payer([self.build_ix()], user))

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the global-to-market deposit transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


__all__ = [
    "DepositBuilder",
    "WithdrawBuilder",
    "RedeemWinningsBuilder",
    "WithdrawFromPositionBuilder",
    "InitPositionTokensBuilder",
    "ExtendPositionTokensBuilder",
    "DepositToGlobalBuilder",
    "WithdrawFromGlobalBuilder",
    "GlobalToMarketDepositBuilder",
]
