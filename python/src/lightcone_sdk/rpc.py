"""RPC sub-client — exchange-level on-chain fetchers, global deposit helpers, and blockhash access.

Mirrors rust/src/rpc.rs.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

from solders.hash import Hash
from solders.instruction import Instruction
from solders.message import Message
from solders.pubkey import Pubkey
from solders.transaction import Transaction

from .error import SdkError
from .program.accounts import (
    deserialize_exchange,
    deserialize_global_deposit_token,
)
from .program.constants import PROGRAM_ID
from .program.errors import AccountNotFoundError
from .program.pda import (
    get_exchange_pda,
    get_global_deposit_pda,
    get_user_global_deposit_pda,
)
from .program.types import Exchange, GlobalDepositToken

if TYPE_CHECKING:
    from solana.rpc.async_api import AsyncClient

    from .client import LightconeClient


def require_connection(client: "LightconeClient") -> "AsyncClient":
    """Resolve the Solana RPC client, or raise if not configured."""
    conn = client.connection
    if conn is None:
        raise SdkError(
            "Solana RPC not configured — use .rpc_url() on the builder"
        )
    return conn


class Rpc:
    """RPC sub-client — PDA helpers, account fetchers, and blockhash access."""

    def __init__(self, client: "LightconeClient") -> None:
        self._client = client

    # ── PDA helpers (sync, always available) ─────────────────────────────

    def get_exchange_pda(self) -> Pubkey:
        """Get the Exchange PDA."""
        pda, _ = get_exchange_pda(self._client.program_id)
        return pda

    def get_global_deposit_token_pda(self, mint: Pubkey) -> Pubkey:
        """Get a GlobalDepositToken PDA."""
        pda, _ = get_global_deposit_pda(mint, self._client.program_id)
        return pda

    def get_user_global_deposit_pda(self, user: Pubkey, mint: Pubkey) -> Pubkey:
        """Get a User Global Deposit PDA."""
        pda, _ = get_user_global_deposit_pda(user, mint, self._client.program_id)
        return pda

    # ── On-chain account fetchers (async, require connection) ────────────

    async def get_latest_blockhash(self) -> Hash:
        """Get the latest blockhash for transaction building."""
        conn = require_connection(self._client)
        response = await conn.get_latest_blockhash()
        return response.value.blockhash

    async def get_exchange(self) -> Exchange:
        """Fetch the Exchange account."""
        conn = require_connection(self._client)
        pda = self.get_exchange_pda()
        response = await conn.get_account_info(pda)
        if response.value is None:
            raise AccountNotFoundError(str(pda))
        return deserialize_exchange(response.value.data)

    async def get_global_deposit_token(self, mint: Pubkey) -> GlobalDepositToken:
        """Fetch a GlobalDepositToken account by mint."""
        conn = require_connection(self._client)
        pda = self.get_global_deposit_token_pda(mint)
        response = await conn.get_account_info(pda)
        if response.value is None:
            raise AccountNotFoundError(str(pda))
        return deserialize_global_deposit_token(response.value.data)

    # ── Convenience ──────────────────────────────────────────────────────

    async def build_transaction(self, instructions: list[Instruction]) -> Transaction:
        """Build an unsigned transaction with a fresh blockhash."""
        blockhash = await self.get_latest_blockhash()
        message = Message.new_with_blockhash(instructions, None, blockhash)
        return Transaction.new_unsigned(message)


__all__ = ["Rpc", "require_connection"]
