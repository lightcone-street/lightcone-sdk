"""Read Solana state, prepare exact transaction fees, and confirm submissions.

RPC failover state flow: The trigger is an operation on the active connection. The
handoff retries the same endpoint before trying the configured alternate. The guard
allows failover only for infrastructure errors. The result returns from a working
endpoint, propagates a configured alternate's failure, or preserves the retry
failure when no alternate exists. Recovery restores the primary after its cooldown.

Confirmation state flow: The trigger is a submitted signature entering polling.
The handoff sends status and block-height reads through RPC failover. Confirmation,
on-chain failure, consecutive-error, and repeated-expiry guards choose the result.
Recovery resets expiry evidence after gaps or live sightings and checks ledger
history before reporting safe expiry. Three consecutive signature-status poll
failures or the poll cap raise ``ConfirmationTimeout``.
"""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import TYPE_CHECKING, TypeVar, cast

from solders.hash import Hash
from solders.instruction import Instruction
from solders.message import Message
from solders.pubkey import Pubkey
from solders.signature import Signature
from solders.transaction import Transaction
from solders.transaction_status import TransactionConfirmationStatus
from spl.token._layouts import ACCOUNT_LAYOUT
from spl.token.constants import TOKEN_PROGRAM_ID, WRAPPED_SOL_MINT
from spl.token.instructions import get_associated_token_address

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
from .program.errors import AccountNotFoundError
from .program.pda import (
    get_event_authority_pda,
    get_exchange_pda,
    get_global_deposit_pda,
    get_user_global_deposit_pda,
)
from .program.types import Exchange, GlobalDepositToken
from .rpc_failover import (
    FAST_RETRY_DELAY_SECS,
    ActiveRpc,
    is_infrastructure_error,
)

#: Largest exact non-negative lamport value accepted by Solana u64 fields.
_MAX_SOLANA_LAMPORTS = 2**64 - 1


@dataclass(frozen=True)
class CanonicalWsolAccountInfo:
    """Store exact live facts for a validated canonical Tokenkeg WSOL account.

    ``canonical_wsol_account_info`` returns all fields from one confirmed account
    read. ``account_lamports`` includes native-token rent and donated lamports.
    ``token_amount_lamports`` is the decoded SPL token amount.
    ``native_reserve_lamports`` is the decoded native-account rent reserve. All
    fields are integer lamports in Solana's unsigned 64-bit range. Direct
    dataclass construction does not perform RPC validation.
    """

    account_lamports: int
    token_amount_lamports: int
    native_reserve_lamports: int


def _rpc_lamports(value: object, label: str) -> int:
    """Return an RPC value only when it is exact non-negative ``u64`` lamports."""
    if (
        isinstance(value, bool)
        or not isinstance(value, int)
        or value < 0
        or value > _MAX_SOLANA_LAMPORTS
    ):
        raise SdkError(f"{label} must fit the non-negative u64 lamport range")
    return value


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


def require_connection(client: LightconeClient) -> AsyncClient:
    """Resolve the currently-active Solana RPC client, or raise if not configured."""
    conn = client.connection
    if conn is None:
        raise SdkError("Solana RPC not configured — use .rpc_url() on the builder")
    return cast("AsyncClient", conn)


def _resolve_connection_for(
    client: LightconeClient, target: ActiveRpc
) -> AsyncClient | None:
    """Resolve the connection for a specific endpoint."""
    if target == ActiveRpc.PRIMARY:
        return cast("AsyncClient | None", client._primary_connection)
    return cast("AsyncClient | None", client._backup_connection)


async def _connection_with_failover(
    client: LightconeClient,
    operation: Callable[[AsyncClient], Awaitable[T]],
) -> T:
    """Execute an RPC operation with fast retry + failover.

    Same flow as Rust's ``solana_rpc_with_failover``.
    """
    state = client.rpc_failover_state
    state.maybe_recover_to_primary()
    original_active = state.active

    active_conn = require_connection(client)

    # First attempt.
    try:
        return await operation(active_conn)
    except Exception as first_error:
        if not is_infrastructure_error(first_error):
            raise

    # Fast retry on same connection.
    await asyncio.sleep(FAST_RETRY_DELAY_SECS)
    retry_failure: Exception
    try:
        return await operation(active_conn)
    except Exception as error:
        retry_failure = error
        if not is_infrastructure_error(error):
            raise

    # Flip and try the other connection.
    other_target = (
        ActiveRpc.BACKUP if original_active == ActiveRpc.PRIMARY else ActiveRpc.PRIMARY
    )
    other_conn = _resolve_connection_for(client, other_target)
    if other_conn is not None:
        try:
            result = await operation(other_conn)
            if other_target == ActiveRpc.PRIMARY:
                state.flip_to_primary()
            else:
                state.flip_to_backup()
            return result
        except Exception:
            raise

    raise retry_failure


class Rpc:
    """RPC sub-client — PDA helpers, account fetchers, and blockhash access."""

    def __init__(self, client: LightconeClient) -> None:
        self._client = client

    # ── PDA helpers (sync, always available) ─────────────────────────────

    def get_exchange_pda(self) -> Pubkey:
        """Get the Exchange PDA."""
        pda, _ = get_exchange_pda(self._client.program_id)
        return pda

    def get_event_authority_pda(self) -> Pubkey:
        """Get the event-authority PDA appended to every public instruction."""
        pda, _ = get_event_authority_pda(self._client.program_id)
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
        (
            blockhash,
            _last_valid_block_height,
        ) = await self.get_latest_blockhash_with_height()
        return blockhash

    async def get_latest_blockhash_with_height(self) -> tuple[Hash, int]:
        """Get the latest blockhash and the last block height at which it is valid.

        The blockhash is requested at ``confirmed`` commitment (pinned, not
        the connection's default — matching the Rust and TypeScript SDKs).
        Past the returned height, a transaction built on the blockhash can
        never land, which is what makes expiry detection in
        ``confirm_signature`` safe.
        """
        from solana.rpc.commitment import Confirmed

        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_latest_blockhash(Confirmed),
        )
        value = response.value  # type: ignore[union-attr]
        return value.blockhash, value.last_valid_block_height

    async def get_block_height(self) -> int:
        """Get the current block height at ``confirmed`` commitment (pinned)."""
        from solana.rpc.commitment import Confirmed

        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_block_height(Confirmed),
        )
        return int(response.value)

    async def account_exists(self, address: Pubkey) -> bool:
        """Return account presence at confirmed commitment.

        A missing account returns ``False``. Transport and RPC failures propagate
        instead of being interpreted as absence.
        """
        from solana.rpc.commitment import Confirmed

        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_account_info(address, Confirmed),
        )
        return response.value is not None

    async def canonical_wsol_account_exists(
        self, address: Pubkey, wallet: Pubkey
    ) -> bool:
        """Return whether the validated canonical WSOL account is present.

        This compatibility method delegates address, owner, state, authority,
        and lamport checks to :meth:`canonical_wsol_account_info`.
        """
        return await self.canonical_wsol_account_info(address, wallet) is not None

    async def canonical_wsol_account_info(
        self, address: Pubkey, wallet: Pubkey
    ) -> CanonicalWsolAccountInfo | None:
        """Return exact confirmed facts for the wallet's canonical WSOL account.

        Missing accounts return ``None``. Present accounts must be initialized,
        unfrozen legacy Token Program native-mint accounts controlled by
        ``wallet`` at its exact Tokenkeg native-mint ATA. Invalid derivation,
        ownership, layout, native reserve, close authority, or integer lamport
        data fails closed rather than looking absent.
        """
        from solana.rpc.commitment import Confirmed

        canonical = get_associated_token_address(
            wallet, WRAPPED_SOL_MINT, TOKEN_PROGRAM_ID
        )
        if address != canonical:
            raise SdkError(
                "canonical WSOL address is not the wallet's Tokenkeg native-mint ATA"
            )
        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_account_info(address, Confirmed),
        )
        info = response.value
        if info is None:
            return None
        if info.owner != TOKEN_PROGRAM_ID:
            raise SdkError(
                "canonical WSOL account is not owned by the legacy Token Program"
            )
        if len(info.data) != ACCOUNT_LAYOUT.sizeof():
            raise SdkError("canonical WSOL token account has invalid data length")
        try:
            account = ACCOUNT_LAYOUT.parse(bytes(info.data))
            mint = Pubkey.from_bytes(account.mint)
            owner = Pubkey.from_bytes(account.owner)
            close_authority = (
                Pubkey.from_bytes(account.close_authority)
                if account.close_authority_option == 1
                else None
            )
        except Exception as error:
            raise SdkError(
                f"canonical WSOL token account is invalid: {error}"
            ) from error
        if (
            mint != WRAPPED_SOL_MINT
            or owner != wallet
            or account.state != 1
            or account.delegate_option not in (0, 1)
            or account.is_native_option != 1
            or account.close_authority_option not in (0, 1)
            or (close_authority is not None and close_authority != wallet)
        ):
            raise SdkError(
                "canonical WSOL token account has incompatible mint, authority, or native state"
            )
        account_lamports = _rpc_lamports(
            info.lamports, "canonical WSOL account balance"
        )
        token_amount_lamports = _rpc_lamports(
            account.amount, "canonical WSOL token amount"
        )
        native_reserve_lamports = _rpc_lamports(
            account.is_native, "canonical WSOL native reserve"
        )
        accounted_lamports = token_amount_lamports + native_reserve_lamports
        if (
            accounted_lamports > _MAX_SOLANA_LAMPORTS
            or accounted_lamports > account_lamports
        ):
            raise SdkError(
                "canonical WSOL token amount and native reserve exceed account lamports"
            )
        return CanonicalWsolAccountInfo(
            account_lamports=account_lamports,
            token_amount_lamports=token_amount_lamports,
            native_reserve_lamports=native_reserve_lamports,
        )

    async def minimum_balance_for_rent_exemption(self, data_len: int) -> int:
        """Return confirmed rent-exempt lamports for ``data_len`` account bytes.

        Missing, negative, inexact, boolean, or out-of-range RPC values raise
        :class:`SdkError` rather than becoming a guessed rent value.
        """
        from solana.rpc.commitment import Confirmed

        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_minimum_balance_for_rent_exemption(
                data_len, Confirmed
            ),
        )
        return _rpc_lamports(response.value, "rent-exempt minimum")

    async def balance_lamports(self, fee_payer: Pubkey) -> int:
        """Return the confirmed Native SOL Balance for ``fee_payer``, in lamports."""
        from solana.rpc.commitment import Confirmed

        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_balance(fee_payer, Confirmed),
        )
        return _rpc_lamports(response.value, "fee-payer balance")

    async def prepare_and_estimate_transaction_fee(
        self, transaction: Transaction
    ) -> int:
        """Prepare ``transaction`` with a fresh blockhash and return its fee.

        The return value is exact lamports for that message at confirmed
        commitment. The unsigned transaction is updated to carry the blockhash
        used by fee estimation.
        """
        blockhash = await self.get_latest_blockhash()
        transaction.partial_sign([], blockhash)
        return await self.estimate_prepared_transaction_fee(transaction)

    async def estimate_prepared_transaction_fee(self, transaction: Transaction) -> int:
        """Return the confirmed live fee for an already-prepared message.

        The transaction is not mutated. A missing blockhash, unavailable estimate,
        or negative, inexact, boolean, or out-of-range fee raises
        :class:`SdkError`; unavailable never becomes zero.
        """
        from solana.rpc.commitment import Confirmed
        from solders.hash import Hash

        if transaction.message.recent_blockhash == Hash.default():
            raise SdkError("prepared transaction is missing a recent blockhash")
        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.get_fee_for_message(transaction.message, Confirmed),
        )
        fee = response.value
        if fee is None:
            raise SdkError("transaction fee estimate is unavailable")
        return _rpc_lamports(fee, "transaction fee estimate")

    async def send_raw_transaction(self, tx_bytes: bytes) -> str:
        """Submit a signed transaction, returning its signature.

        Fire-and-forget: confirmation is skipped explicitly rather than left
        to solana-py's ``TxOpts`` defaults — waiting is ``confirm_signature``'s
        job, with its terminal error taxonomy. Preflight simulates at
        ``confirmed`` commitment, matching the other submit paths.
        """
        from solana.rpc.commitment import Confirmed
        from solana.rpc.types import TxOpts

        response = await _connection_with_failover(
            self._client,
            lambda conn: conn.send_raw_transaction(
                tx_bytes,
                opts=TxOpts(
                    skip_confirmation=True,
                    preflight_commitment=Confirmed,
                ),
            ),
        )
        return str(response.value)  # type: ignore[attr-defined]

    async def send_raw_transaction_once(self, tx_bytes: bytes) -> str:
        """Submit signed bytes once on the active RPC and return the signature.

        This method does not retry or fail over because a transport error does not
        prove that the active endpoint rejected the transaction.
        """
        from solana.rpc.commitment import Confirmed
        from solana.rpc.types import TxOpts

        response = await require_connection(self._client).send_raw_transaction(
            tx_bytes,
            opts=TxOpts(
                skip_confirmation=True,
                preflight_commitment=Confirmed,
            ),
        )
        return str(response.value)  # type: ignore[attr-defined]

    async def get_signature_statuses(
        self,
        signatures: list[str],
        search_transaction_history: bool = False,
    ) -> list[TransactionStatus | None]:
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
        statuses: list[TransactionStatus | None] = response.value
        return statuses

    async def confirm_signature(
        self, signature: str, last_valid_block_height: int | None
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
        await self.confirm_signature_status(signature, last_valid_block_height)

    async def confirm_signature_status(
        self, signature: str, last_valid_block_height: int | None
    ) -> TransactionStatus:
        """Same as :meth:`confirm_signature`, returning the confirmed status."""
        consecutive_failures = 0
        over_bound_samples = 0

        for _ in range(_MAX_CONFIRMATION_POLLS):
            statuses: list[TransactionStatus | None] | None = None
            try:
                statuses = await self.get_signature_statuses([signature])
                consecutive_failures = 0
            except Exception as error:
                consecutive_failures += 1
                # A failed poll is a gap in expiry evidence — restart it.
                over_bound_samples = 0
                if consecutive_failures >= _MAX_CONSECUTIVE_POLL_FAILURES:
                    raise ConfirmationTimeout(signature) from error

            if statuses is not None:
                status = statuses[0] if statuses else None
                if status is not None and _is_transaction_confirmed(status):
                    if status.err is not None:
                        raise TransactionFailed(signature, str(status.err))
                    return status
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
                        history: list[TransactionStatus | None] | None
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
                                    raise TransactionFailed(signature, str(landed.err))
                                return landed
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


__all__ = ["CanonicalWsolAccountInfo", "Rpc", "require_connection"]
