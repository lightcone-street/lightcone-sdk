"""Query-string encoding for paging parameters."""

from __future__ import annotations

from ..error import ValidationError

_U32_MAX = (1 << 32) - 1


def query_u32(name: str, value: int) -> str:
    """Encode a ``u32`` paging parameter (``limit``, ``fills``).

    Raises :class:`~lightcone_sdk.error.ValidationError` for a ``bool``, a
    non-integer, or a value outside ``0..=u32::MAX``, which the Rust SDK's
    ``u32`` parameters make unrepresentable. The backend clamps or rejects
    values outside each route's own range.
    """
    if isinstance(value, bool) or not isinstance(value, int):
        raise ValidationError(f"{name} must be an integer, got {value!r}")
    if value < 0 or value > _U32_MAX:
        raise ValidationError(f"{name} must be between 0 and {_U32_MAX}, got {value}")
    return str(value)


__all__ = ["query_u32"]
