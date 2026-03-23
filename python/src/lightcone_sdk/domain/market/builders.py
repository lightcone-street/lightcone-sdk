"""Fluent builders for mint and merge complete-set operations.

Created via ``client.markets().mint_complete_set()`` and ``client.markets().merge_complete_set()``.
"""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING

from solders.instruction import Instruction
from solders.pubkey import Pubkey
from solders.transaction import Transaction

from ...error import SdkError
from ...program.types import MintCompleteSetParams, MergeCompleteSetParams
from ...program.instructions import (
    build_mint_complete_set_instruction,
    build_merge_complete_set_instruction,
)

if TYPE_CHECKING:
    from ...client import LightconeClient


class MintCompleteSetBuilder:
    """Fluent builder for mint-complete-set operations.

    Created via ``client.markets().mint_complete_set()``.

    Example::

        ix = (client.markets().mint_complete_set()
            .user(keypair.pubkey())
            .market(market_pubkey)
            .mint(deposit_mint)
            .amount(1_000_000)
            .num_outcomes(2)
            .build_ix())
    """

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None
        self._num_outcomes: Optional[int] = None

    def user(self, user: Pubkey) -> "MintCompleteSetBuilder":
        self._user = user
        return self

    def market(self, market: Pubkey) -> "MintCompleteSetBuilder":
        self._market = market
        return self

    def mint(self, mint: Pubkey) -> "MintCompleteSetBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "MintCompleteSetBuilder":
        self._amount = amount
        return self

    def num_outcomes(self, num_outcomes: int) -> "MintCompleteSetBuilder":
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
        return build_mint_complete_set_instruction(
            user=user, market=market, deposit_mint=mint,
            amount=amount, num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        ix = self.build_ix()
        return Transaction.new_with_payer([ix], user)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the mint-complete-set transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


class MergeCompleteSetBuilder:
    """Fluent builder for merge-complete-set operations.

    Created via ``client.markets().merge_complete_set()``.

    Example::

        ix = (client.markets().merge_complete_set()
            .user(keypair.pubkey())
            .market(market_pubkey)
            .mint(deposit_mint)
            .amount(1_000_000)
            .num_outcomes(2)
            .build_ix())
    """

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._user: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._mint: Optional[Pubkey] = None
        self._amount: Optional[int] = None
        self._num_outcomes: Optional[int] = None

    def user(self, user: Pubkey) -> "MergeCompleteSetBuilder":
        self._user = user
        return self

    def market(self, market: Pubkey) -> "MergeCompleteSetBuilder":
        self._market = market
        return self

    def mint(self, mint: Pubkey) -> "MergeCompleteSetBuilder":
        self._mint = mint
        return self

    def amount(self, amount: int) -> "MergeCompleteSetBuilder":
        self._amount = amount
        return self

    def num_outcomes(self, num_outcomes: int) -> "MergeCompleteSetBuilder":
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
        return build_merge_complete_set_instruction(
            user=user, market=market, deposit_mint=mint,
            amount=amount, num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def build_tx(self) -> Transaction:
        user = self._user
        if user is None:
            raise SdkError("user is required")
        ix = self.build_ix()
        return Transaction.new_with_payer([ix], user)

    async def sign_and_submit(self) -> str:
        """Build, sign, and submit the merge-complete-set transaction."""
        tx = self.build_tx()
        return await self._client.sign_and_submit_tx(tx)


__all__ = [
    "MintCompleteSetBuilder",
    "MergeCompleteSetBuilder",
]
