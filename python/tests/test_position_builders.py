import pytest
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk.client import LightconeClient
from lightcone_sdk.error import SdkError
from lightcone_sdk.http import LightconeHttp
from lightcone_sdk.program.errors import InvalidOutcomeIndexError


def builder(client: LightconeClient):
    return (
        client.positions()
        .withdraw_from_position()
        .user(Keypair().pubkey())
        .market(Pubkey.new_unique())
        .deposit_mint(Pubkey.new_unique())
        .amount(1)
        .outcome_index(2)
    )


def test_withdraw_from_position_requires_num_outcomes() -> None:
    client = LightconeClient(LightconeHttp("https://example.com"))

    with pytest.raises(SdkError, match="num_outcomes is required"):
        builder(client).build_ix()


def test_withdraw_from_position_validates_against_num_outcomes() -> None:
    client = LightconeClient(LightconeHttp("https://example.com"))

    with pytest.raises(InvalidOutcomeIndexError):
        builder(client).num_outcomes(2).build_ix()


def test_v1_builders_preserve_encoded_accounts_amount_and_context() -> None:
    from solders.hash import Hash

    from lightcone_sdk import V1ResourceConfig, V1Transaction, V1TransactionContext
    from lightcone_sdk.program.types import DepositToGlobalParams

    client = LightconeClient(LightconeHttp("https://example.com"))
    payer = Keypair().pubkey()
    mint = Pubkey.new_unique()
    context = V1TransactionContext(
        Hash.new_unique(), 456, V1ResourceConfig(200_000, 1_048_576, 9)
    )
    amount = 2**53 + 1
    fluent = (
        client.positions().deposit_to_global().user(payer).mint(mint).amount(amount)
    )
    from spl.token.instructions import get_associated_token_address

    from lightcone_sdk.program.constants import (
        INSTRUCTION_DEPOSIT_TO_GLOBAL,
        SYSTEM_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    )

    program = client.program_id
    global_deposit = Pubkey.find_program_address(
        [b"global_deposit", bytes(mint)], program
    )[0]
    custody = Pubkey.find_program_address(
        [b"global_deposit", bytes(payer), bytes(mint)], program
    )[0]
    exchange = Pubkey.find_program_address([b"central_state"], program)[0]
    event_authority = Pubkey.find_program_address([b"__event_authority"], program)[0]
    expected_accounts = [
        payer,
        global_deposit,
        mint,
        custody,
        get_associated_token_address(payer, mint),
        TOKEN_PROGRAM_ID,
        SYSTEM_PROGRAM_ID,
        exchange,
        event_authority,
        program,
    ]
    built = fluent.build_tx(context)
    direct = client.positions().deposit_to_global_tx(
        DepositToGlobalParams(payer, mint, amount), context
    )
    assert built == direct
    assert isinstance(built, V1Transaction)
    assert built.context == context
    compiled = built.message.instructions[0]
    assert len(compiled.data) == 9
    assert compiled.data[0] == INSTRUCTION_DEPOSIT_TO_GLOBAL
    assert int.from_bytes(compiled.data[1:9], "little") == amount
    assert [
        built.message.account_keys[index] for index in compiled.accounts
    ] == expected_accounts
    assert built.message.account_keys[compiled.program_id_index] == program
    for index, key in enumerate(built.message.account_keys):
        assert built.message.is_signer(index) == (key == payer)
        assert built.message.is_maybe_writable(index) == (
            key in (payer, custody, expected_accounts[4])
        )
    assert built.required_signers == (payer,)
    assert client.orders().increment_nonce_tx(payer, context).context == context


@pytest.mark.asyncio
async def test_fluent_submission_obtains_configured_context() -> None:
    from unittest.mock import AsyncMock

    from solders.hash import Hash

    from lightcone_sdk import V1ResourceConfig, V1TransactionContext

    client = LightconeClient(LightconeHttp("https://example.com"))
    context = V1TransactionContext(
        Hash.new_unique(), 789, V1ResourceConfig(200_000, 1_048_576, 0)
    )
    client.transaction_context = AsyncMock(return_value=context)
    client.sign_and_submit_tx = AsyncMock(return_value="signature")
    fluent = (
        client.positions()
        .deposit_to_global()
        .user(Keypair().pubkey())
        .mint(Pubkey.new_unique())
        .amount(42)
    )
    assert await fluent.sign_and_submit() == "signature"
    client.transaction_context.assert_awaited_once()
    submitted = client.sign_and_submit_tx.call_args.args[0]
    assert submitted == fluent.build_tx(context)


@pytest.mark.parametrize("missing", ["user", "mint", "amount"])
def test_global_deposit_requires_each_input(missing):
    client = LightconeClient(LightconeHttp("https://example.com"))
    fluent = client.positions().deposit_to_global()
    for name, value in {
        "user": Keypair().pubkey(),
        "mint": Pubkey.new_unique(),
        "amount": 42,
    }.items():
        if name != missing:
            getattr(fluent, name)(value)
    with pytest.raises(SdkError, match=f"{missing} is required"):
        fluent.build_ix()


def test_global_deposit_rejects_zero_amount():
    from lightcone_sdk.program.errors import ZeroAmountError

    client = LightconeClient(LightconeHttp("https://example.com"))
    with pytest.raises(ZeroAmountError):
        client.positions().deposit_to_global().user(Keypair().pubkey()).mint(
            Pubkey.new_unique()
        ).amount(0).build_ix()
