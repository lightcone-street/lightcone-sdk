"""Tests for transaction confirmation — Rpc.confirm_signature.

Drives the poll loop with a stubbed connection: the stub returns a scripted
sequence of get_signature_statuses results (repeating the last entry), so each
terminal outcome — confirmed, failed on-chain, expired, unknown — is exercised
without a network.
"""

from __future__ import annotations

from collections.abc import Sequence
from types import SimpleNamespace

import pytest
from solders.signature import Signature
from solders.transaction_status import TransactionConfirmationStatus

from lightcone_sdk.error import (
    ConfirmationTimeout,
    TransactionExpired,
    TransactionFailed,
)
from lightcone_sdk.rpc import Rpc
from lightcone_sdk.rpc_failover import RpcFailoverState

SIGNATURE = str(Signature.default())


def _status(
    confirmation_status: TransactionConfirmationStatus,
    err: object | None = None,
) -> SimpleNamespace:
    return SimpleNamespace(
        slot=1, confirmations=1, err=err, confirmation_status=confirmation_status
    )


class _StubConnection:
    """Scripted AsyncClient stand-in for the confirmation poll loop.

    ``history`` scripts responses to history-searching status calls; when
    omitted, those calls fall through to the regular ``sequence``.
    ``block_height`` may be a single value or a per-call sequence (repeating
    the last entry, raising entries that are exceptions).
    """

    def __init__(
        self,
        sequence: Sequence[Sequence[SimpleNamespace | None] | Exception],
        block_height: int | Sequence[int | Exception] = 0,
        history: Sequence[Sequence[SimpleNamespace | None]] | None = None,
    ):
        self._sequence = list(sequence)
        self._block_heights = (
            list(block_height) if isinstance(block_height, Sequence) else [block_height]
        )
        self._history = list(history) if history is not None else None
        self.status_calls = 0
        self.history_calls = 0
        self.height_calls = 0

    async def get_signature_statuses(
        self, signatures: object, search_transaction_history: bool = False
    ) -> SimpleNamespace:
        if search_transaction_history and self._history is not None:
            step = self._history[min(self.history_calls, len(self._history) - 1)]
            self.history_calls += 1
            return SimpleNamespace(value=list(step))
        step = self._sequence[min(self.status_calls, len(self._sequence) - 1)]
        self.status_calls += 1
        if isinstance(step, Exception):
            raise step
        return SimpleNamespace(value=list(step))

    async def get_block_height(self, commitment: object = None) -> SimpleNamespace:
        height = self._block_heights[
            min(self.height_calls, len(self._block_heights) - 1)
        ]
        self.height_calls += 1
        if isinstance(height, Exception):
            raise height
        return SimpleNamespace(value=height)


class _StubClient:
    """Minimal LightconeClient stand-in satisfying the failover helpers."""

    def __init__(self, connection: _StubConnection):
        self._primary_connection = connection
        self._backup_connection = None
        self._rpc_failover_state = RpcFailoverState()

    @property
    def connection(self) -> _StubConnection:
        return self._primary_connection

    @property
    def rpc_failover_state(self) -> RpcFailoverState:
        return self._rpc_failover_state


def _rpc(
    sequence: Sequence[Sequence[SimpleNamespace | None] | Exception],
    block_height: int | Sequence[int | Exception] = 0,
    history: Sequence[Sequence[SimpleNamespace | None]] | None = None,
) -> tuple[Rpc, _StubConnection]:
    connection = _StubConnection(sequence, block_height, history)
    return Rpc(_StubClient(connection)), connection  # type: ignore[arg-type]


@pytest.mark.asyncio
async def test_resolves_once_signature_reaches_confirmed() -> None:
    rpc, connection = _rpc(
        [
            [_status(TransactionConfirmationStatus.Processed)],
            [_status(TransactionConfirmationStatus.Confirmed)],
        ]
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    assert connection.status_calls == 2


@pytest.mark.asyncio
async def test_returns_confirmed_status_with_slot() -> None:
    rpc, _ = _rpc([[_status(TransactionConfirmationStatus.Confirmed)]])
    confirmed = await rpc.confirm_signature_status(SIGNATURE, 100)
    assert confirmed.slot == 1


@pytest.mark.asyncio
async def test_raises_transaction_failed_when_landed_with_error() -> None:
    rpc, _ = _rpc(
        [
            [
                _status(
                    TransactionConfirmationStatus.Confirmed,
                    err={"InstructionError": [0, {"Custom": 42}]},
                )
            ]
        ]
    )
    with pytest.raises(TransactionFailed) as raised:
        await rpc.confirm_signature(SIGNATURE, 100)
    assert raised.value.signature == SIGNATURE
    assert "Custom" in str(raised.value)


@pytest.mark.asyncio
async def test_raises_transaction_expired_when_unseen_past_height() -> None:
    rpc, connection = _rpc([[None]], block_height=101, history=[[None]])
    with pytest.raises(TransactionExpired) as raised:
        await rpc.confirm_signature(SIGNATURE, 100)
    assert raised.value.signature == SIGNATURE
    # Expiry is only declared after a history-searching check comes back empty.
    assert connection.history_calls == 1


@pytest.mark.asyncio
async def test_resolves_on_grace_poll_after_expiry_observed() -> None:
    rpc, connection = _rpc(
        [[None], [_status(TransactionConfirmationStatus.Confirmed)]],
        block_height=101,
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    assert connection.status_calls == 2


@pytest.mark.asyncio
async def test_history_check_rescues_landed_transaction() -> None:
    rpc, connection = _rpc(
        [[None]],
        block_height=101,
        history=[[_status(TransactionConfirmationStatus.Confirmed)]],
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    assert connection.history_calls == 1


@pytest.mark.asyncio
async def test_single_skewed_height_sample_does_not_expire() -> None:
    rpc, connection = _rpc(
        [[None], [None], [_status(TransactionConfirmationStatus.Confirmed)]],
        block_height=[101, 99],
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    # One over-bound sample followed by an under-bound one resets the streak,
    # so no expiry (and no history lookup) happens.
    assert connection.history_calls == 0
    assert connection.height_calls == 2


@pytest.mark.asyncio
async def test_status_poll_failure_restarts_expiry_evidence() -> None:
    rpc, connection = _rpc(
        [
            [None],
            RuntimeError("boom"),
            [None],
            [_status(TransactionConfirmationStatus.Confirmed)],
        ],
        block_height=101,
        history=[[None]],
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    # The failed status poll broke the streak, so the unseen polls on either
    # side of it never combined into an expiry declaration.
    assert connection.history_calls == 0


@pytest.mark.asyncio
async def test_height_failure_restarts_expiry_evidence() -> None:
    rpc, connection = _rpc(
        [
            [None],
            [None],
            [None],
            [_status(TransactionConfirmationStatus.Confirmed)],
        ],
        block_height=[101, RuntimeError("boom"), 101],
        history=[[None]],
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    # The failed height poll broke the streak, so the over-bound readings on
    # either side of it never combined into an expiry declaration.
    assert connection.history_calls == 0
    assert connection.height_calls == 3


@pytest.mark.asyncio
async def test_processed_sighting_restarts_expiry_evidence() -> None:
    rpc, connection = _rpc(
        [
            [None],
            [_status(TransactionConfirmationStatus.Processed)],
            [None],
            [_status(TransactionConfirmationStatus.Confirmed)],
        ],
        block_height=101,
        history=[[None]],
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    # The processed sighting reset the over-bound streak, so the lone unseen
    # poll after it never reached the history/expiry stage.
    assert connection.history_calls == 0
    assert connection.status_calls == 4


@pytest.mark.asyncio
async def test_history_sighting_restarts_expiry_evidence() -> None:
    rpc, connection = _rpc(
        [[None], [None], [None], [_status(TransactionConfirmationStatus.Confirmed)]],
        block_height=101,
        history=[[_status(TransactionConfirmationStatus.Processed)]],
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    # The history sighting reset the streak, so only one lookup happened
    # before the transaction confirmed.
    assert connection.history_calls == 1


@pytest.mark.asyncio
async def test_unknown_expiry_bound_never_reports_expired() -> None:
    rpc, connection = _rpc(
        [[None], [_status(TransactionConfirmationStatus.Confirmed)]],
        block_height=101,
    )
    await rpc.confirm_signature(SIGNATURE, None)
    # Without a bound the loop only polls statuses — no expiry machinery runs.
    assert connection.status_calls == 2
    assert connection.history_calls == 0


@pytest.mark.asyncio
async def test_raises_confirmation_timeout_after_persistent_failures() -> None:
    rpc, connection = _rpc([RuntimeError("boom")])
    with pytest.raises(ConfirmationTimeout) as raised:
        await rpc.confirm_signature(SIGNATURE, 100)
    assert raised.value.signature == SIGNATURE
    assert connection.status_calls == 3
