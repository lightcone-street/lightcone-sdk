"""V1 submission retains exact fee authority, full signatures, and original expiry."""

from types import SimpleNamespace
from unittest.mock import AsyncMock

import pytest
from solders.account import Account
from solders.hash import Hash
from solders.instruction import AccountMeta, Instruction
from solders.keypair import Keypair
from solders.message.v1 import Message
from solders.pubkey import Pubkey
from solders.transaction import Transaction, VersionedTransaction

from lightcone_sdk import (
    InsufficientSolForTransactionFees,
    LightconeClientBuilder,
    SdkError,
    SubmissionUnknown,
    V1ResourceConfig,
    V1Transaction,
    V1TransactionContext,
)
from lightcone_sdk.shared.signing import ExternalSigner, SigningStrategy

CONTEXT = V1TransactionContext(
    Hash.from_bytes(bytes([7] * 32)), 123, V1ResourceConfig(200_000, 1_048_576, 0)
)


class Connection:
    def __init__(self):
        self.sent = []
        self.simulated = []
        self.fee = 5000
        self.balance = 1_000_000
        self.feature = Account(
            1,
            b"\x01" + (50).to_bytes(8, "little"),
            Pubkey.from_string("Feature111111111111111111111111111111111111"),
        )
        self.send_error = None
        self.response_signature = None
        self.simulation_error = None

    async def get_fee_for_message(self, message, commitment):
        self.fee_message = message
        if isinstance(self.fee, Exception):
            raise self.fee
        return SimpleNamespace(value=self.fee)

    async def get_balance(self, payer, commitment):
        return SimpleNamespace(value=self.balance)

    async def get_latest_blockhash(self, commitment):
        return SimpleNamespace(
            value=SimpleNamespace(
                blockhash=CONTEXT.blockhash,
                last_valid_block_height=CONTEXT.last_valid_block_height,
            )
        )

    async def get_account_info(self, address, commitment):
        return SimpleNamespace(value=self.feature, context=SimpleNamespace(slot=100))

    async def simulate_transaction(self, transaction, **options):
        self.simulated.append((bytes(transaction), options))
        return SimpleNamespace(
            context=SimpleNamespace(slot=101),
            value=SimpleNamespace(
                err=self.simulation_error,
                units_consumed=1000,
                loaded_accounts_data_size=256,
                logs=["ok"],
            ),
        )

    async def send_raw_transaction(self, wire, opts):
        self.sent.append((wire, opts))
        if self.send_error:
            raise self.send_error
        return SimpleNamespace(
            value=self.response_signature
            or VersionedTransaction.from_bytes(wire).signatures[0]
        )


class Wallet(ExternalSigner):
    def __init__(self, keypair, mutate=False, unsigned=False):
        self.keypair = keypair
        self.wallet_address = str(keypair.pubkey())
        self.calls = 0
        self.mutate = mutate
        self.unsigned = unsigned

    async def sign_message(self, message):
        return bytes(self.keypair.sign_message(message))

    async def sign_transaction(self, wire):
        self.calls += 1
        if self.unsigned:
            return wire
        tx = VersionedTransaction.from_bytes(wire)
        message = tx.message
        if self.mutate:
            message = Message(
                message.header,
                message.config,
                Hash.new_unique(),
                message.account_keys,
                message.instructions,
            )
        return bytes(VersionedTransaction(message, [self.keypair]))


def setup(wallet=False):
    keypair, connection = Keypair(), Connection()
    builder = (
        LightconeClientBuilder()
        .rpc_connection(connection)
        .transaction_resources(CONTEXT.resources)
    )
    signer = Wallet(keypair) if wallet else None
    builder = (
        builder.external_signer(signer) if wallet else builder.native_signer(keypair)
    )
    client = builder.build()
    tx = V1Transaction.compile(
        [Instruction(Pubkey.new_unique(), b"test", [])], keypair.pubkey(), CONTEXT
    )
    return client, connection, keypair, signer, tx


@pytest.mark.parametrize("external", [False, True])
async def test_exact_signed_simulation_single_send_and_immutable_input(external):
    client, connection, _, signer, tx = setup(external)
    original = bytes(tx)
    signature, height = await client._sign_and_submit_tx_inner(tx)
    assert height == CONTEXT.last_valid_block_height
    assert bytes(tx) == original
    assert len(connection.sent) == 1
    wire, options = connection.sent[0]
    assert signature == str(VersionedTransaction.from_bytes(wire).signatures[0])
    assert connection.simulated[0][0] == wire
    assert connection.simulated[0][1] == {
        "sig_verify": True,
        "commitment": "confirmed",
        "replace_recent_blockhash": False,
    }
    assert (
        options.max_retries == 0
        and options.skip_preflight is False
        and options.skip_confirmation
    )
    assert connection.fee_message == tx.message
    assert signer is None or signer.calls == 1
    await client.close()


@pytest.mark.parametrize("prepared", [False, True])
async def test_confirmation_preserves_original_expiry(prepared):
    client, _, _, _, tx = setup()
    client.rpc().confirm_signature_status = AsyncMock(
        return_value=SimpleNamespace(slot=999)
    )
    method = (
        client.sign_and_submit_prepared_tx_confirmed_with_slot
        if prepared
        else client.sign_and_submit_tx_confirmed_with_slot
    )
    result = await method(tx)
    client.rpc().confirm_signature_status.assert_awaited_once_with(
        result.signature, 123
    )
    assert result.slot == 999
    await client.close()


async def test_feature_failure_precedes_wallet_prompt_and_send():
    client, connection, _, signer, tx = setup(True)
    connection.feature = None
    with pytest.raises(SdkError, match="feature"):
        await client.sign_and_submit_tx(tx)
    assert signer.calls == 0 and not connection.sent
    await client.close()


@pytest.mark.parametrize(
    "data,owner,executable",
    [
        (b"\0" * 9, "Feature111111111111111111111111111111111111", False),
        (
            b"\1" + (101).to_bytes(8, "little"),
            "Feature111111111111111111111111111111111111",
            False,
        ),
        (b"\1", "Feature111111111111111111111111111111111111", False),
        (b"\1" + bytes(8), "11111111111111111111111111111111", False),
        (b"\1" + bytes(8), "Feature111111111111111111111111111111111111", True),
    ],
)
async def test_feature_account_validation(data, owner, executable):
    client, conn, _, _, tx = setup()
    conn.feature = Account(1, data, Pubkey.from_string(owner), executable)
    with pytest.raises(SdkError, match="feature"):
        await client.sign_and_submit_tx(tx)
    assert not conn.sent
    await client.close()


@pytest.mark.parametrize("mutate,unsigned", [(True, False), (False, True)])
async def test_wallet_rejects_changed_or_unsigned_bytes_before_submission(
    mutate, unsigned
):
    client, connection, _, signer, tx = setup(True)
    signer.mutate, signer.unsigned = mutate, unsigned
    with pytest.raises(SdkError):
        await client.sign_and_submit_tx(tx)
    assert not connection.sent and not connection.simulated
    await client.close()


async def test_fee_shortfall_rejects_before_signer_unknown_fees_continue():
    client, connection, _, signer, tx = setup(True)
    connection.balance = 4999
    with pytest.raises(InsufficientSolForTransactionFees) as error:
        await client.sign_and_submit_tx(tx)
    assert (error.value.available_lamports, error.value.required_lamports) == (
        4999,
        5000,
    )
    assert not connection.sent and signer.calls == 0
    connection.fee = ValueError("fee unavailable")
    await client.sign_and_submit_tx(tx)
    assert len(connection.sent) == 1
    await client.close()


@pytest.mark.parametrize("kind", ["transport", "signature"])
async def test_ambiguous_send_retains_signature_expiry_without_retry(kind):
    client, conn, payer, _, tx = setup()
    if kind == "transport":
        conn.send_error = ConnectionError("connection lost after send")
    else:
        conn.response_signature = "wrong"
    with pytest.raises(SubmissionUnknown) as error:
        await client.sign_and_submit_tx(tx)
    assert len(conn.sent) == 1
    assert error.value.signature == str(tx.sign([payer]).signatures[0])
    assert error.value.last_valid_block_height == 123
    await client.close()


async def test_simulation_failure_and_unsigned_direct_send_fail_before_send():
    client, conn, payer, _, tx = setup()
    with pytest.raises(SdkError):
        await client.rpc().send_raw_transaction(tx)
    conn.simulation_error = "program failed"
    with pytest.raises(SdkError, match="simulation failed"):
        await client.rpc().submit_signed_transaction(tx.sign([payer]))
    assert not conn.sent
    await client.close()


async def test_context_requires_explicit_resources_and_rejects_legacy_and_privy():
    client, conn, _, _, tx = setup()
    assert await client.transaction_context() == CONTEXT
    client._transaction_resources = None
    with pytest.raises(SdkError, match="resources are required"):
        await client.transaction_context()
    with pytest.raises(SdkError, match="v1"):
        await client.sign_and_submit_tx(Transaction.default())
    client.signing_strategy = SigningStrategy.privy("id")
    with pytest.raises(SdkError, match="Privy"):
        await client.sign_and_submit_tx(tx)
    with pytest.raises(SdkError, match="Privy"):
        await client.privy().sign_and_send_tx("id", "AA==")
    assert not conn.sent
    await client.close()


async def test_sponsorship_keeps_payer_rules():
    client, conn, payer, _, tx = setup()
    client.transaction_sponsorship_enabled = True
    with pytest.raises(SdkError, match="sponsorship"):
        await client.sign_and_submit_tx(tx)
    client.transaction_sponsorship_enabled = False
    client.signing_strategy = SigningStrategy.native(Keypair())
    with pytest.raises(SdkError, match="fee payer"):
        await client.sign_and_submit_tx(tx)
    assert not conn.sent
    await client.close()


async def test_real_solana_rpc_requests_encode_canonical_v1_bytes():
    """Exercise maintained binding request serializers, not only fake RPC methods."""
    import base64
    import json

    from solana.rpc.async_api import AsyncClient

    client, _, payer, _, tx = setup()
    rpc = AsyncClient("http://localhost:8899")
    rpc._provider.make_request = AsyncMock(return_value=SimpleNamespace(value=5000))
    await rpc.get_fee_for_message(tx.message, "confirmed")
    request = rpc._provider.make_request.call_args.args[0]
    assert (
        json.loads(request.to_json())["params"][0]
        == base64.b64encode(tx.message_bytes()).decode()
    )
    signed = tx.sign([payer])
    await rpc.simulate_transaction(
        signed.as_versioned(),
        sig_verify=True,
        replace_recent_blockhash=False,
        commitment="confirmed",
    )
    request = json.loads(rpc._provider.make_request.call_args.args[0].to_json())
    assert request["params"][0] == base64.b64encode(bytes(signed)).decode()
    assert request["params"][1]["sigVerify"] is True
    assert request["params"][1]["replaceRecentBlockhash"] is False
    await rpc.close()
    await client.close()


async def test_sponsored_external_submission_requires_every_signature():
    client, conn, wallet_key, signer, _ = setup(True)
    sponsor = Keypair()
    tx = V1Transaction.compile(
        [
            Instruction(
                Pubkey.new_unique(),
                b"test",
                [AccountMeta(wallet_key.pubkey(), True, False)],
            )
        ],
        sponsor.pubkey(),
        CONTEXT,
    )
    client.set_transaction_sponsorship_enabled(True)
    conn.fee = 1_000_000
    conn.balance = 0

    async def sign_all(wire):
        return bytes(
            VersionedTransaction(
                VersionedTransaction.from_bytes(wire).message, [sponsor, wallet_key]
            )
        )

    signer.sign_transaction = sign_all
    await client.sign_and_submit_tx(tx)
    assert len(conn.sent) == 1
    assert not hasattr(conn, "fee_message")
    await client.close()


async def test_submission_snapshots_strategy_before_awaiting_fee_authority():
    client, conn, payer, _, tx = setup()
    original_fee = conn.get_fee_for_message

    async def change_strategy(message, commitment):
        client.signing_strategy.keypair = Keypair()
        client.set_transaction_sponsorship_enabled(True)
        return await original_fee(message, commitment)

    conn.get_fee_for_message = change_strategy
    signature = await client.sign_and_submit_tx(tx)
    assert signature == str(tx.sign([payer]).signatures[0])
    await client.close()


async def test_unsponsored_external_signer_requires_known_wallet_identity():
    client, conn, _, signer, tx = setup(True)
    signer.wallet_address = None
    with pytest.raises(SdkError, match="identity"):
        await client.sign_and_submit_tx(tx)
    assert signer.calls == 0 and not conn.sent
    await client.close()


@pytest.mark.parametrize("failure", ["http429", "transport", "rpc_error"])
async def test_injected_solana_connection_cannot_retry_uncertain_send(failure):
    import httpx2
    from solana.rpc.async_api import AsyncClient

    client, _, payer, _, tx = setup()
    connection = AsyncClient("http://localhost:8899", max_transport_retries=3)
    await connection._provider.session.aclose()
    attempts = []

    async def handle(request):
        attempts.append(request)
        if failure == "http429":
            return httpx2.Response(429, request=request)
        if failure == "transport":
            raise httpx2.RemoteProtocolError("response lost")
        return httpx2.Response(
            200,
            json={
                "jsonrpc": "2.0",
                "id": 0,
                "error": {"code": -32002, "message": "simulation rejected"},
            },
            request=request,
        )

    connection._provider.session = httpx2.AsyncClient(
        transport=httpx2.MockTransport(handle)
    )
    client._primary_connection = connection
    client.rpc().simulate_transaction = AsyncMock()
    with pytest.raises(SubmissionUnknown) as error:
        await client.rpc().submit_signed_transaction(tx.sign([payer]))
    assert len(attempts) == 1
    assert error.value.signature == str(tx.sign([payer]).signatures[0])
    assert error.value.last_valid_block_height == CONTEXT.last_valid_block_height
    await connection.close()
    await client.close()
