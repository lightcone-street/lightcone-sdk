"""Admin domain types."""

from dataclasses import dataclass
from typing import Optional


@dataclass
class AdminEnvelope:
    """Signed admin request envelope."""
    payload: dict
    signature: str


__all__ = ["AdminEnvelope"]
