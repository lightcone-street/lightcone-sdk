from types import SimpleNamespace
from unittest.mock import AsyncMock

import pytest
from solders.hash import Hash
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk import V1ResourceConfig, V1Transaction, V1TransactionContext
from lightcone_sdk.client import LightconeClient
from lightcone_sdk.error import SdkError
from lightcone_sdk.http import LightconeHttp
from lightcone_sdk.program.errors import InvalidOutcomeIndexError
from lightcone_sdk.shared.signing import SigningStrategy


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


_FLUENT_BUILDERS = (
    "deposit",
    "withdraw",
    "merge",
    "redeem_winnings",
    "withdraw_from_position",
    "init_position_tokens",
    "deposit_to_global",
    "withdraw_from_global",
    "global_to_market_deposit",
)


def _fluent_submission_client():
    """Keep real compilation/signing while observing only asynchronous RPC edges."""
    resources = V1ResourceConfig(200_000, 1_048_576, 0)
    client = LightconeClient(
        LightconeHttp("https://example.com"), transaction_resources=resources
    )
    context = V1TransactionContext(Hash.new_unique(), 789, resources)
    client.transaction_context = AsyncMock(wraps=client.transaction_context)
    rpc = client.rpc()
    rpc.get_latest_blockhash_with_height = AsyncMock(
        return_value=(context.blockhash, context.last_valid_block_height)
    )
    rpc.estimate_prepared_transaction_fee = AsyncMock(return_value=5_000)
    rpc.balance_lamports = AsyncMock(return_value=1_000_000)
    rpc.ensure_v1_supported = AsyncMock()
    rpc.submit_signed_transaction = AsyncMock(return_value="signature")
    return client, context


def _configured_fluent(client, name, payer):
    """Supply every builder's real local inputs, including a distinct init user."""
    fluent = getattr(client.positions(), name)()
    mint = Pubkey.new_unique()
    market = Pubkey.new_unique()
    if name == "init_position_tokens":
        return (
            fluent.payer(payer)
            .user(Keypair().pubkey())
            .market(market)
            .deposit_mints([mint])
            .num_outcomes(2)
        )
    fluent.user(payer).amount(42)
    if name == "withdraw_from_position":
        return fluent.market(market).deposit_mint(mint).outcome_index(0).num_outcomes(2)
    fluent.mint(mint)
    if name == "merge":
        fluent.market(SimpleNamespace(pubkey=str(market), num_outcomes=2))
    elif name == "redeem_winnings":
        fluent.market(market).outcome_index(0)
    elif name == "global_to_market_deposit":
        fluent.market(market).num_outcomes(2)
    return fluent


@pytest.mark.asyncio
@pytest.mark.parametrize("name", _FLUENT_BUILDERS)
async def test_fluent_missing_fields_fail_before_context_rpc(name):
    client, _context = _fluent_submission_client()
    fluent = getattr(client.positions(), name)()
    with pytest.raises(SdkError, match="is required"):
        await fluent.sign_and_submit()
    client.transaction_context.assert_not_awaited()
    client.rpc().get_latest_blockhash_with_height.assert_not_awaited()
    client.rpc().submit_signed_transaction.assert_not_awaited()


@pytest.mark.asyncio
@pytest.mark.parametrize("missing", ["mint", "amount"])
async def test_fluent_validates_remaining_fields_before_signer_and_context(missing):
    client, _context = _fluent_submission_client()
    fluent = client.positions().deposit_to_global().user(Keypair().pubkey())
    if missing == "mint":
        fluent.amount(42)
    else:
        fluent.mint(Pubkey.new_unique())
    with pytest.raises(SdkError, match=f"{missing} is required"):
        await fluent.sign_and_submit()
    client.transaction_context.assert_not_awaited()
    client.rpc().get_latest_blockhash_with_height.assert_not_awaited()


@pytest.mark.asyncio
@pytest.mark.parametrize("name", _FLUENT_BUILDERS)
@pytest.mark.parametrize(
    "failure,expected",
    [
        ("absent", "signing strategy is not set"),
        ("privy", "Privy"),
        ("wrong_wallet", "fee payer"),
        ("native_sponsorship", "sponsorship"),
        ("missing_identity", "wallet identity"),
    ],
)
async def test_fluent_signing_validation_precedes_context_rpc(name, failure, expected):
    client, _context = _fluent_submission_client()
    payer = Keypair()
    fluent = _configured_fluent(client, name, payer.pubkey())
    if failure == "privy":
        client.set_signing_strategy(
            SigningStrategy.privy("wallet", str(payer.pubkey()))
        )
    elif failure == "wrong_wallet":
        client.set_signing_strategy(SigningStrategy.native(Keypair()))
    elif failure == "native_sponsorship":
        client.set_signing_strategy(SigningStrategy.native(payer))
        client.set_transaction_sponsorship_enabled(True)
    elif failure == "missing_identity":
        client.set_signing_strategy(
            SigningStrategy.wallet_adapter(SimpleNamespace(wallet_address=None))
        )
    with pytest.raises(SdkError, match=expected):
        await fluent.sign_and_submit()
    client.transaction_context.assert_not_awaited()
    client.rpc().get_latest_blockhash_with_height.assert_not_awaited()
    client.rpc().estimate_prepared_transaction_fee.assert_not_awaited()
    client.rpc().submit_signed_transaction.assert_not_awaited()


@pytest.mark.asyncio
@pytest.mark.parametrize("name", _FLUENT_BUILDERS)
async def test_fluent_captures_inputs_and_signing_context_before_blockhash_await(name):
    client, context = _fluent_submission_client()
    payer = Keypair()
    strategy = SigningStrategy.native(payer)
    client.set_signing_strategy(strategy)
    fluent = _configured_fluent(client, name, payer.pubkey())
    instruction = fluent.build_ix()
    expected_message = V1Transaction.compile(
        [instruction], payer.pubkey(), context
    ).message_bytes()

    async def fetch_blockhash():
        # Mutate the original strategy as well as replacing the client-wide setting.
        strategy.keypair = Keypair()
        client.set_signing_strategy(SigningStrategy.privy("replacement"))
        client.set_transaction_sponsorship_enabled(True)
        fluent.user(Keypair().pubkey())
        if name == "init_position_tokens":
            fluent.payer(Keypair().pubkey())
        else:
            fluent.amount(900)
        return context.blockhash, context.last_valid_block_height

    client.rpc().get_latest_blockhash_with_height.side_effect = fetch_blockhash
    assert await fluent.sign_and_submit() == "signature"
    client.transaction_context.assert_awaited_once()
    client.rpc().get_latest_blockhash_with_height.assert_awaited_once()
    client.rpc().estimate_prepared_transaction_fee.assert_awaited_once()
    client.rpc().balance_lamports.assert_awaited_once_with(payer.pubkey())
    client.rpc().submit_signed_transaction.assert_awaited_once()
    submitted = client.rpc().submit_signed_transaction.call_args.args[0]
    assert submitted.message_bytes() == expected_message
    assert submitted.context == context
    submitted.verify_signatures()
