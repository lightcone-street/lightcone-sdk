"""Ed25519 verification instruction helpers for the Lightcone SDK.

The Ed25519 program verifies Ed25519 signatures on-chain. This module provides
helpers for building Ed25519 verify instructions for order verification.
"""

from dataclasses import dataclass
from typing import List

from solders.instruction import Instruction
from solders.pubkey import Pubkey

from ..shared.constants import ED25519_PROGRAM_ID, ORDER_HASH_SIZE, SIGNATURE_SIZE
from .orders import hash_order
from ..shared.types import FullOrder
from ..shared.utils import encode_u16, encode_u8


@dataclass
class Ed25519SignatureOffsets:
    """Offsets for Ed25519 signature verification data."""

    signature_offset: int  # u16
    signature_instruction_index: int  # u16
    public_key_offset: int  # u16
    public_key_instruction_index: int  # u16
    message_data_offset: int  # u16
    message_data_size: int  # u16
    message_instruction_index: int  # u16


def build_ed25519_verify_instruction(
    pubkey: bytes,
    signature: bytes,
    message: bytes,
) -> Instruction:
    """Build an Ed25519 verify instruction for a single signature.

    The instruction data layout:
    - num_signatures (1 byte): Number of signatures to verify
    - padding (1 byte): Zero padding
    - For each signature:
      - signature_offset (u16 LE): Offset to signature in data
      - signature_instruction_index (u16 LE): 0xFFFF for current instruction
      - public_key_offset (u16 LE): Offset to pubkey in data
      - public_key_instruction_index (u16 LE): 0xFFFF for current instruction
      - message_data_offset (u16 LE): Offset to message in data
      - message_data_size (u16 LE): Length of message
      - message_instruction_index (u16 LE): 0xFFFF for current instruction
    - Followed by: signature (64), pubkey (32), message (variable)
    """
    if len(signature) != SIGNATURE_SIZE:
        raise ValueError(f"Signature must be {SIGNATURE_SIZE} bytes")
    if len(pubkey) != 32:
        raise ValueError("Public key must be 32 bytes")

    num_signatures = 1
    header_size = 2 + 14  # 2 bytes header + 14 bytes per signature offset struct

    # Calculate offsets within the instruction data
    signature_offset = header_size
    public_key_offset = signature_offset + SIGNATURE_SIZE
    message_offset = public_key_offset + 32

    # Build instruction data
    data = bytearray()

    # Header
    data.append(num_signatures)  # num_signatures
    data.append(0)  # padding

    # Signature offsets struct
    data.extend(encode_u16(signature_offset))
    data.extend(encode_u16(0xFFFF))  # current instruction
    data.extend(encode_u16(public_key_offset))
    data.extend(encode_u16(0xFFFF))  # current instruction
    data.extend(encode_u16(message_offset))
    data.extend(encode_u16(len(message)))
    data.extend(encode_u16(0xFFFF))  # current instruction

    # Append signature, pubkey, message
    data.extend(signature)
    data.extend(pubkey)
    data.extend(message)

    return Instruction(
        program_id=ED25519_PROGRAM_ID,
        accounts=[],  # Ed25519 program takes no accounts
        data=bytes(data),
    )


def build_ed25519_verify_instruction_for_order(order: FullOrder) -> Instruction:
    """Build an Ed25519 verify instruction for a signed order.

    Verifies that the order's signature is valid for the order's hash.
    """
    order_hash = hash_order(order)
    return build_ed25519_verify_instruction(
        pubkey=bytes(order.maker),
        signature=order.signature,
        message=order_hash,
    )


def build_ed25519_batch_verify_instruction(
    orders: List[FullOrder],
) -> Instruction:
    """Build an Ed25519 verify instruction for multiple orders.

    All signatures are verified in a single instruction, which is more
    efficient than multiple individual verify instructions.
    """
    if not orders:
        raise ValueError("Must provide at least one order")

    num_signatures = len(orders)
    header_size = 2 + (14 * num_signatures)  # 2 bytes + 14 per signature

    # Calculate where each signature/pubkey/message will be placed
    current_offset = header_size

    signature_data = bytearray()
    offsets_data = bytearray()

    for order in orders:
        order_hash = hash_order(order)

        sig_offset = current_offset
        current_offset += SIGNATURE_SIZE

        pk_offset = current_offset
        current_offset += 32

        msg_offset = current_offset
        current_offset += ORDER_HASH_SIZE

        # Build offsets for this signature
        offsets_data.extend(encode_u16(sig_offset))
        offsets_data.extend(encode_u16(0xFFFF))
        offsets_data.extend(encode_u16(pk_offset))
        offsets_data.extend(encode_u16(0xFFFF))
        offsets_data.extend(encode_u16(msg_offset))
        offsets_data.extend(encode_u16(ORDER_HASH_SIZE))
        offsets_data.extend(encode_u16(0xFFFF))

        # Append the actual data
        signature_data.extend(order.signature)
        signature_data.extend(bytes(order.maker))
        signature_data.extend(order_hash)

    # Build final instruction data
    data = bytearray()
    data.append(num_signatures)
    data.append(0)  # padding
    data.extend(offsets_data)
    data.extend(signature_data)

    return Instruction(
        program_id=ED25519_PROGRAM_ID,
        accounts=[],
        data=bytes(data),
    )


def build_ed25519_cross_ref_instruction(
    orders: List[FullOrder],
    target_instruction_index: int,
) -> Instruction:
    """Build an Ed25519 verify instruction that references data in another instruction.

    This is used when the order data is already present in another instruction
    (like match_orders_multi) and we want to verify signatures without
    duplicating the data.

    Args:
        orders: List of orders to verify
        target_instruction_index: Index of the instruction containing the order data

    NOTE: This is the legacy implementation that still duplicates data.
    For efficient cross-ref verification, use create_cross_ref_ed25519_instructions.
    """
    if not orders:
        raise ValueError("Must provide at least one order")

    num_signatures = len(orders)
    header_size = 2 + (14 * num_signatures)

    current_offset = header_size
    signature_data = bytearray()
    offsets_data = bytearray()

    for order in orders:
        order_hash = hash_order(order)

        sig_offset = current_offset
        current_offset += SIGNATURE_SIZE

        pk_offset = current_offset
        current_offset += 32

        msg_offset = current_offset
        current_offset += ORDER_HASH_SIZE

        offsets_data.extend(encode_u16(sig_offset))
        offsets_data.extend(encode_u16(0xFFFF))
        offsets_data.extend(encode_u16(pk_offset))
        offsets_data.extend(encode_u16(0xFFFF))
        offsets_data.extend(encode_u16(msg_offset))
        offsets_data.extend(encode_u16(ORDER_HASH_SIZE))
        offsets_data.extend(encode_u16(0xFFFF))

        signature_data.extend(order.signature)
        signature_data.extend(bytes(order.maker))
        signature_data.extend(order_hash)

    data = bytearray()
    data.append(num_signatures)
    data.append(0)
    data.extend(offsets_data)
    data.extend(signature_data)

    return Instruction(
        program_id=ED25519_PROGRAM_ID,
        accounts=[],
        data=bytes(data),
    )


# =============================================================================
# Efficient Cross-Reference Ed25519 Verification
# =============================================================================
# These instructions reference data within another instruction (match_orders_multi)
# rather than duplicating it, resulting in much smaller transaction sizes.


class MatchIxOffsets:
    """Offsets for data within the match_orders_multi instruction.

    Match instruction data layout:
    - [0]: discriminator (1 byte)
    - [1..33]: taker_hash (32 bytes) <- taker message
    - [33..98]: taker_compact (65 bytes): nonce(8) | maker(32) <- pubkey at 41
    - [98..162]: taker_signature (64 bytes)
    - [162]: num_makers (1 byte)
    - [163..]: maker entries (169 bytes each):
      - [0..32]: maker_hash (32 bytes) <- maker message
      - [32..97]: maker_compact (65 bytes): nonce(8) | maker(32) <- pubkey at offset+40
      - [97..161]: maker_signature (64 bytes)
      - [161..169]: fill_amount (8 bytes)
    """

    # Taker offsets
    TAKER_MESSAGE = 1  # taker_hash starts at offset 1
    TAKER_PUBKEY = 41  # compact.maker at 33 + 8
    TAKER_SIGNATURE = 98  # after hash (32) + compact (65)
    NUM_MAKERS = 162

    @staticmethod
    def maker_offsets(maker_index: int) -> tuple[int, int, int]:
        """Get (message, pubkey, signature) offsets for a maker.

        Each maker entry is 169 bytes: hash(32) + compact(65) + sig(64) + fill(8)
        """
        base = 163 + maker_index * 169
        message = base
        pubkey = base + 32 + 8  # after hash, skip nonce
        signature = base + 32 + 65  # after hash + compact
        return (message, pubkey, signature)


@dataclass
class CrossRefEd25519Params:
    """Parameters for a single cross-reference Ed25519 verify instruction."""

    signature_offset: int
    signature_ix_index: int
    pubkey_offset: int
    pubkey_ix_index: int
    message_offset: int
    message_size: int
    message_ix_index: int


def create_single_cross_ref_ed25519_instruction(params: CrossRefEd25519Params) -> Instruction:
    """Create a single Ed25519 instruction that references data in another instruction.

    This is only 16 bytes (just header with offsets) - no embedded signature/pubkey/message.
    """
    data = bytearray(16)

    # num_signatures (u8)
    data[0] = 1
    # padding (u8)
    data[1] = 0

    # Signature offsets (14 bytes total)
    offset = 2
    data[offset : offset + 2] = encode_u16(params.signature_offset)
    offset += 2
    data[offset : offset + 2] = encode_u16(params.signature_ix_index)
    offset += 2
    data[offset : offset + 2] = encode_u16(params.pubkey_offset)
    offset += 2
    data[offset : offset + 2] = encode_u16(params.pubkey_ix_index)
    offset += 2
    data[offset : offset + 2] = encode_u16(params.message_offset)
    offset += 2
    data[offset : offset + 2] = encode_u16(params.message_size)
    offset += 2
    data[offset : offset + 2] = encode_u16(params.message_ix_index)

    return Instruction(
        program_id=ED25519_PROGRAM_ID,
        accounts=[],
        data=bytes(data),
    )


def create_cross_ref_ed25519_instructions(
    num_makers: int,
    match_ix_index: int,
) -> List[Instruction]:
    """Create Ed25519 verify instructions that reference the match instruction.

    This creates one instruction per order (taker + each maker) that references
    data offsets within the match instruction, resulting in very small verify
    instructions (16 bytes each).

    Args:
        num_makers: Number of maker orders
        match_ix_index: Index of the match_orders_multi instruction in the transaction

    Returns:
        List of Ed25519 verify instructions (1 + num_makers total)
    """
    instructions = []

    # Taker verification instruction
    taker_ix = create_single_cross_ref_ed25519_instruction(
        CrossRefEd25519Params(
            signature_offset=MatchIxOffsets.TAKER_SIGNATURE,
            signature_ix_index=match_ix_index,
            pubkey_offset=MatchIxOffsets.TAKER_PUBKEY,
            pubkey_ix_index=match_ix_index,
            message_offset=MatchIxOffsets.TAKER_MESSAGE,
            message_size=32,
            message_ix_index=match_ix_index,
        )
    )
    instructions.append(taker_ix)

    # Maker verification instructions
    for i in range(num_makers):
        message, pubkey, signature = MatchIxOffsets.maker_offsets(i)
        maker_ix = create_single_cross_ref_ed25519_instruction(
            CrossRefEd25519Params(
                signature_offset=signature,
                signature_ix_index=match_ix_index,
                pubkey_offset=pubkey,
                pubkey_ix_index=match_ix_index,
                message_offset=message,
                message_size=32,
                message_ix_index=match_ix_index,
            )
        )
        instructions.append(maker_ix)

    return instructions
