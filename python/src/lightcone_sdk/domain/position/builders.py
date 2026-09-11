"""Fluent builders for deposit, withdraw, merge, and position operations.

Created via factory methods on ``client.positions()``.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from solders.instruction import Instruction
from solders.pubkey import Pubkey

from ...error import MissingMarketContext, SdkError
from ...program.instructions import (
    build_deposit_instruction,
    build_deposit_to_global_instruction,
    build_global_to_market_deposit_instruction,
    build_init_position_tokens_instruction,
    build_merge_instruction,
    build_redeem_winnings_instruction,
    build_withdraw_conditional_from_position_instruction,
    build_withdraw_from_global_instruction,
)
from ...program.transaction import V1Transaction, V1TransactionContext
from ...program.utils import validate_outcome_count, validate_outcome_index
from ...shared.types import DepositSource

if TYPE_CHECKING:
    from ...client import LightconeClient


# ─── DepositBuilder ─────────────────────────────────────────────────────────


class DepositBuilder:
    """Fluent builder for unified deposit operations.

    Dispatches based on deposit source:
    - **Global**: ``deposit_to_global`` — wallet -> global pool
    - **Market**: ``mint_complete_set`` — wallet -> market, mints conditional tokens

    Created via ``client.positions().deposit()``.
    """

    def __init__(self, client: LightconeClient, deposit_source: DepositSource):
        self._client = client
        self._user: Pubkey | None = None
        self._mint: Pubkey | None = None
        self._amount: int | None = None
        self._market: object = None
        self._deposit_source: DepositSource | None = deposit_source

    def user(self, user: Pubkey) -> DepositBuilder:
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> DepositBuilder:
        self._mint = mint
        return self

    def amount(self, amount: int) -> DepositBuilder:
        self._amount = amount
        return self

    def market(self, market: object) -> DepositBuilder:
        """Set the market reference (required when deposit source is ``Market``)."""
        self._market = market
        return self

    def deposit_source(self, source: DepositSource) -> DepositBuilder:
        self._deposit_source = source
        return self

    def with_market_deposit_source(self, market: object) -> DepositBuilder:
        """Set deposit source to ``Market`` and provide the required market reference."""
        self._deposit_source = DepositSource.MARKET
        self._market = market
        return self

    def with_global_deposit_source(self) -> DepositBuilder:
        self._deposit_source = DepositSource.GLOBAL
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        mint = self._mint
        if mint is None:
            raise SdkError("mint is required")
        amount = self._amount
        if amount is None:
            raise SdkError("amount is required")

        source = self._client.resolve_deposit_source(self._deposit_source)
        program_id = self._client.program_id

        if source == DepositSource.GLOBAL:
            return build_deposit_to_global_instruction(
                user=user,
                mint=mint,
                amount=amount,
                program_id=program_id,
            )
        else:  # Market -> deposit (mint complete set)
            market = self._market
            if market is None:
                raise MissingMarketContext(
                    "market is required for Market deposit source"
                )
            market_pubkey = Pubkey.from_string(market.pubkey)  # type: ignore[attr-defined]
            num_outcomes = market.num_outcomes  # type: ignore[attr-defined]
            return build_deposit_instruction(
                user=user,
                market=market_pubkey,
                deposit_mint=mint,
                amount=amount,
                num_outcomes=num_outcomes,
                program_id=program_id,
            )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        user = self._user
        if user is None:
            raise SdkError("user is required")
        ix = self.build_ix()
        return V1Transaction.compile([ix], user, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the deposit transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


# ─── WithdrawBuilder ────────────────────────────────────────────────────────


class MergeBuilder:
    """Fluent builder for merge operations.

    Burns a complete set of conditional tokens (one of each outcome) from a market
    position and releases the underlying collateral back to the user's wallet.

    Created via ``client.positions().merge()``.

    Example::

        ix = (client.positions().merge()
            .user(keypair.pubkey())
            .market(market)
            .mint(deposit_mint)
            .amount(1_000_000)
            .build_ix())
    """

    def __init__(self, client: LightconeClient):
        self._client = client
        self._user: Pubkey | None = None
        self._mint: Pubkey | None = None
        self._amount: int | None = None
        self._market: object = None

    def user(self, user: Pubkey) -> MergeBuilder:
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> MergeBuilder:
        self._mint = mint
        return self

    def amount(self, amount: int) -> MergeBuilder:
        self._amount = amount
        return self

    def market(self, market: object) -> MergeBuilder:
        """Set the market reference (required)."""
        self._market = market
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        mint = self._mint
        if mint is None:
            raise SdkError("mint is required")
        amount = self._amount
        if amount is None:
            raise SdkError("amount is required")
        market = self._market
        if market is None:
            raise MissingMarketContext("market is required for merge")
        market_pubkey = Pubkey.from_string(market.pubkey)  # type: ignore[attr-defined]
        num_outcomes = market.num_outcomes  # type: ignore[attr-defined]
        return build_merge_instruction(
            user=user,
            market=market_pubkey,
            deposit_mint=mint,
            amount=amount,
            num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        user = self._user
        if user is None:
            raise SdkError("user is required")
        ix = self.build_ix()
        return V1Transaction.compile([ix], user, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the merge transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


class WithdrawBuilder:
    """Fluent builder for unified withdraw operations.

    Dispatches based on deposit source:
    - **Global**: ``withdraw_from_global`` — global pool -> wallet
    - **Market**: ``withdraw_conditional_from_position`` — conditional-token ATA -> user's wallet

    Created via ``client.positions().withdraw()``.
    """

    def __init__(self, client: LightconeClient, deposit_source: DepositSource):
        self._client = client
        self._user: Pubkey | None = None
        self._mint: Pubkey | None = None
        self._amount: int | None = None
        self._market: object = None
        self._deposit_source: DepositSource | None = deposit_source
        self._outcome_index: int | None = None

    def user(self, user: Pubkey) -> WithdrawBuilder:
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> WithdrawBuilder:
        """Set the token mint.

        In ``Global`` mode this is the deposit token mint to withdraw. In
        ``Market`` mode this is the market's registered deposit mint; the
        conditional mint is derived from this mint plus ``outcome_index``.
        """
        self._mint = mint
        return self

    def deposit_mint(self, deposit_mint: Pubkey) -> WithdrawBuilder:
        """Set the registered deposit mint for a market withdrawal."""
        return self.mint(deposit_mint)

    def amount(self, amount: int) -> WithdrawBuilder:
        self._amount = amount
        return self

    def market(self, market: object) -> WithdrawBuilder:
        """Set the market reference (required when deposit source is ``Market``)."""
        self._market = market
        return self

    def deposit_source(self, source: DepositSource) -> WithdrawBuilder:
        self._deposit_source = source
        return self

    def outcome_index(self, outcome_index: int) -> WithdrawBuilder:
        """Set the outcome index (required when deposit source is ``Market``)."""
        self._outcome_index = outcome_index
        return self

    def with_market_deposit_source(self, market: object) -> WithdrawBuilder:
        """Set deposit source to ``Market`` and provide the required market reference."""
        self._deposit_source = DepositSource.MARKET
        self._market = market
        return self

    def with_global_deposit_source(self) -> WithdrawBuilder:
        self._deposit_source = DepositSource.GLOBAL
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        mint = self._mint
        if mint is None:
            raise SdkError("mint is required")
        amount = self._amount
        if amount is None:
            raise SdkError("amount is required")

        source = self._client.resolve_deposit_source(self._deposit_source)
        program_id = self._client.program_id

        if source == DepositSource.GLOBAL:
            return build_withdraw_from_global_instruction(
                user=user,
                mint=mint,
                amount=amount,
                program_id=program_id,
            )
        else:  # Market -> withdraw_conditional_from_position
            market = self._market
            if market is None:
                raise MissingMarketContext("market is required for Market withdrawal")
            market_pubkey = Pubkey.from_string(market.pubkey)  # type: ignore[attr-defined]
            outcome_index = self._outcome_index
            if outcome_index is None:
                raise SdkError("outcome_index is required for Market withdrawal")
            validate_outcome_index(outcome_index, market.num_outcomes)  # type: ignore[attr-defined]
            return build_withdraw_conditional_from_position_instruction(
                user=user,
                market=market_pubkey,
                deposit_mint=mint,
                amount=amount,
                outcome_index=outcome_index,
                program_id=program_id,
            )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        user = self._user
        if user is None:
            raise SdkError("user is required")
        ix = self.build_ix()
        return V1Transaction.compile([ix], user, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the withdraw transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


# ─── RedeemWinningsBuilder ──────────────────────────────────────────────────


class RedeemWinningsBuilder:
    """Fluent builder for redeem winnings operations."""

    def __init__(self, client: LightconeClient):
        self._client = client
        self._user: Pubkey | None = None
        self._market: Pubkey | None = None
        self._mint: Pubkey | None = None
        self._amount: int | None = None
        self._outcome_index: int | None = None

    def user(self, user: Pubkey) -> RedeemWinningsBuilder:
        self._user = user
        return self

    def market(self, market: Pubkey) -> RedeemWinningsBuilder:
        self._market = market
        return self

    def mint(self, mint: Pubkey) -> RedeemWinningsBuilder:
        self._mint = mint
        return self

    def amount(self, amount: int) -> RedeemWinningsBuilder:
        self._amount = amount
        return self

    def outcome_index(self, outcome_index: int) -> RedeemWinningsBuilder:
        self._outcome_index = outcome_index
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        market = self._market
        if market is None:
            raise SdkError("market is required")
        mint = self._mint
        if mint is None:
            raise SdkError("mint is required")
        amount = self._amount
        if amount is None:
            raise SdkError("amount is required")
        outcome_index = self._outcome_index
        if outcome_index is None:
            raise SdkError("outcome_index is required")
        return build_redeem_winnings_instruction(
            user=user,
            market=market,
            deposit_mint=mint,
            outcome_index=outcome_index,
            amount=amount,
            program_id=self._client.program_id,
        )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        user = self._user
        if user is None:
            raise SdkError("user is required")
        return V1Transaction.compile([self.build_ix()], user, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the redeem winnings transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


# ─── WithdrawFromPositionBuilder ────────────────────────────────────────────


class WithdrawFromPositionBuilder:
    """Fluent builder for conditional-token withdraw-from-position operations."""

    def __init__(self, client: LightconeClient):
        self._client = client
        self._user: Pubkey | None = None
        self._market: Pubkey | None = None
        self._deposit_mint: Pubkey | None = None
        self._amount: int | None = None
        self._outcome_index: int | None = None
        self._num_outcomes: int | None = None

    def user(self, user: Pubkey) -> WithdrawFromPositionBuilder:
        self._user = user
        return self

    def market(self, market: Pubkey) -> WithdrawFromPositionBuilder:
        self._market = market
        return self

    def deposit_mint(self, deposit_mint: Pubkey) -> WithdrawFromPositionBuilder:
        self._deposit_mint = deposit_mint
        return self

    def mint(self, deposit_mint: Pubkey) -> WithdrawFromPositionBuilder:
        """Set the registered deposit mint for the market."""
        return self.deposit_mint(deposit_mint)

    def amount(self, amount: int) -> WithdrawFromPositionBuilder:
        self._amount = amount
        return self

    def outcome_index(self, outcome_index: int) -> WithdrawFromPositionBuilder:
        self._outcome_index = outcome_index
        return self

    def num_outcomes(self, num_outcomes: int) -> WithdrawFromPositionBuilder:
        """Set the market's authoritative outcome count."""
        self._num_outcomes = num_outcomes
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        market = self._market
        if market is None:
            raise SdkError("market is required")
        deposit_mint = self._deposit_mint
        if deposit_mint is None:
            raise SdkError("deposit_mint is required")
        amount = self._amount
        if amount is None:
            raise SdkError("amount is required")
        outcome_index = self._outcome_index
        if outcome_index is None:
            raise SdkError("outcome_index is required")
        num_outcomes = self._num_outcomes
        if num_outcomes is None:
            raise SdkError("num_outcomes is required")
        validate_outcome_count(num_outcomes)
        validate_outcome_index(outcome_index, num_outcomes)
        return build_withdraw_conditional_from_position_instruction(
            user=user,
            market=market,
            deposit_mint=deposit_mint,
            amount=amount,
            outcome_index=outcome_index,
            program_id=self._client.program_id,
        )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        user = self._user
        if user is None:
            raise SdkError("user is required")
        return V1Transaction.compile([self.build_ix()], user, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the withdraw-from-position transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


# ─── InitPositionTokensBuilder ──────────────────────────────────────────────


class InitPositionTokensBuilder:
    """Prepare position and conditional ATAs without a recent slot.

    Supply unique deposit mints in increasing GDT registration index order.
    The same instruction supports initial setup, retries, and additional groups.
    """

    def __init__(self, client: LightconeClient):
        self._client = client
        self._payer: Pubkey | None = None
        self._user: Pubkey | None = None
        self._market: Pubkey | None = None
        self._deposit_mints: list[Pubkey] | None = None
        self._num_outcomes: int | None = None

    def payer(self, payer: Pubkey) -> InitPositionTokensBuilder:
        self._payer = payer
        return self

    def user(self, user: Pubkey) -> InitPositionTokensBuilder:
        self._user = user
        return self

    def market(self, market: Pubkey) -> InitPositionTokensBuilder:
        self._market = market
        return self

    def deposit_mints(self, deposit_mints: list[Pubkey]) -> InitPositionTokensBuilder:
        self._deposit_mints = deposit_mints
        return self

    def num_outcomes(self, num_outcomes: int) -> InitPositionTokensBuilder:
        self._num_outcomes = num_outcomes
        return self

    def build_ix(self) -> Instruction:
        payer = self._payer
        if payer is None:
            raise SdkError("payer is required")
        user = self._user
        if user is None:
            raise SdkError("user is required")
        market = self._market
        if market is None:
            raise SdkError("market is required")
        deposit_mints = self._deposit_mints
        if deposit_mints is None:
            raise SdkError("deposit_mints is required")
        num_outcomes = self._num_outcomes
        if num_outcomes is None:
            raise SdkError("num_outcomes is required")
        return build_init_position_tokens_instruction(
            payer,
            user,
            market,
            deposit_mints,
            num_outcomes,
            self._client.program_id,
        )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        payer = self._payer
        if payer is None:
            raise SdkError("payer is required")
        return V1Transaction.compile([self.build_ix()], payer, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the init-position-tokens transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


# ─── DepositToGlobalBuilder ─────────────────────────────────────────────────


class DepositToGlobalBuilder:
    """Fluent builder for deposit-to-global operations."""

    def __init__(self, client: LightconeClient):
        self._client = client
        self._user: Pubkey | None = None
        self._mint: Pubkey | None = None
        self._amount: int | None = None

    def user(self, user: Pubkey) -> DepositToGlobalBuilder:
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> DepositToGlobalBuilder:
        self._mint = mint
        return self

    def amount(self, amount: int) -> DepositToGlobalBuilder:
        self._amount = amount
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        mint = self._mint
        if mint is None:
            raise SdkError("mint is required")
        amount = self._amount
        if amount is None:
            raise SdkError("amount is required")
        return build_deposit_to_global_instruction(
            user=user,
            mint=mint,
            amount=amount,
            program_id=self._client.program_id,
        )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        user = self._user
        if user is None:
            raise SdkError("user is required")
        return V1Transaction.compile([self.build_ix()], user, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the deposit-to-global transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


# ─── WithdrawFromGlobalBuilder ──────────────────────────────────────────────


class WithdrawFromGlobalBuilder:
    """Fluent builder for withdraw-from-global operations."""

    def __init__(self, client: LightconeClient):
        self._client = client
        self._user: Pubkey | None = None
        self._mint: Pubkey | None = None
        self._amount: int | None = None

    def user(self, user: Pubkey) -> WithdrawFromGlobalBuilder:
        self._user = user
        return self

    def mint(self, mint: Pubkey) -> WithdrawFromGlobalBuilder:
        self._mint = mint
        return self

    def amount(self, amount: int) -> WithdrawFromGlobalBuilder:
        self._amount = amount
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        mint = self._mint
        if mint is None:
            raise SdkError("mint is required")
        amount = self._amount
        if amount is None:
            raise SdkError("amount is required")
        return build_withdraw_from_global_instruction(
            user=user,
            mint=mint,
            amount=amount,
            program_id=self._client.program_id,
        )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        user = self._user
        if user is None:
            raise SdkError("user is required")
        return V1Transaction.compile([self.build_ix()], user, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the withdraw-from-global transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


# ─── GlobalToMarketDepositBuilder ───────────────────────────────────────────


class GlobalToMarketDepositBuilder:
    """Fluent builder for global-to-market deposit operations."""

    def __init__(self, client: LightconeClient):
        self._client = client
        self._user: Pubkey | None = None
        self._market: Pubkey | None = None
        self._mint: Pubkey | None = None
        self._amount: int | None = None
        self._num_outcomes: int | None = None

    def user(self, user: Pubkey) -> GlobalToMarketDepositBuilder:
        self._user = user
        return self

    def market(self, market: Pubkey) -> GlobalToMarketDepositBuilder:
        self._market = market
        return self

    def mint(self, mint: Pubkey) -> GlobalToMarketDepositBuilder:
        self._mint = mint
        return self

    def amount(self, amount: int) -> GlobalToMarketDepositBuilder:
        self._amount = amount
        return self

    def num_outcomes(self, num_outcomes: int) -> GlobalToMarketDepositBuilder:
        self._num_outcomes = num_outcomes
        return self

    def build_ix(self) -> Instruction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        market = self._market
        if market is None:
            raise SdkError("market is required")
        mint = self._mint
        if mint is None:
            raise SdkError("mint is required")
        amount = self._amount
        if amount is None:
            raise SdkError("amount is required")
        num_outcomes = self._num_outcomes
        if num_outcomes is None:
            raise SdkError("num_outcomes is required")
        return build_global_to_market_deposit_instruction(
            user=user,
            market=market,
            deposit_mint=mint,
            amount=amount,
            num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def build_tx(self, context: V1TransactionContext) -> V1Transaction:
        """Compile a v1 message with the caller's blockhash expiry and budgets."""
        user = self._user
        if user is None:
            raise SdkError("user is required")
        return V1Transaction.compile([self.build_ix()], user, context)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the global-to-market deposit transaction."""
        tx = self.build_tx(await self._client.transaction_context())
        return await self._client.sign_and_submit_tx(tx)


__all__ = [
    "DepositBuilder",
    "MergeBuilder",
    "WithdrawBuilder",
    "RedeemWinningsBuilder",
    "WithdrawFromPositionBuilder",
    "InitPositionTokensBuilder",
    "DepositToGlobalBuilder",
    "WithdrawFromGlobalBuilder",
    "GlobalToMarketDepositBuilder",
]
