"""Fail-closed validation for integer lamport values returned by Solana RPC."""

from types import SimpleNamespace

import pytest
from solders.account import Account
from solders.hash import Hash
from solders.message import Message
from solders.pubkey import Pubkey
from solders.transaction import Transaction
from spl.token.constants import TOKEN_PROGRAM_ID, WRAPPED_SOL_MINT

from lightcone_sdk.error import SdkError
from lightcone_sdk.rpc import Rpc
from lightcone_sdk.rpc_failover import RpcFailoverState


class _StubConnection:
    """Return one configured value from the two lamport-bearing RPC methods."""

    def __init__(self, value: object) -> None:
        """Store the raw boundary value without coercion."""
        self.value = value

    async def get_minimum_balance_for_rent_exemption(
        self, _data_len: int, _commitment: object
    ) -> SimpleNamespace:
        """Model solana-py's rent response wrapper."""
        return SimpleNamespace(value=self.value)

    async def get_fee_for_message(
        self, _message: Message, _commitment: object
    ) -> SimpleNamespace:
        """Model solana-py's fee response wrapper."""
        return SimpleNamespace(value=self.value)

    async def get_account_info(
        self, _address: Pubkey, _commitment: object
    ) -> SimpleNamespace:
        """Model solana-py's account-info response wrapper."""
        return SimpleNamespace(value=self.value)


class _StubClient:
    """Expose only the connection and failover state required by :class:`Rpc`."""

    def __init__(self, connection: _StubConnection) -> None:
        """Install the stub as the active primary endpoint."""
        self._primary_connection = connection
        self._backup_connection = None
        self._rpc_failover_state = RpcFailoverState()

    @property
    def connection(self) -> _StubConnection:
        """Return the active stub connection."""
        return self._primary_connection

    @property
    def rpc_failover_state(self) -> RpcFailoverState:
        """Return mutable endpoint authority used by failover wrappers."""
        return self._rpc_failover_state


def _rpc(value: object) -> Rpc:
    """Build an RPC sub-client around one untrusted lamport value."""
    return Rpc(_StubClient(_StubConnection(value)))  # type: ignore[arg-type]


def _prepared_transaction() -> Transaction:
    """Build a message with a non-default blockhash suitable for fee lookup."""
    return Transaction.new_unsigned(
        Message.new_with_blockhash([], Pubkey.new_unique(), Hash.new_unique())
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [-1, 1.5, True, 2**64])
async def test_rent_exemption_rejects_non_u64_lamports(value: object) -> None:
    """Reject negative, inexact, boolean, and overflowing rent responses."""
    with pytest.raises(SdkError, match="non-negative u64"):
        await _rpc(value).minimum_balance_for_rent_exemption(165)


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [-1, 1.5, True, 2**64])
async def test_fee_estimate_rejects_non_u64_lamports(value: object) -> None:
    """Reject negative, inexact, boolean, and overflowing fee responses."""
    with pytest.raises(SdkError, match="non-negative u64"):
        await _rpc(value).estimate_prepared_transaction_fee(_prepared_transaction())


def _canonical_account_data(wallet: Pubkey) -> bytes:
    """Encode one initialized native-mint token account for ``wallet``."""
    data = bytearray(165)
    data[0:32] = bytes(WRAPPED_SOL_MINT)
    data[32:64] = bytes(wallet)
    data[108] = 1
    data[109:113] = (1).to_bytes(4, "little")
    data[113:121] = (2_039_280).to_bytes(8, "little")
    return bytes(data)


@pytest.mark.asyncio
async def test_canonical_wsol_account_validation_accepts_only_tokenkeg_native_account() -> (
    None
):
    """Validate program owner, token layout, native mint, and wallet authority."""
    wallet = Pubkey.new_unique()
    address = Pubkey.new_unique()
    valid = Account(2_039_280, _canonical_account_data(wallet), TOKEN_PROGRAM_ID)
    assert await _rpc(valid).canonical_wsol_account_exists(address, wallet)

    invalid = Account(2_039_280, _canonical_account_data(wallet), Pubkey.default())
    with pytest.raises(SdkError, match="legacy Token Program"):
        await _rpc(invalid).canonical_wsol_account_exists(address, wallet)
