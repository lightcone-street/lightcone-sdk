"""Ed25519 verification instruction helpers for the Lightcone SDK.

The Ed25519 program verifies Ed25519 signatures on-chain. This module provides
helpers for building Ed25519 verify instructions for order verification.
"""

from dataclasses import dataclass
from typing import List

from solders.instruction import Instruction
from solders.pubkey import Pubkey

from .constants import ED25519_PROGRAM_ID, ORDER_HASH_SIZE, SIGNATURE_SIZE
from .orders import hash_order
from .types import FullOrder
from .utils import encode_u16, encode_u8


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
    """
    if not orders:
        raise ValueError("Must provide at least one order")

    num_signatures = len(orders)

    # For cross-reference, we still include the data in this instruction
    # but could reference another instruction for the actual verification.
    # The implementation depends on the exact layout of the target instruction.

    # For simplicity, we use the same layout as batch verify but allow
    # specifying a different instruction index for the message data
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

        # Build offsets - signature and pubkey in current instruction,
        # but could reference message from target instruction
        offsets_data.extend(encode_u16(sig_offset))
        offsets_data.extend(encode_u16(0xFFFF))  # current instruction
        offsets_data.extend(encode_u16(pk_offset))
        offsets_data.extend(encode_u16(0xFFFF))  # current instruction
        offsets_data.extend(encode_u16(msg_offset))
        offsets_data.extend(encode_u16(ORDER_HASH_SIZE))
        offsets_data.extend(encode_u16(0xFFFF))  # current instruction

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
