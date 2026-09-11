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
    instruction = fluent.build_ix()
    built = fluent.build_tx(context)
    direct = client.positions().deposit_to_global_tx(
        DepositToGlobalParams(payer, mint, amount), context
    )
    assert built == direct
    assert isinstance(built, V1Transaction)
    assert built.context == context
    compiled = built.message.instructions[0]
    assert compiled.data == instruction.data
    assert [built.message.account_keys[index] for index in compiled.accounts] == [
        meta.pubkey for meta in instruction.accounts
    ]
    assert (
        built.message.account_keys[compiled.program_id_index] == instruction.program_id
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
