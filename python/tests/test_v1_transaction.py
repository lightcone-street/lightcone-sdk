"""Canonical Solana v1 boundaries and immutable signing."""

from dataclasses import FrozenInstanceError, replace

import pytest
from solders.hash import Hash
from solders.instruction import AccountMeta, Instruction
from solders.keypair import Keypair
from solders.message import Message as LegacyMessage
from solders.message import MessageV0
from solders.message.v1 import Message, TransactionConfig
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


def test_imports_explicit_zero_fee_without_rewriting_signatures():
    payer = Keypair()
    context = replace(CONTEXT, resources=replace(RESOURCES, priority_fee_lamports=0))
    omitted = V1Transaction.compile(
        [Instruction(Pubkey.new_unique(), b"test", [])], payer.pubkey(), context
    )
    original = omitted.message
    message = Message(
        original.header,
        TransactionConfig(
            priority_fee=0,
            compute_unit_limit=context.resources.compute_unit_limit,
            loaded_accounts_data_size_limit=context.resources.loaded_accounts_data_size_limit,
            heap_size=context.resources.heap_size,
        ),
        original.lifetime_specifier,
        original.account_keys,
        original.instructions,
    )
    wire = bytes(VersionedTransaction(message, [payer]))
    imported = V1Transaction.from_wire_bytes(wire, context)
    imported.verify_signatures()
    assert imported.message.config.priority_fee == 0
    assert bytes(imported) == wire
    assert imported.accept_signed_bytes(wire) == imported
    with pytest.raises(SdkError, match="wallet changed"):
        omitted.accept_signed_bytes(wire)
    with pytest.raises(SdkError, match="context"):
        V1Transaction.from_wire_bytes(
            wire,
            replace(
                context, resources=replace(context.resources, priority_fee_lamports=1)
            ),
        )


@pytest.mark.parametrize("zero_signers", [True, False])
def test_upstream_rejects_zero_signers_and_payer_as_program(zero_signers):
    from solders.instruction import CompiledInstruction
    from solders.message import MessageHeader

    tx = transaction()
    original = tx.message
    header = original.header
    instructions = original.instructions
    if zero_signers:
        header = MessageHeader(0, 0, header.num_readonly_unsigned_accounts)
    else:
        instruction = instructions[0]
        instructions[0] = CompiledInstruction(0, instruction.data, instruction.accounts)
    message = Message(
        header,
        original.config,
        original.lifetime_specifier,
        original.account_keys,
        instructions,
    )
    invalid = VersionedTransaction.populate(
        message, [] if zero_signers else tx.signatures
    )
    with pytest.raises(SdkError):
        V1Transaction.from_versioned(invalid, CONTEXT)


def test_cross_language_compilation_and_signing_match_pinned_digests():
    from hashlib import sha256

    from solders.system_program import TransferParams, transfer

    # Shared with Rust/TypeScript: pin upstream Rust bytes without fixture files.
    payer, second = Keypair.from_seed(bytes([1] * 32)), Keypair.from_seed(
        bytes([2] * 32)
    )

    def key(byte):
        return Pubkey.from_bytes(bytes([byte] * 32))

    accounts = [
        AccountMeta(second.pubkey(), True, False),
        AccountMeta(key(10), False, False),
        AccountMeta(payer.pubkey(), True, True),
        AccountMeta(second.pubkey(), True, False),
        AccountMeta(key(8), False, True),
    ]
    instruction = Instruction(key(99), bytes([0, 1, 2, 127, 128, 255]), accounts)
    merged = Instruction(
        key(99),
        instruction.data,
        accounts
        + [AccountMeta(key(10), False, True), AccountMeta(second.pubkey(), True, True)],
    )
    cases = [
        (
            V1ResourceConfig(1_400_000, 67_108_864, 2**64 - 1, 262_144),
            [instruction],
            [payer, second],
            "c5fdf05cc77574469ea3b7b08798722fda2bd225b1b8e35995d42f31e7011290",
            "40156bfa4370bd072ca9000a3c42d1159be9c6c8b32cf598793a276b16f332c3",
        ),
        (
            V1ResourceConfig(200_000, 1_048_576, 1001, 32_768),
            [
                instruction,
                merged,
                transfer(
                    TransferParams(
                        from_pubkey=payer.pubkey(), to_pubkey=key(9), lamports=42
                    )
                ),
            ],
            [payer, second],
            "fdf9fd85a3963251ca654526c1fb6134f4e76ad361aa2b057bb3dfff379f41c4",
            "368fb841224f6273bfc741ac09d88df014050640e8d3f4b31a38c05b1ef04809",
        ),
        (
            V1ResourceConfig(200_000, 1_048_576, 0),
            [
                Instruction(
                    key(99),
                    bytes([1, 2]),
                    [
                        AccountMeta(key(99), False, True),
                        AccountMeta(key(98), False, True),
                    ],
                ),
                Instruction(
                    key(98),
                    bytes([3, 4]),
                    [
                        AccountMeta(key(99), False, False),
                        AccountMeta(payer.pubkey(), True, True),
                    ],
                ),
            ],
            [payer],
            "68e412b10a0ac0b912f9606cf2b62bc158e74ee25a2f5a7a5c20117584cfd7c8",
            "e7629ba9b05db1fd502a408343395bb001aa0fdb48d659c59c1b06983f924307",
        ),
    ]
    for resources, instructions, signers, message_hash, wire_hash in cases:
        context = replace(CONTEXT, resources=resources)
        tx = V1Transaction.compile(instructions, payer.pubkey(), context)
        signed = tx.sign(signers)
        assert sha256(tx.message_bytes()).hexdigest() == message_hash
        assert sha256(bytes(signed)).hexdigest() == wire_hash
        signed.verify_signatures()
