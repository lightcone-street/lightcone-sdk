"""RPC sub-client — exchange-level on-chain fetchers, global deposit helpers, and blockhash access.

Mirrors rust/src/rpc.rs.
"""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING, Callable, Optional, TypeVar

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
from .env import PROGRAM_ID
from .program.errors import AccountNotFoundError
from .program.pda import (
    get_exchange_pda,
    get_global_deposit_pda,
    get_user_global_deposit_pda,
)
from .program.types import Exchange, GlobalDepositToken
from .rpc_failover import (
    ActiveRpc,
    is_infrastructure_error,
    FAST_RETRY_DELAY_SECS,
)

if TYPE_CHECKING:
    from solana.rpc.async_api import AsyncClient

    from .client import LightconeClient

T = TypeVar("T")


def require_connection(client: "LightconeClient") -> "AsyncClient":
    """Resolve the currently-active Solana RPC client, or raise if not configured."""
    conn = client.connection
    if conn is None:
        raise SdkError(
            "Solana RPC not configured — use .rpc_url() on the builder"
        )
    return conn


def _resolve_connection_for(
    client: "LightconeClient", target: ActiveRpc
) -> Optional["AsyncClient"]:
    """Resolve the connection for a specific endpoint."""
    if target == ActiveRpc.PRIMARY:
        return client._primary_connection
    return client._backup_connection


async def _connection_with_failover(
    client: "LightconeClient",
    operation: Callable[["AsyncClient"], object],
) -> object:
    """Execute an RPC operation with fast retry + failover.

    Same flow as Rust's ``solana_rpc_with_failover``.
    """
    state = client.rpc_failover_state
    state.maybe_recover_to_primary()
    original_active = state.active

    active_conn = require_connection(client)

    # First attempt.
    try:
        return await operation(active_conn)  # type: ignore[misc]
    except Exception as first_error:
        if not is_infrastructure_error(first_error):
            raise

    # Fast retry on same connection.
    await asyncio.sleep(FAST_RETRY_DELAY_SECS)
    try:
        return await operation(active_conn)  # type: ignore[misc]
    except Exception as retry_error:
        if not is_infrastructure_error(retry_error):
            raise

    # Flip and try the other connection.
    other_target = (
        ActiveRpc.BACKUP
        if original_active == ActiveRpc.PRIMARY
        else ActiveRpc.PRIMARY
    )
    other_conn = _resolve_connection_for(client, other_target)
    if other_conn is not None:
        try:
            result = await operation(other_conn)  # type: ignore[misc]
            if other_target == ActiveRpc.PRIMARY:
                state.flip_to_primary()
            else:
                state.flip_to_backup()
            return result
        except Exception:
            raise

    raise retry_error  # type: ignore[name-defined]  # noqa: F821


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
        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_latest_blockhash(),
        )
        return response.value.blockhash  # type: ignore[union-attr]

    async def get_exchange(self) -> Exchange:
        """Fetch the Exchange account."""
        pda = self.get_exchange_pda()
        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_account_info(pda),
        )
        if response.value is None:  # type: ignore[union-attr]
            raise AccountNotFoundError(str(pda))
        return deserialize_exchange(response.value.data)  # type: ignore[union-attr]

    async def get_global_deposit_token(self, mint: Pubkey) -> GlobalDepositToken:
        """Fetch a GlobalDepositToken account by mint."""
        pda = self.get_global_deposit_token_pda(mint)
        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_account_info(pda),
        )
        if response.value is None:  # type: ignore[union-attr]
            raise AccountNotFoundError(str(pda))
        return deserialize_global_deposit_token(response.value.data)  # type: ignore[union-attr]

    # ── Convenience ──────────────────────────────────────────────────────

    async def build_transaction(self, instructions: list[Instruction]) -> Transaction:
        """Build an unsigned transaction with a fresh blockhash."""
        blockhash = await self.get_latest_blockhash()
        message = Message.new_with_blockhash(instructions, None, blockhash)
        return Transaction.new_unsigned(message)


__all__ = ["Rpc", "require_connection"]
