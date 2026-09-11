"""Canonical Solana v1 boundaries and shared Rust wire fixtures."""

import base64
import json
from dataclasses import FrozenInstanceError, replace
from pathlib import Path

import pytest
from solders.hash import Hash
from solders.instruction import AccountMeta, Instruction
from solders.keypair import Keypair
from solders.message import Message as LegacyMessage
from solders.message import MessageV0
from solders.message.v1 import Message
from solders.pubkey import Pubkey
from solders.transaction import Transaction, VersionedTransaction

from lightcone_sdk import (
    SdkError,
    V1ResourceConfig,
    V1Transaction,
    V1TransactionContext,
)

RESOURCES = V1ResourceConfig(200_000, 1_048_576, 123)
CONTEXT = V1TransactionContext(Hash.from_bytes(bytes([7] * 32)), 123, RESOURCES)


def transaction(payer=None, data=b"test", accounts=()):
    payer = payer or Keypair()
    return V1Transaction.compile(
        [Instruction(Pubkey.from_bytes(bytes([9] * 32)), data, list(accounts))],
        payer.pubkey(),
        CONTEXT,
    )


def test_canonical_signatures_and_immutability():
    payer, cosigner = Keypair(), Keypair()
    tx = transaction(payer, accounts=[AccountMeta(cosigner.pubkey(), True, False)] * 3)
    signed = tx.sign([cosigner, payer, cosigner])
    assert len(tx.required_signers) == 2
    assert len(bytes(tx)) == len(bytes(signed))
    assert bytes(signed)[0] == 0x81
    assert signed == tx.accept_signed_bytes(bytes(signed))
    assert tx.message.config.priority_fee == 123
    assert len(tx.message.instructions[0].accounts) == 3
    assert len(tx.message.account_keys) == 3
    with pytest.raises(FrozenInstanceError):
        tx.context = CONTEXT
    copy = signed.as_versioned()
    copy.signatures = []
    signed.verify_signatures()
    with pytest.raises(SdkError, match="missing signer"):
        tx.sign([payer])
    with pytest.raises(SdkError):
        tx.accept_signed_bytes(bytes(tx))
    with pytest.raises(SdkError):
        V1Transaction.from_wire_bytes(bytes(signed) + b"\0", CONTEXT)


@pytest.mark.parametrize(
    "field,value",
    [
        ("compute_unit_limit", 0),
        ("compute_unit_limit", 1_400_001),
        ("loaded_accounts_data_size_limit", 0),
        ("loaded_accounts_data_size_limit", 67_108_865),
        ("priority_fee_lamports", -1),
        ("priority_fee_lamports", 2**64),
        ("priority_fee_lamports", True),
        ("priority_fee_lamports", 1.0),
        ("heap_size", 32769),
        ("heap_size", 16384),
    ],
)
def test_invalid_resource_boundaries(field, value):
    with pytest.raises(SdkError):
        replace(RESOURCES, **{field: value})


def test_rejects_legacy_v0_empty_and_compute_budget():
    payer = Keypair()
    ix = Instruction(Pubkey.new_unique(), b"", [])
    for message in [
        LegacyMessage.new_with_blockhash([ix], payer.pubkey(), CONTEXT.blockhash),
        MessageV0.try_compile(payer.pubkey(), [ix], [], CONTEXT.blockhash),
    ]:
        legacy = VersionedTransaction(message, [payer])
        with pytest.raises(SdkError, match="only Solana v1"):
            V1Transaction.from_versioned(legacy, CONTEXT)
        with pytest.raises(SdkError, match="only Solana v1"):
            V1Transaction.from_wire_bytes(bytes(legacy), CONTEXT)
    with pytest.raises(SdkError):
        V1Transaction.from_versioned(Transaction.default(), CONTEXT)
    with pytest.raises(SdkError):
        V1Transaction.compile([], payer.pubkey(), CONTEXT)
    with pytest.raises(SdkError, match="ComputeBudget"):
        V1Transaction.compile(
            [
                Instruction(
                    Pubkey.from_string("ComputeBudget111111111111111111111111111111"),
                    b"",
                    [],
                )
            ],
            payer.pubkey(),
            CONTEXT,
        )


def test_size_and_address_limits_include_all_signatures():
    payer, other = Keypair(), Keypair()
    accounts = [AccountMeta(other.pubkey(), True, False)]
    overhead = len(bytes(transaction(payer, b"", accounts)))
    assert len(bytes(transaction(payer, bytes(4096 - overhead), accounts))) == 4096
    with pytest.raises(SdkError):
        transaction(payer, bytes(4097 - overhead), accounts)
    accounts = [AccountMeta(Pubkey.new_unique(), False, False) for _ in range(62)]
    assert len(transaction(payer, accounts=accounts).message.account_keys) == 64
    with pytest.raises(SdkError):
        transaction(
            payer, accounts=accounts + [AccountMeta(Pubkey.new_unique(), False, False)]
        )


@pytest.mark.parametrize("field", ["hash", "config", "data", "accounts"])
def test_wallet_cannot_change_even_correctly_signed_message(field):
    payer = Keypair()
    tx = transaction(payer)
    message = tx.message
    config, blockhash = message.config, message.lifetime_specifier
    keys, instructions = message.account_keys, message.instructions
    if field == "hash":
        blockhash = Hash.new_unique()
    elif field == "config":
        config = replace(RESOURCES, priority_fee_lamports=456)._config()
    elif field == "accounts":
        keys[-1] = Pubkey.new_unique()
    else:
        from solders.instruction import CompiledInstruction

        instructions[0] = CompiledInstruction(
            instructions[0].program_id_index, b"changed", instructions[0].accounts
        )
    changed = VersionedTransaction(
        Message(message.header, config, blockhash, keys, instructions), [payer]
    )
    with pytest.raises(SdkError):
        tx.accept_signed_bytes(bytes(changed))


def test_shared_rust_golden_fixtures():
    fixture = (
        Path(__file__).resolve().parents[2]
        / "rust/src/program/fixtures/solana_v1_transactions.json"
    )
    cases = json.loads(fixture.read_text())["cases"]
    for case in cases:
        raw = case["context"]
        resource = raw["resources"]
        context = V1TransactionContext(
            Hash.from_string(raw["blockhash"]),
            int(raw["last_valid_block_height"]),
            V1ResourceConfig(
                resource["compute_unit_limit"],
                resource["loaded_accounts_data_size_limit"],
                int(resource["priority_fee_lamports"]),
                resource["heap_size"],
            ),
        )
        instructions = [
            Instruction(
                Pubkey.from_string(ix["program_id"]),
                bytes.fromhex(ix["data_hex"]),
                [
                    AccountMeta(
                        Pubkey.from_string(meta["pubkey"]),
                        meta["is_signer"],
                        meta["is_writable"],
                    )
                    for meta in ix["accounts"]
                ],
            )
            for ix in case["instructions"]
        ]
        tx = V1Transaction.compile(
            instructions, Pubkey.from_string(case["payer"]), context
        )
        expected = case["expected"]
        assert tx.message_bytes() == base64.b64decode(expected["message_base64"]), case[
            "name"
        ]
        assert bytes(tx) == base64.b64decode(
            expected["unsigned_transaction_base64"]
        ), case["name"]
        signed = tx.sign(
            [
                Keypair.from_seed(bytes.fromhex(seed))
                for seed in case["signer_seeds_hex"]
            ]
        )
        assert bytes(signed) == base64.b64decode(
            expected["signed_transaction_base64"]
        ), case["name"]
        assert len(bytes(signed)) == expected["wire_size"]
        assert [str(key) for key in tx.required_signers] == expected["required_signers"]
        assert [str(key) for key in tx.message.account_keys] == expected["account_keys"]
        assert [str(sig) for sig in signed.signatures] == expected["signatures"]
