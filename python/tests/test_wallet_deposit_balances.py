import json
import time

import pytest
from solders.instruction import AccountMeta, Instruction
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solders.system_program import decode_transfer
from spl.token.constants import (
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    WRAPPED_SOL_MINT,
)
from spl.token.instructions import decode_close_account, decode_sync_native

from lightcone_sdk.auth import AuthCredentials
from lightcone_sdk.domain.position import (
    DepositTokenBalance,
    DepositTokenBalancesSnapshot,
    WalletDepositBalancesApplyResult,
    WalletDepositBalanceSnapshot,
    WalletDepositBalancesState,
    WalletDepositBalanceStatus,
    WalletDepositBalanceStatusEvent,
    WalletDepositBalanceUpdate,
    WalletNativeSolBalanceUpdate,
)
from lightcone_sdk.domain.position.client import Positions
from lightcone_sdk.error import SdkError
from lightcone_sdk.program import get_associated_token_address
from lightcone_sdk.ws import parse_message_in

WSOL_MINT = str(WRAPPED_SOL_MINT)


def balance(mint: str, idle: str) -> DepositTokenBalance:
    return DepositTokenBalance(
        mint=mint,
        idle=idle,
        symbol="TOKEN",
        name="Token",
    )


def initialized_state(wallet: str) -> WalletDepositBalancesState:
    state = WalletDepositBalancesState()
    state.apply_rest_snapshot(
        wallet,
        DepositTokenBalancesSnapshot(
            context_slot=100,
            native_sol_balance="2.000000000",
            balances={WSOL_MINT: balance(WSOL_MINT, "0.500000000")},
        ),
    )
    return state


def test_nested_wallet_balance_wire_variants_are_strictly_decoded() -> None:
    snapshot = parse_message_in(
        json.dumps(
            {
                "type": "wallet_deposit_balances",
                "data": {
                    "event_type": "wallet_deposit_balance_snapshot",
                    "wallet_address": "wallet-a",
                    "context_slot": 10,
                    "native_sol_balance": "1.000000000",
                    "balances": {
                        "MintA": {
                            "mint": "MintA",
                            "idle": "1.000000000",
                            "symbol": "TOKEN",
                            "name": "Token",
                            "icon_url_low": None,
                        }
                    },
                },
            }
        )
    ).data
    native = parse_message_in(
        json.dumps(
            {
                "type": "wallet_deposit_balances",
                "data": {
                    "event_type": "wallet_native_sol_balance_update",
                    "wallet_address": "wallet-a",
                    "context_slot": 11,
                    "native_sol_balance": "1.000000001",
                },
            }
        )
    ).data
    balance_update = parse_message_in(
        json.dumps(
            {
                "type": "wallet_deposit_balances",
                "data": {
                    "event_type": "wallet_deposit_balance_update",
                    "wallet_address": "wallet-a",
                    "context_slot": 12,
                    "balance": {
                        "mint": "MintA",
                        "idle": "2.000000000",
                        "symbol": "TOKEN",
                        "name": "Token",
                    },
                },
            }
        )
    ).data
    status = parse_message_in(
        json.dumps(
            {
                "type": "wallet_deposit_balances",
                "data": {
                    "event_type": "wallet_deposit_balance_status",
                    "wallet_address": "wallet-a",
                    "status": "reconnecting",
                    "code": "RPC_UNAVAILABLE",
                },
            }
        )
    ).data

    assert isinstance(snapshot, WalletDepositBalanceSnapshot)
    assert snapshot.balances["MintA"].icon_url_low is None
    assert isinstance(native, WalletNativeSolBalanceUpdate)
    assert isinstance(balance_update, WalletDepositBalanceUpdate)
    assert balance_update.balance.mint == "MintA"
    assert isinstance(status, WalletDepositBalanceStatusEvent)
    assert status.status is WalletDepositBalanceStatus.RECONNECTING

    malformed = {
        "type": "wallet_deposit_balances",
        "data": {
            "event_type": "wallet_native_sol_balance_update",
            "wallet_address": "wallet-a",
            "context_slot": 11,
            "native_sol_balance": "1.0",
        },
    }
    with pytest.raises(TypeError, match="exactly nine decimal places"):
        parse_message_in(json.dumps(malformed))


@pytest.mark.parametrize(
    "message",
    [
        {"type": "wallet_deposit_balances"},
        {"type": "wallet_deposit_balances", "data": None},
        {"type": "wallet_deposit_balances", "data": []},
        {"type": "wallet_deposit_balances", "data": "invalid"},
        {"type": "wallet_deposit_balances", "data": 1},
    ],
)
def test_wallet_balance_wire_requires_object_data(message: dict[str, object]) -> None:
    with pytest.raises(TypeError, match="data must be an object"):
        parse_message_in(json.dumps(message))


def test_state_replacement_component_updates_zero_removal_and_exact_combined_sol() -> (
    None
):
    state = initialized_state("wallet-a")
    assert state.combined_sol_balance() == "2.500000000"

    assert (
        state.apply_event(
            WalletNativeSolBalanceUpdate(
                event_type="wallet_native_sol_balance_update",
                wallet_address="wallet-b",
                context_slot=101,
                native_sol_balance="9.000000000",
            )
        )
        is WalletDepositBalancesApplyResult.IGNORED
    )
    assert (
        state.apply_event(
            WalletNativeSolBalanceUpdate(
                event_type="wallet_native_sol_balance_update",
                wallet_address="wallet-a",
                context_slot=101,
                native_sol_balance="2.000000001",
            )
        )
        is WalletDepositBalancesApplyResult.APPLIED
    )
    assert state.combined_sol_balance() == "2.500000001"

    state.apply_event(
        WalletDepositBalanceUpdate(
            event_type="wallet_deposit_balance_update",
            wallet_address="wallet-a",
            context_slot=102,
            balance=balance("MintHighPrecision", "0.000000000001"),
        )
    )
    assert "MintHighPrecision" in state.balances
    state.apply_event(
        WalletDepositBalanceUpdate(
            event_type="wallet_deposit_balance_update",
            wallet_address="wallet-a",
            context_slot=103,
            balance=balance("MintHighPrecision", "0.000000000000"),
        )
    )
    assert "MintHighPrecision" not in state.balances

    state.apply_event(
        WalletDepositBalanceSnapshot(
            event_type="wallet_deposit_balance_snapshot",
            wallet_address="wallet-c",
            context_slot=90,
            native_sol_balance="3.000000000",
            balances={},
        )
    )
    assert state.wallet_address == "wallet-c"
    assert state.context_slot == 90
    assert state.combined_sol_balance() == "3.000000000"

    assert (
        state.apply_event(
            WalletDepositBalanceStatusEvent(
                event_type="wallet_deposit_balance_status",
                wallet_address="wallet-c",
                status=WalletDepositBalanceStatus.METADATA_UNAVAILABLE,
                code="METADATA_UNAVAILABLE",
            )
        )
        is WalletDepositBalancesApplyResult.IGNORED
    )


class FakeAuth:
    def __init__(self, credentials: AuthCredentials | None) -> None:
        self._credentials = credentials

    def credentials(self) -> AuthCredentials | None:
        return self._credentials


class FakeConversionClient:
    def __init__(
        self, credentials: AuthCredentials | None, failure: Exception | None = None
    ) -> None:
        self._auth = FakeAuth(credentials)
        self.transactions = []
        self.failure = failure

    def auth(self) -> FakeAuth:
        return self._auth

    async def sign_and_submit_tx_confirmed(self, transaction) -> str:
        self.transactions.append(transaction)
        if self.failure is not None:
            raise self.failure
        return "confirmed-signature"


def compiled_instruction(transaction, index: int) -> Instruction:
    message = transaction.message
    compiled = message.instructions[index]
    keys = message.account_keys
    return Instruction(
        keys[compiled.program_id_index],
        bytes(compiled.data),
        [AccountMeta(keys[i], False, True) for i in bytes(compiled.accounts)],
    )


@pytest.mark.asyncio
async def test_wrap_uses_maintained_builders_and_confirmed_submission() -> None:
    wallet_key = Keypair().pubkey()
    wallet = str(wallet_key)
    client = FakeConversionClient(
        AuthCredentials(
            user_id="user-a",
            wallet_address=wallet,
            expires_at=int(time.time()) + 60,
        )
    )
    state = initialized_state(wallet)

    signature = await Positions(client).wrap_sol("0.250000001", state)  # type: ignore[arg-type]

    assert signature == "confirmed-signature"
    assert len(client.transactions) == 1
    transaction = client.transactions[0]
    program_ids = [
        transaction.message.account_keys[ix.program_id_index]
        for ix in transaction.message.instructions
    ]
    assert program_ids == [
        ASSOCIATED_TOKEN_PROGRAM_ID,
        Pubkey.default(),
        TOKEN_PROGRAM_ID,
    ]
    transfer_params = decode_transfer(compiled_instruction(transaction, 1))
    assert transfer_params["lamports"] == 250_000_001
    assert transfer_params["from_pubkey"] == wallet_key
    assert transfer_params["to_pubkey"] == get_associated_token_address(
        wallet_key, WRAPPED_SOL_MINT
    )
    sync_params = decode_sync_native(compiled_instruction(transaction, 2))
    assert sync_params.program_id == TOKEN_PROGRAM_ID
    assert state.native_sol_balance == "2.000000000"
    assert state.balances[WSOL_MINT].idle == "0.500000000"


@pytest.mark.asyncio
async def test_full_unwrap_closes_canonical_wsol_ata_and_confirms() -> None:
    wallet_key = Keypair().pubkey()
    wallet = str(wallet_key)
    client = FakeConversionClient(
        AuthCredentials(
            user_id="user-a",
            wallet_address=wallet,
            expires_at=int(time.time()) + 60,
        )
    )

    signature = await Positions(client).unwrap_wsol(  # type: ignore[arg-type]
        initialized_state(wallet)
    )

    assert signature == "confirmed-signature"
    assert len(client.transactions) == 1
    close_params = decode_close_account(compiled_instruction(client.transactions[0], 0))
    assert close_params.program_id == TOKEN_PROGRAM_ID
    assert close_params.account == get_associated_token_address(
        wallet_key, WRAPPED_SOL_MINT
    )
    assert close_params.owner == wallet_key
    assert close_params.dest == wallet_key


@pytest.mark.asyncio
@pytest.mark.parametrize("wsol_idle", [None, "0.000000000"])
async def test_unwrap_requires_positive_cached_wsol_before_signing(
    wsol_idle: str | None,
) -> None:
    wallet = str(Keypair().pubkey())
    client = FakeConversionClient(
        AuthCredentials(
            user_id="user-a",
            wallet_address=wallet,
            expires_at=int(time.time()) + 60,
        )
    )
    state = initialized_state(wallet)
    if wsol_idle is None:
        del state.balances[WSOL_MINT]
    else:
        state.balances[WSOL_MINT] = balance(WSOL_MINT, wsol_idle)

    with pytest.raises(SdkError, match="must be greater than zero"):
        await Positions(client).unwrap_wsol(state)  # type: ignore[arg-type]
    assert client.transactions == []


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "amount",
    [-1.0, "-1", "0", "0.0000000001", "3.000000000", "18446744073.709551616"],
)
async def test_wrap_rejects_invalid_amounts_before_signing(amount: object) -> None:
    wallet = str(Keypair().pubkey())
    client = FakeConversionClient(
        AuthCredentials(
            user_id="user-a",
            wallet_address=wallet,
            expires_at=int(time.time()) + 60,
        )
    )

    with pytest.raises(SdkError):
        await Positions(client).wrap_sol(  # type: ignore[arg-type]
            amount, initialized_state(wallet)
        )
    assert client.transactions == []


@pytest.mark.asyncio
async def test_conversion_requires_unexpired_matching_credentials() -> None:
    wallet = str(Keypair().pubkey())
    other_wallet = str(Keypair().pubkey())
    cases = [
        None,
        AuthCredentials(
            user_id="user-a",
            wallet_address=wallet,
            expires_at=int(time.time()) - 1,
        ),
        AuthCredentials(
            user_id="user-a",
            wallet_address=other_wallet,
            expires_at=int(time.time()) + 60,
        ),
    ]
    for credentials in cases:
        client = FakeConversionClient(credentials)
        with pytest.raises(SdkError):
            await Positions(client).wrap_sol(  # type: ignore[arg-type]
                "0.100000000", initialized_state(wallet)
            )
        assert client.transactions == []


@pytest.mark.asyncio
async def test_submission_failure_propagates_without_mutating_state() -> None:
    wallet = str(Keypair().pubkey())
    client = FakeConversionClient(
        AuthCredentials(
            user_id="user-a",
            wallet_address=wallet,
            expires_at=int(time.time()) + 60,
        ),
        failure=RuntimeError("submission failed"),
    )
    state = initialized_state(wallet)
    before = state.combined_sol_balance()

    with pytest.raises(RuntimeError, match="submission failed"):
        await Positions(client).wrap_sol("0.100000000", state)  # type: ignore[arg-type]
    assert state.combined_sol_balance() == before
