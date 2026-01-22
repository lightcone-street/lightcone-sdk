"""Input validation utilities for the API client."""

import re

from solders.pubkey import Pubkey

from .error import InvalidParameterError

# Hex signature pattern (128 characters for Ed25519 signature)
HEX_SIGNATURE_PATTERN = re.compile("^[0-9a-fA-F]{128}$")

MAX_PAGINATION_LIMIT = 500


def validate_pubkey(value: str, field_name: str) -> None:
    """Validate that a string is a valid Solana pubkey (Base58).

    Uses solders.Pubkey for proper validation including length check.

    Raises:
        InvalidParameterError: If not a valid pubkey
    """
    if not value or not value.strip():
        raise InvalidParameterError(f"{field_name} cannot be empty")

    try:
        Pubkey.from_string(value)
    except Exception:
        raise InvalidParameterError(f"{field_name} is not a valid pubkey")


def validate_signature(signature: str) -> None:
    """Validate that a signature is 128 hex characters.

    Raises:
        InvalidParameterError: If signature is invalid
    """
    if not HEX_SIGNATURE_PATTERN.match(signature):
        raise InvalidParameterError(
            f"Signature must be 128 hex characters, got {len(signature)}"
        )


def validate_limit(limit: int) -> None:
    """Validate pagination limit is within bounds.

    Raises:
        InvalidParameterError: If limit is out of bounds
    """
    if limit < 1 or limit > MAX_PAGINATION_LIMIT:
        raise InvalidParameterError(f"Limit must be 1-{MAX_PAGINATION_LIMIT}")
