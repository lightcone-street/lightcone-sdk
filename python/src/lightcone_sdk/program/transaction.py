"""Validated, immutable Solana v1 transactions backed by solders' Rust codec."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import ClassVar

from solders.hash import Hash
from solders.instruction import Instruction
from solders.keypair import Keypair
from solders.message import to_bytes_versioned
from solders.message.v1 import V1_PREFIX, Message, TransactionConfig
from solders.pubkey import Pubkey
from solders.signature import Signature
from solders.transaction import VersionedTransaction

from ..error import SdkError

_COMPUTE_BUDGET = Pubkey.from_string("ComputeBudget111111111111111111111111111111")


def _integer(value: int, minimum: int, maximum: int, label: str) -> None:
    """Reject booleans, inexact numbers, and values outside the wire range."""
    if type(value) is not int or not minimum <= value <= maximum:
        raise SdkError(f"{label} must be an integer in {minimum}..={maximum}")


@dataclass(frozen=True, slots=True)
class V1ResourceConfig:
    """Inline limits and total priority fee; callers must choose explicit budgets."""

    #: Maximum compute units, 1..=1,400,000.
    compute_unit_limit: int
    #: Maximum loaded account data in bytes, 1..=64 MiB.
    loaded_accounts_data_size_limit: int
    #: Total transaction priority fee in integer lamports (u64).
    priority_fee_lamports: int
    #: Optional heap bytes, 32..=256 KiB in 1-KiB steps; None uses 32 KiB.
    heap_size: int | None = None

    def __post_init__(self) -> None:
        """Validate runtime limits and exact integer units at construction."""
        _integer(self.compute_unit_limit, 1, 1_400_000, "compute unit limit")
        _integer(
            self.loaded_accounts_data_size_limit,
            1,
            64 * 1024 * 1024,
            "loaded account data bytes",
        )
        _integer(
            self.priority_fee_lamports, 0, 2**64 - 1, "total priority fee lamports"
        )
        if self.heap_size is not None:
            _integer(self.heap_size, 32 * 1024, 256 * 1024, "heap bytes")
            if self.heap_size % 1024:
                raise SdkError("heap bytes must be a multiple of 1024")

    def _config(self) -> TransactionConfig:
        """Produce the canonical runtime configuration (zero priority fee omitted)."""
        return TransactionConfig(
            priority_fee=self.priority_fee_lamports or None,
            compute_unit_limit=self.compute_unit_limit,
            loaded_accounts_data_size_limit=self.loaded_accounts_data_size_limit,
            heap_size=self.heap_size,
        )


@dataclass(frozen=True, slots=True)
class V1TransactionContext:
    """A recent blockhash, its original expiry height, and explicit resources."""

    #: Confirmed blockhash encoded in the signed message.
    blockhash: Hash
    #: Original last valid block height from the same RPC result.
    last_valid_block_height: int
    resources: V1ResourceConfig

    def __post_init__(self) -> None:
        """Reject incomplete blockhash evidence and resource configurations."""
        if not isinstance(self.blockhash, Hash) or self.blockhash == Hash.default():
            raise SdkError("a recent nonzero blockhash is required")
        _integer(self.last_valid_block_height, 1, 2**64 - 1, "last valid block height")
        if not isinstance(self.resources, V1ResourceConfig):
            raise SdkError("explicit V1ResourceConfig is required")


@dataclass(frozen=True, slots=True)
class V1Transaction:
    """Canonical v1 bytes bound to their original blockhash and resource context.

    Immutable bytes own the message and signatures. Inspection returns independent
    solders values, so changing them cannot mutate a prepared transaction.
    """

    _wire_bytes: bytes
    context: V1TransactionContext
    MAX_TRANSACTION_SIZE: ClassVar[int] = 4096
    MAX_ADDRESSES: ClassVar[int] = 64

    def __post_init__(self) -> None:
        """Validate every construction, including direct dataclass construction."""
        if not isinstance(self.context, V1TransactionContext):
            raise SdkError("a V1TransactionContext is required")
        if not isinstance(self._wire_bytes, bytes):
            raise SdkError("transaction wire data must be immutable bytes")
        if len(self._wire_bytes) > self.MAX_TRANSACTION_SIZE:
            raise SdkError("transaction exceeds the v1 wire size limit")
        if not self._wire_bytes or self._wire_bytes[0] != V1_PREFIX:
            raise SdkError("only Solana v1 transactions are supported")
        try:
            tx = VersionedTransaction.from_bytes(self._wire_bytes)
            message = tx.message
            if not isinstance(message, Message):
                raise SdkError("only Solana v1 transactions are supported")
            message.validate()
            tx.sanitize()
            if (
                message.lifetime_specifier != self.context.blockhash
                or message.config.compute_unit_limit
                != self.context.resources.compute_unit_limit
                or message.config.loaded_accounts_data_size_limit
                != self.context.resources.loaded_accounts_data_size_limit
                or (message.config.priority_fee or 0)
                != self.context.resources.priority_fee_lamports
                or message.config.heap_size != self.context.resources.heap_size
            ):
                raise SdkError("message does not match its blockhash/resource context")
            if not message.instructions:
                raise SdkError("transaction has no instructions")
            if len(message.account_keys) > self.MAX_ADDRESSES:
                raise SdkError("transaction exceeds 64 distinct account addresses")
            if any(
                message.account_keys[ix.program_id_index] == _COMPUTE_BUDGET
                for ix in message.instructions
            ):
                raise SdkError(
                    "configure v1 resources inline; ComputeBudget instructions are unsupported"
                )
            if bytes(tx) != self._wire_bytes:
                raise SdkError("transaction encoding is not canonical")
        except SdkError:
            raise
        except Exception as error:
            raise SdkError(f"invalid v1 transaction: {error}") from error

    @classmethod
    def compile(
        cls,
        instructions: Sequence[Instruction],
        payer: Pubkey,
        context: V1TransactionContext,
    ) -> V1Transaction:
        """Compile offline with inline keys and all required signature slots."""
        if not isinstance(context, V1TransactionContext):
            raise SdkError("a V1TransactionContext is required")
        try:
            message = Message.try_compile(
                payer, instructions, context.blockhash, context.resources._config()
            )
            tx = VersionedTransaction.populate(
                message, [Signature.default()] * message.header.num_required_signatures
            )
            return cls.from_versioned(tx, context)
        except SdkError:
            raise
        except Exception as error:
            raise SdkError(f"v1 transaction compilation failed: {error}") from error

    @classmethod
    def from_versioned(
        cls, transaction: VersionedTransaction, context: V1TransactionContext
    ) -> V1Transaction:
        """Import a v1 solders transaction; reject legacy and v0 values."""
        if not isinstance(transaction, VersionedTransaction) or not isinstance(
            transaction.message, Message
        ):
            raise SdkError("only Solana v1 transactions are supported")
        return cls(bytes(transaction), context)

    @classmethod
    def from_wire_bytes(
        cls, wire_bytes: bytes, context: V1TransactionContext
    ) -> V1Transaction:
        """Import canonical v1 bytes, rejecting malformed or trailing data."""
        return cls(wire_bytes, context)

    @property
    def message(self) -> Message:
        """Return an independent message for inspection."""
        return self.as_versioned().message

    @property
    def signatures(self) -> tuple[Signature, ...]:
        """Return signature slots in required signer order."""
        return tuple(self.as_versioned().signatures)

    @property
    def required_signers(self) -> tuple[Pubkey, ...]:
        """Return distinct required signers, beginning with the fee payer."""
        message = self.message
        return tuple(message.account_keys[: message.header.num_required_signatures])

    def as_versioned(self) -> VersionedTransaction:
        """Return an independent solders transaction for integration."""
        return VersionedTransaction.from_bytes(self._wire_bytes)

    def message_bytes(self) -> bytes:
        """Return the canonical version-prefixed signing and fee-estimation bytes."""
        return to_bytes_versioned(self.message)

    def to_wire_bytes(self) -> bytes:
        """Return canonical bytes including every required signature slot."""
        return self._wire_bytes

    def __bytes__(self) -> bytes:
        return self.to_wire_bytes()

    def sign(self, signers: Sequence[Keypair]) -> V1Transaction:
        """Sign all required keys once; supplied order and duplicates do not matter."""
        supplied = {signer.pubkey(): signer for signer in signers}
        try:
            ordered = [supplied[key] for key in self.required_signers]
            signed = self.from_versioned(
                VersionedTransaction(self.message, ordered), self.context
            )
            signed.verify_signatures()
            return signed
        except KeyError as error:
            raise SdkError(f"missing signer {error.args[0]}") from error
        except SdkError:
            raise
        except Exception as error:
            raise SdkError(f"transaction signing failed: {error}") from error

    def verify_signatures(self) -> None:
        """Require a valid signature from every signer over this exact message."""
        try:
            self.as_versioned().verify_and_hash_message()
        except Exception as error:
            raise SdkError(f"invalid transaction signatures: {error}") from error

    def accept_signed_bytes(self, wire_bytes: bytes) -> V1Transaction:
        """Accept wallet bytes only if the message is unchanged and fully signed."""
        signed = self.from_wire_bytes(wire_bytes, self.context)
        if signed.message_bytes() != self.message_bytes():
            raise SdkError("wallet changed the prepared transaction message")
        signed.verify_signatures()
        return signed
