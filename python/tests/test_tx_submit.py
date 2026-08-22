"""Tests for the wallet-adapter submit path — _sign_and_submit_tx_inner.

Exercises the real submit path in-process (stubbing only the RPC edges):
the SDK sets a fresh blockhash via ``Transaction.partial_sign([], blockhash)``
before external signing, and the expiry bound is kept only when the signed
bytes still carry that blockhash.
"""

from __future__ import annotations

from types import SimpleNamespace

import pytest
from solders.hash import Hash
from solders.keypair import Keypair
from solders.message import Message
from solders.pubkey import Pubkey
from solders.signature import Signature
from solders.system_program import TransferParams, transfer
from solders.transaction import Transaction

from lightcone_sdk.client import LightconeClient
from lightcone_sdk.error import SdkError
from lightcone_sdk.http.client import LightconeHttp
from lightcone_sdk.shared.signing import ExternalSigner, SigningStrategy

FRESH_BLOCKHASH = Hash.new_unique()
LAST_VALID_BLOCK_HEIGHT = 123


class _EchoSigner(ExternalSigner):
    """Signs by returning the wire bytes unchanged (blockhash preserved)."""

    async def sign_message(self, message: bytes) -> bytes:
        return message

    async def sign_transaction(self, tx_bytes: bytes) -> bytes:
        return tx_bytes


class _RecordingSigner(_EchoSigner):
    def __init__(self) -> None:
        self.transaction_calls = 0

    async def sign_transaction(self, tx_bytes: bytes) -> bytes:
        self.transaction_calls += 1
        return tx_bytes


class _RehashSigner(ExternalSigner):
    """Simulates a wallet that replaces the blockhash before signing."""

    async def sign_message(self, message: bytes) -> bytes:
        return message

    async def sign_transaction(self, tx_bytes: bytes) -> bytes:
        original = Transaction.from_bytes(tx_bytes)
        rehashed = Transaction.new_unsigned(
            Message.new_with_blockhash(
                _instructions(), original.message.account_keys[0], Hash.new_unique()
            )
        )
        return bytes(rehashed)


def _instructions() -> list:
    keypair = Keypair()
    return [
        transfer(
            TransferParams(
                from_pubkey=keypair.pubkey(),
                to_pubkey=Pubkey.new_unique(),
                lamports=1,
            )
        )
    ]


def _unsigned_tx() -> Transaction:
    message = Message.new_with_blockhash(
        _instructions(), Keypair().pubkey(), Hash.new_unique()
    )
    return Transaction.new_unsigned(message)


def _wallet_client(
    signer: ExternalSigner, expected_bound: int | None = LAST_VALID_BLOCK_HEIGHT
) -> LightconeClient:
    """Build a wallet-adapter client with deterministic submit and confirmation edges."""
    client = LightconeClient(
        LightconeHttp("http://localhost:0"),
        signing_strategy=SigningStrategy.wallet_adapter(signer),
    )

    async def fake_blockhash_with_height() -> tuple[Hash, int]:
        return FRESH_BLOCKHASH, LAST_VALID_BLOCK_HEIGHT

    async def fake_rpc_call(body: dict) -> dict:
        assert body["method"] == "sendTransaction"
        return {"result": str(Signature.default())}

    async def fake_confirm_signature_status(
        signature: str, last_valid_block_height: int | None
    ) -> SimpleNamespace:
        assert signature == str(Signature.default())
        assert last_valid_block_height == expected_bound
        return SimpleNamespace(slot=7)

    async def fake_send_raw_transaction(_tx_bytes: bytes) -> str:
        """Return a signature without contacting RPC; message checks happen earlier."""
        return str(Signature.default())

    client._rpc.get_latest_blockhash_with_height = (  # type: ignore[method-assign]
        fake_blockhash_with_height
    )
    client._rpc_call_with_failover = fake_rpc_call  # type: ignore[method-assign]
    client._rpc.confirm_signature_status = (  # type: ignore[method-assign]
        fake_confirm_signature_status
    )
    client._rpc.send_raw_transaction = fake_send_raw_transaction  # type: ignore[method-assign]
    client._rpc.send_raw_transaction_once = (  # type: ignore[method-assign]
        fake_send_raw_transaction
    )
    return client


@pytest.mark.asyncio
async def test_wallet_submit_sets_fresh_blockhash_and_keeps_bound() -> None:
    client = _wallet_client(_EchoSigner())
    tx = _unsigned_tx()

    signature, bound = await client._sign_and_submit_tx_inner(tx)

    assert signature == str(Signature.default())
    # partial_sign([], blockhash) replaced the caller's stale blockhash.
    assert tx.message.recent_blockhash == FRESH_BLOCKHASH
    # The signer returned the bytes unchanged, so the exact bound is kept.
    assert bound == LAST_VALID_BLOCK_HEIGHT


@pytest.mark.asyncio
async def test_wallet_submit_drops_bound_when_signer_rehashes() -> None:
    client = _wallet_client(_RehashSigner())

    _signature, bound = await client._sign_and_submit_tx_inner(_unsigned_tx())

    # The signed bytes no longer carry the SDK's blockhash, so no expiry
    # bound can be trusted.
    assert bound is None


@pytest.mark.asyncio
async def test_confirmed_submit_uses_explicit_strategy_after_configuration_swap() -> (
    None
):
    validated_signer = _RecordingSigner()
    replacement_signer = _RecordingSigner()
    client = _wallet_client(replacement_signer)

    signature = await client._sign_and_submit_tx_confirmed_with_strategy(
        _unsigned_tx(), SigningStrategy.wallet_adapter(validated_signer)
    )

    assert signature == str(Signature.default())
    assert validated_signer.transaction_calls == 1
    assert replacement_signer.transaction_calls == 0


@pytest.mark.asyncio
async def test_prepared_submit_preserves_the_fee_estimated_message() -> None:
    """Submit the exact fee-estimated message and expose its confirmed slot."""
    tx = _unsigned_tx()
    signer = _EchoSigner()
    signer.wallet_address = str(tx.message.account_keys[0])
    client = _wallet_client(signer, expected_bound=None)
    expected_message = bytes(tx.message)
    expected_blockhash = tx.message.recent_blockhash

    confirmed = await client.sign_and_submit_prepared_tx_confirmed_with_slot(tx)

    assert bytes(tx.message) == expected_message
    assert tx.message.recent_blockhash == expected_blockhash
    assert confirmed.signature == str(Signature.default())
    assert confirmed.slot == 7


@pytest.mark.asyncio
async def test_prepared_submit_rejects_a_signer_blockhash_change() -> None:
    """Reject a wallet mutation before any changed prepared bytes reach RPC."""
    tx = _unsigned_tx()
    signer = _RehashSigner()
    signer.wallet_address = str(tx.message.account_keys[0])
    client = _wallet_client(signer, expected_bound=None)

    with pytest.raises(SdkError, match="changed the fee-prepared transaction message"):
        await client.sign_and_submit_prepared_tx_confirmed_with_slot(tx)


@pytest.mark.asyncio
async def test_prepared_submit_rejects_a_mismatched_signing_wallet() -> None:
    """Reject the wrong wallet before invoking it or submitting bytes."""
    tx = _unsigned_tx()
    signer = _RecordingSigner()
    signer.wallet_address = str(Pubkey.new_unique())
    client = _wallet_client(signer, expected_bound=None)

    with pytest.raises(
        SdkError, match="does not control prepared transaction fee payer"
    ):
        await client.sign_and_submit_prepared_tx_confirmed_with_slot(tx)
    assert signer.transaction_calls == 0


@pytest.mark.asyncio
async def test_prepared_submission_transport_failure_is_sent_once() -> None:
    """Do not retry or fail over signed bytes after an uncertain RPC response."""

    class FailingConnection:
        def __init__(self, message: str) -> None:
            self.message = message
            self.attempts = 0

        async def send_raw_transaction(self, _tx_bytes: bytes, *, opts: object) -> None:
            self.attempts += 1
            raise ConnectionError(self.message)

    keypair = Keypair()
    transaction = Transaction.new_unsigned(
        Message.new_with_blockhash([], keypair.pubkey(), Hash.new_unique())
    )
    primary = FailingConnection("network response was lost")
    backup = FailingConnection("backup must not receive prepared bytes")
    client = LightconeClient(
        LightconeHttp("http://localhost:0"),
        connection=primary,
        backup_connection=backup,
        signing_strategy=SigningStrategy.native(keypair),
    )

    with pytest.raises(ConnectionError, match="network response was lost"):
        await client.sign_and_submit_prepared_tx_confirmed_with_slot(transaction)

    assert primary.attempts == 1
    assert backup.attempts == 0
