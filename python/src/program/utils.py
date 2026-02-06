"""Utility functions for the Lightcone program module."""

import struct
from typing import Union

from Crypto.Hash import keccak
from solders.pubkey import Pubkey

from .constants import (
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_2022_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    MAX_OUTCOMES,
    MIN_OUTCOMES
)


def keccak256(data: bytes) -> bytes:
    """Compute keccak256 hash of data."""
    k = keccak.new(digest_bits=256)
    k.update(data)
    return k.digest()


def encode_u8(value: int) -> bytes:
    """Encode an unsigned 8-bit integer.

    Raises:
        ValueError: If value is out of range [0, 255]
    """
    if not 0 <= value <= 255:
        raise ValueError(f"u8 value out of range: {value} (must be 0-255)")
    return struct.pack("<B", value)


def encode_u16(value: int) -> bytes:
    """Encode an unsigned 16-bit integer (little-endian).

    Raises:
        ValueError: If value is out of range [0, 65535]
    """
    if not 0 <= value <= 65535:
        raise ValueError(f"u16 value out of range: {value} (must be 0-65535)")
    return struct.pack("<H", value)


def encode_u32(value: int) -> bytes:
    """Encode an unsigned 32-bit integer (little-endian).

    Raises:
        ValueError: If value is out of range [0, 4294967295]
    """
    if not 0 <= value <= 4294967295:
        raise ValueError(f"u32 value out of range: {value} (must be 0-4294967295)")
    return struct.pack("<I", value)


def encode_u64(value: int) -> bytes:
    """Encode an unsigned 64-bit integer (little-endian).

    Raises:
        ValueError: If value is out of range [0, 2^64-1]
    """
    if not 0 <= value <= 18446744073709551615:
        raise ValueError(f"u64 value out of range: {value} (must be 0-18446744073709551615)")
    return struct.pack("<Q", value)


def encode_i64(value: int) -> bytes:
    """Encode a signed 64-bit integer (little-endian).

    Raises:
        ValueError: If value is out of range [-2^63, 2^63-1]
    """
    if not -9223372036854775808 <= value <= 9223372036854775807:
        raise ValueError(f"i64 value out of range: {value}")
    return struct.pack("<q", value)


def decode_u8(data: bytes, offset: int = 0) -> int:
    """Decode an unsigned 8-bit integer."""
    return struct.unpack_from("<B", data, offset)[0]


def decode_u16(data: bytes, offset: int = 0) -> int:
    """Decode an unsigned 16-bit integer (little-endian)."""
    return struct.unpack_from("<H", data, offset)[0]


def decode_u32(data: bytes, offset: int = 0) -> int:
    """Decode an unsigned 32-bit integer (little-endian)."""
    return struct.unpack_from("<I", data, offset)[0]


def decode_u64(data: bytes, offset: int = 0) -> int:
    """Decode an unsigned 64-bit integer (little-endian)."""
    return struct.unpack_from("<Q", data, offset)[0]


def decode_i64(data: bytes, offset: int = 0) -> int:
    """Decode a signed 64-bit integer (little-endian)."""
    return struct.unpack_from("<q", data, offset)[0]


def decode_pubkey(data: bytes, offset: int = 0) -> Pubkey:
    """Decode a Pubkey from 32 bytes.

    Raises:
        ValueError: If not enough bytes available for Pubkey
    """
    if offset + 32 > len(data):
        raise ValueError(
            f"Not enough bytes for Pubkey at offset {offset}: "
            f"need 32 bytes, have {len(data) - offset}"
        )
    return Pubkey.from_bytes(data[offset : offset + 32])


def decode_bool(data: bytes, offset: int = 0) -> bool:
    """Decode a boolean from a single byte.

    Raises:
        ValueError: If not enough bytes available
    """
    if offset >= len(data):
        raise ValueError(f"Not enough bytes for bool at offset {offset}")
    return data[offset] != 0


def pubkey_to_bytes(pubkey: Union[Pubkey, bytes]) -> bytes:
    """Convert a Pubkey to bytes."""
    if isinstance(pubkey, bytes):
        return pubkey
    return bytes(pubkey)


def get_associated_token_address(
    owner: Pubkey,
    mint: Pubkey,
    token_program_id: Pubkey = TOKEN_PROGRAM_ID,
) -> Pubkey:
    """Derive the associated token account address for a wallet and mint."""
    seeds = [
        bytes(owner),
        bytes(token_program_id),
        bytes(mint),
    ]
    pda, _ = Pubkey.find_program_address(seeds, ASSOCIATED_TOKEN_PROGRAM_ID)
    return pda


def get_associated_token_address_2022(owner: Pubkey, mint: Pubkey) -> Pubkey:
    """Derive the ATA address for Token-2022 tokens."""
    return get_associated_token_address(owner, mint, TOKEN_2022_PROGRAM_ID)


def derive_condition_id(
    oracle: Pubkey,
    question_id: bytes,
    num_outcomes: int,
) -> bytes:
    """Derive the condition ID for a market.

    condition_id = keccak256(oracle || question_id || num_outcomes)
    """
    data = bytes(oracle) + question_id + encode_u8(num_outcomes)
    return keccak256(data)


def encode_string(s: str, max_len: int) -> bytes:
    """Encode a string with u16 length prefix (matches Rust SDK).

    Format: [length (2 bytes LE)][utf-8 bytes]
    """
    encoded = s.encode("utf-8")
    if len(encoded) > max_len:
        raise ValueError(f"String too long: {len(encoded)} > {max_len}")
    # Length prefix (2 bytes u16 LE) + string content
    return struct.pack("<H", len(encoded)) + encoded


def encode_string_fixed(s: str, max_len: int) -> bytes:
    """Encode a string as fixed-length with null padding."""
    encoded = s.encode("utf-8")
    if len(encoded) > max_len:
        raise ValueError(f"String too long: {len(encoded)} > {max_len}")
    return encoded + b"\x00" * (max_len - len(encoded))


def validate_outcome_count(num_outcomes: int) -> None:
    """Validate that the outcome count is within bounds."""

    if num_outcomes < MIN_OUTCOMES or num_outcomes > MAX_OUTCOMES:
        raise ValueError(
            f"Invalid outcome count: {num_outcomes} "
            f"(must be between {MIN_OUTCOMES} and {MAX_OUTCOMES})"
        )


def validate_order_hash(order_hash: bytes) -> None:
    """Validate that an order hash is 32 bytes."""
    if len(order_hash) != 32:
        raise ValueError(f"Invalid order hash length: {len(order_hash)} (expected 32)")


def orders_cross(
    bid_maker_amount: int,
    bid_taker_amount: int,
    ask_maker_amount: int,
    ask_taker_amount: int,
) -> bool:
    """Check if a bid and ask order cross (prices match).

    For orders to cross: buyer_price >= seller_price
    buyer_price = bid_maker_amount / bid_taker_amount (quote per base)
    seller_price = ask_taker_amount / ask_maker_amount (quote per base)

    Cross condition (using 128-bit math to avoid overflow):
    bid_maker_amount * ask_maker_amount >= bid_taker_amount * ask_taker_amount
    """
    return bid_maker_amount * ask_maker_amount >= bid_taker_amount * ask_taker_amount
