from types import SimpleNamespace

import pytest
from solders.account import Account
from solders.hash import Hash
from solders.message import Message
from solders.pubkey import Pubkey
from solders.transaction import Transaction
from spl.token.constants import TOKEN_PROGRAM_ID, WRAPPED_SOL_MINT
from spl.token.instructions import get_associated_token_address

from lightcone_sdk import CanonicalWsolAccountInfo
from lightcone_sdk.error import SdkError
from lightcone_sdk.rpc import Rpc
from lightcone_sdk.rpc_failover import RpcFailoverState


class _StubConnection:
    def __init__(self, value: object) -> None:
        self.value = value

    async def get_minimum_balance_for_rent_exemption(
        self, _data_len: int, _commitment: object
    ) -> SimpleNamespace:
        return SimpleNamespace(value=self.value)

    async def get_fee_for_message(
        self, _message: Message, _commitment: object
    ) -> SimpleNamespace:
        return SimpleNamespace(value=self.value)

    async def get_account_info(
        self, _address: Pubkey, _commitment: object
    ) -> SimpleNamespace:
        """Model solana-py's account-info response wrapper."""
        return SimpleNamespace(value=self.value)


class _StubClient:
    def __init__(self, connection: _StubConnection) -> None:
        self._primary_connection = connection
        self._backup_connection = None
        self._rpc_failover_state = RpcFailoverState()

    @property
    def connection(self) -> _StubConnection:
        return self._primary_connection

    @property
    def rpc_failover_state(self) -> RpcFailoverState:
        return self._rpc_failover_state


def _rpc(value: object) -> Rpc:
    return Rpc(_StubClient(_StubConnection(value)))  # type: ignore[arg-type]


def _prepared_transaction() -> Transaction:
    return Transaction.new_unsigned(
        Message.new_with_blockhash([], Pubkey.new_unique(), Hash.new_unique())
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [-1, 1.5, True, 2**64])
async def test_rent_exemption_rejects_non_u64_lamports(value: object) -> None:
    with pytest.raises(SdkError, match="non-negative u64"):
        await _rpc(value).minimum_balance_for_rent_exemption(165)


@pytest.mark.asyncio
@pytest.mark.parametrize("value", [-1, 1.5, True, 2**64])
async def test_fee_estimate_rejects_non_u64_lamports(value: object) -> None:
    with pytest.raises(SdkError, match="non-negative u64"):
        await _rpc(value).estimate_prepared_transaction_fee(_prepared_transaction())


def _canonical_account_data(
    wallet: Pubkey,
    *,
    mint: Pubkey = WRAPPED_SOL_MINT,
    amount_lamports: int = 500_000_000,
    state: int = 1,
    delegate_option: int = 0,
    native_option: int = 1,
    native_reserve_lamports: int = 2_039_280,
    close_authority: Pubkey | None = None,
) -> bytes:
    """Encode configurable legacy-token account data without SDK builders."""
    data = bytearray(165)
    data[0:32] = bytes(mint)
    data[32:64] = bytes(wallet)
    data[64:72] = amount_lamports.to_bytes(8, "little")
    data[72:76] = delegate_option.to_bytes(4, "little")
    data[108] = state
    data[109:113] = native_option.to_bytes(4, "little")
    data[113:121] = native_reserve_lamports.to_bytes(8, "little")
    if close_authority is not None:
        data[129:133] = (1).to_bytes(4, "little")
        data[133:165] = bytes(close_authority)
    return bytes(data)


@pytest.mark.asyncio
async def test_canonical_wsol_account_validation_accepts_only_tokenkeg_native_account() -> (
    None
):
    """Return exact lamports while preserving delegated boolean presence."""
    wallet = Pubkey.new_unique()
    address = get_associated_token_address(wallet, WRAPPED_SOL_MINT, TOKEN_PROGRAM_ID)
    # The extra 1_000_000 lamports are a valid unsynchronized donation. Exact
    # inspection exposes it without pretending it is part of the token amount.
    valid = Account(503_039_280, _canonical_account_data(wallet), TOKEN_PROGRAM_ID)
    exact = await _rpc(valid).canonical_wsol_account_info(address, wallet)
    assert exact == CanonicalWsolAccountInfo(
        account_lamports=503_039_280,
        token_amount_lamports=500_000_000,
        native_reserve_lamports=2_039_280,
    )
    assert await _rpc(valid).canonical_wsol_account_exists(address, wallet)

    with pytest.raises(SdkError, match="Tokenkeg native-mint ATA"):
        await _rpc(valid).canonical_wsol_account_info(Pubkey.new_unique(), wallet)

    invalid = Account(502_039_280, _canonical_account_data(wallet), Pubkey.default())
    with pytest.raises(SdkError, match="legacy Token Program"):
        await _rpc(invalid).canonical_wsol_account_exists(address, wallet)


@pytest.mark.asyncio
async def test_canonical_wsol_account_info_distinguishes_missing_from_invalid() -> None:
    """Return None only for absence; reject every occupied incompatible shape."""
    wallet = Pubkey.new_unique()
    address = get_associated_token_address(wallet, WRAPPED_SOL_MINT, TOKEN_PROGRAM_ID)
    assert await _rpc(None).canonical_wsol_account_info(address, wallet) is None
    assert not await _rpc(None).canonical_wsol_account_exists(address, wallet)

    invalid_accounts = [
        Account(502_039_280, bytes(164), TOKEN_PROGRAM_ID),
        Account(
            502_039_280,
            _canonical_account_data(wallet, mint=Pubkey.new_unique()),
            TOKEN_PROGRAM_ID,
        ),
        Account(
            502_039_280,
            _canonical_account_data(Pubkey.new_unique()),
            TOKEN_PROGRAM_ID,
        ),
        Account(
            502_039_280,
            _canonical_account_data(wallet, state=2),
            TOKEN_PROGRAM_ID,
        ),
        Account(
            502_039_280,
            _canonical_account_data(wallet, delegate_option=2),
            TOKEN_PROGRAM_ID,
        ),
        Account(
            502_039_280,
            _canonical_account_data(wallet, native_option=0),
            TOKEN_PROGRAM_ID,
        ),
        Account(
            502_039_280,
            _canonical_account_data(wallet, close_authority=Pubkey.new_unique()),
            TOKEN_PROGRAM_ID,
        ),
        Account(502_039_279, _canonical_account_data(wallet), TOKEN_PROGRAM_ID),
    ]
    for invalid in invalid_accounts:
        with pytest.raises(SdkError, match="canonical WSOL"):
            await _rpc(invalid).canonical_wsol_account_info(address, wallet)


@pytest.mark.asyncio
@pytest.mark.parametrize("lamports", [-1, 1.5, True, 2**64])
async def test_canonical_wsol_account_info_rejects_non_u64_account_lamports(
    lamports: object,
) -> None:
    """Do not coerce the full close return at the RPC trust boundary."""
    wallet = Pubkey.new_unique()
    raw = SimpleNamespace(
        lamports=lamports,
        data=_canonical_account_data(wallet),
        owner=TOKEN_PROGRAM_ID,
    )
    with pytest.raises(SdkError, match="non-negative u64"):
        await _rpc(raw).canonical_wsol_account_info(
            get_associated_token_address(wallet, WRAPPED_SOL_MINT, TOKEN_PROGRAM_ID),
            wallet,
        )
