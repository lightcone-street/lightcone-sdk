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
    """Scripted AsyncClient stand-in for the confirmation poll loop."""

    def __init__(
        self,
        sequence: Sequence[Sequence[SimpleNamespace | None] | Exception],
        block_height: int = 0,
    ):
        self._sequence = list(sequence)
        self._block_height = block_height
        self.status_calls = 0

    async def get_signature_statuses(self, signatures: object) -> SimpleNamespace:
        step = self._sequence[min(self.status_calls, len(self._sequence) - 1)]
        self.status_calls += 1
        if isinstance(step, Exception):
            raise step
        return SimpleNamespace(value=list(step))

    async def get_block_height(self) -> SimpleNamespace:
        return SimpleNamespace(value=self._block_height)


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
    block_height: int = 0,
) -> tuple[Rpc, _StubConnection]:
    connection = _StubConnection(sequence, block_height)
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
    rpc, _ = _rpc([[None]], block_height=101)
    with pytest.raises(TransactionExpired) as raised:
        await rpc.confirm_signature(SIGNATURE, 100)
    assert raised.value.signature == SIGNATURE


@pytest.mark.asyncio
async def test_resolves_on_grace_poll_after_expiry_observed() -> None:
    rpc, connection = _rpc(
        [[None], [_status(TransactionConfirmationStatus.Confirmed)]],
        block_height=101,
    )
    await rpc.confirm_signature(SIGNATURE, 100)
    assert connection.status_calls == 2


@pytest.mark.asyncio
async def test_raises_confirmation_timeout_after_persistent_failures() -> None:
    rpc, connection = _rpc([RuntimeError("boom")])
    with pytest.raises(ConfirmationTimeout) as raised:
        await rpc.confirm_signature(SIGNATURE, 100)
    assert raised.value.signature == SIGNATURE
    assert connection.status_calls == 3
