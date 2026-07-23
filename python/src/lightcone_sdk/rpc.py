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
from solders.signature import Signature
from solders.transaction import Transaction
from solders.transaction_status import TransactionConfirmationStatus

from .error import (
    ConfirmationTimeout,
    SdkError,
    TransactionExpired,
    TransactionFailed,
)
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
    from solders.transaction_status import TransactionStatus

    from .client import LightconeClient

T = TypeVar("T")

# ── Transaction confirmation ─────────────────────────────────────────────────

# Interval between polls while awaiting transaction confirmation.
_CONFIRMATION_POLL_INTERVAL_SECS = 0.8

# Hard cap on confirmation poll iterations (~90 s at the poll interval) — a
# backstop for when block-height expiry cannot be observed (e.g. a failed-over
# RPC node with a skewed view of the chain).
_MAX_CONFIRMATION_POLLS = 110

# Consecutive failed polls tolerated before the outcome is declared unknown.
_MAX_CONSECUTIVE_POLL_FAILURES = 3

# Consecutive over-bound block-height samples required before expiry may be
# declared — a single reading can come from a forward-skewed RPC node.
_EXPIRY_HEIGHT_SAMPLES = 2


def _is_transaction_confirmed(status: TransactionStatus) -> bool:
    """True once the cluster has voted the transaction to confirmed or beyond."""
    return status.confirmation_status in (
        TransactionConfirmationStatus.Confirmed,
        TransactionConfirmationStatus.Finalized,
    )


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
        blockhash, _last_valid_block_height = (
            await self.get_latest_blockhash_with_height()
        )
        return blockhash

    async def get_latest_blockhash_with_height(self) -> tuple[Hash, int]:
        """Get the latest blockhash and the last block height at which it is valid.

        Past the returned height, a transaction built on the blockhash can
        never land, which is what makes expiry detection in
        ``confirm_signature`` safe.
        """
        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_latest_blockhash(),
        )
        value = response.value  # type: ignore[union-attr]
        return value.blockhash, value.last_valid_block_height

    async def get_block_height(self) -> int:
        """Get the current block height at the connection's commitment (confirmed by default)."""
        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_block_height(),
        )
        return response.value  # type: ignore[union-attr]

    async def send_raw_transaction(self, tx_bytes: bytes) -> str:
        """Submit a signed transaction, returning its signature.

        Fire-and-forget: confirmation is skipped explicitly rather than left
        to solana-py's ``TxOpts`` defaults — waiting is ``confirm_signature``'s
        job, with its terminal error taxonomy.
        """
        from solana.rpc.types import TxOpts

        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.send_raw_transaction(
                tx_bytes,
                opts=TxOpts(
                    skip_confirmation=True,
                    preflight_commitment=conn.commitment,
                ),
            ),
        )
        return str(response.value)  # type: ignore[attr-defined]

    async def get_signature_statuses(
        self,
        signatures: list[str],
        search_transaction_history: bool = False,
    ) -> list[Optional[TransactionStatus]]:
        """Get the statuses of recently submitted transactions.

        Returns one entry per signature, in order; ``None`` means the cluster
        has not seen the signature (or, unless ``search_transaction_history``
        is set, it has aged out of the recent-status cache).
        """
        parsed = [Signature.from_string(signature) for signature in signatures]
        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_signature_statuses(
                parsed, search_transaction_history=search_transaction_history
            ),
        )
        return response.value  # type: ignore[union-attr]

    async def confirm_signature(
        self, signature: str, last_valid_block_height: Optional[int]
    ) -> None:
        """Wait until ``signature`` reaches confirmed commitment, or raise.

        Polls ``get_signature_statuses`` (with automatic RPC failover) until
        the cluster reports the transaction as confirmed or finalized.
        ``last_valid_block_height`` bounds the wait: pass the height returned
        alongside the transaction's blockhash, or ``None`` when the blockhash
        was set by the caller and its expiry is unknown — expiry is then never
        reported and only the poll cap ends the wait. Terminal outcomes:

        - ``TransactionFailed``: the transaction landed but errored on-chain;
          resubmitting the same transaction would fail again.
        - ``TransactionExpired``: the chain moved past
          ``last_valid_block_height`` on consecutive height samples and a
          history-searching status check still cannot see the signature; the
          transaction can never land and is safe to resubmit.
        - ``ConfirmationTimeout``: the outcome could not be determined
          (persistent RPC errors or the poll cap); check the signature
          on-chain before resubmitting.
        """
        consecutive_failures = 0
        over_bound_samples = 0

        for _ in range(_MAX_CONFIRMATION_POLLS):
            statuses: Optional[list[Optional[TransactionStatus]]] = None
            try:
                statuses = await self.get_signature_statuses([signature])
                consecutive_failures = 0
            except Exception as error:
                consecutive_failures += 1
                if consecutive_failures >= _MAX_CONSECUTIVE_POLL_FAILURES:
                    raise ConfirmationTimeout(signature) from error

            if statuses is not None:
                status = statuses[0] if statuses else None
                if status is not None and _is_transaction_confirmed(status):
                    if status.err is not None:
                        raise TransactionFailed(signature, str(status.err))
                    return
                if status is not None:
                    # Seen but below confirmed — keep waiting (failed
                    # transactions land in blocks like any other, so an
                    # on-chain error is also reported once confirmed) and
                    # restart expiry evidence: a sighting means the
                    # transaction is live, so expiry must be re-proven from
                    # scratch afterwards.
                    over_bound_samples = 0
                if status is None and last_valid_block_height is not None:
                    # Unseen — sample the block height. Expiry requires
                    # _EXPIRY_HEIGHT_SAMPLES consecutive over-bound samples
                    # (a single reading can come from a forward-skewed node,
                    # and each sample follows a fresh unseen status), then is
                    # still verified against ledger history before being
                    # declared.
                    try:
                        block_height = await self.get_block_height()
                    except Exception:
                        # Height unavailable — reset below: expiry evidence
                        # must be strictly consecutive over-bound readings.
                        block_height = None
                    if (
                        block_height is not None
                        and block_height > last_valid_block_height
                    ):
                        over_bound_samples += 1
                    else:
                        over_bound_samples = 0
                    if over_bound_samples >= _EXPIRY_HEIGHT_SAMPLES:
                        # Search ledger history before declaring expiry — the
                        # recent-status cache can evict landed transactions,
                        # and TransactionExpired promises resubmit safety.
                        history: Optional[list[Optional[TransactionStatus]]]
                        try:
                            history = await self.get_signature_statuses(
                                [signature], search_transaction_history=True
                            )
                        except Exception:
                            # Could not verify — keep polling until the cap.
                            history = None
                        if history is not None:
                            landed = history[0] if history else None
                            if landed is None:
                                raise TransactionExpired(signature)
                            if _is_transaction_confirmed(landed):
                                if landed.err is not None:
                                    raise TransactionFailed(
                                        signature, str(landed.err)
                                    )
                                return
                            # Landed but below confirmed — keep waiting and
                            # restart expiry evidence.
                            over_bound_samples = 0

            await asyncio.sleep(_CONFIRMATION_POLL_INTERVAL_SECS)

        raise ConfirmationTimeout(signature)

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
