"""Tests for the wallet-adapter submit path — _sign_and_submit_tx_inner.

Exercises the real submit path in-process (stubbing only the RPC edges):
the SDK sets a fresh blockhash via ``Transaction.partial_sign([], blockhash)``
before external signing, and the expiry bound is kept only when the signed
bytes still carry that blockhash.
"""

from __future__ import annotations

import pytest
from solders.hash import Hash
from solders.keypair import Keypair
from solders.message import Message
from solders.pubkey import Pubkey
from solders.signature import Signature
from solders.system_program import TransferParams, transfer
from solders.transaction import Transaction

from lightcone_sdk.client import LightconeClient
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


def _wallet_client(signer: ExternalSigner) -> LightconeClient:
    client = LightconeClient(
        LightconeHttp("http://localhost:0"),
        signing_strategy=SigningStrategy.wallet_adapter(signer),
    )

    async def fake_blockhash_with_height() -> tuple[Hash, int]:
        return FRESH_BLOCKHASH, LAST_VALID_BLOCK_HEIGHT

    async def fake_rpc_call(body: dict) -> dict:
        assert body["method"] == "sendTransaction"
        return {"result": str(Signature.default())}

    client._rpc.get_latest_blockhash_with_height = (  # type: ignore[method-assign]
        fake_blockhash_with_height
    )
    client._rpc_call_with_failover = fake_rpc_call  # type: ignore[method-assign]
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
