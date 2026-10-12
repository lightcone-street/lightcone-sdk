"""Exact decimal text for aggregates that can exceed a 96-bit decimal."""

from __future__ import annotations

import re
from decimal import Decimal
from typing import Any

from .wire_decode import MISSING, wire_error, wire_str

_DIGITS = re.compile(r"[0-9]+")
_MAX_MANTISSA = (1 << 96) - 1
_MAX_SCALE = 28


class DecimalText(str):
    """An exact, canonical decimal string such as ``"12.500000"`` or ``"-0.000001"``.

    The committed backend formats some aggregates (funding reservations)
    from ``u128`` atom counts, which can exceed a 96-bit decimal. They stay
    validated text; use :meth:`to_decimal` for arithmetic.

    Construction raises ``ValueError`` for anything but canonical decimal
    text: no sign but a leading ``-``, no exponent, no padding, no ``-0``.
    """

    __slots__ = ()

    def __new__(cls, text: str) -> DecimalText:
        if not isinstance(text, str):
            raise ValueError(f"invalid decimal text: {text!r}")
        unsigned = text[1:] if text.startswith("-") else text
        whole, dot, fraction = unsigned.partition(".")
        if _DIGITS.fullmatch(whole) is None or (
            dot and _DIGITS.fullmatch(fraction) is None
        ):
            raise ValueError(f"invalid decimal text: {text!r}")
        if text.startswith("-") and set(unsigned) <= {"0", "."}:
            raise ValueError(f"negative zero is not canonical: {text!r}")
        return super().__new__(cls, text)

    def is_negative(self) -> bool:
        """True when the value is negative (``-0`` is never accepted)."""
        return self.startswith("-")

    def to_decimal(self) -> Decimal | None:
        """The exact value as a ``Decimal``, or ``None`` beyond the Rust SDK
        decimal's range (a 96-bit mantissa and at most 28 fractional digits)."""
        unsigned = self[1:] if self.is_negative() else str(self)
        whole, _, fraction = unsigned.partition(".")
        if len(fraction) > _MAX_SCALE or int(whole + fraction) > _MAX_MANTISSA:
            return None
        return Decimal(str(self))


def wire_decimal_text(value: Any, field: str) -> DecimalText:
    """Decode a required ``DecimalText`` wire field; JSON numbers are rejected."""
    text = wire_str(value, field)
    try:
        return DecimalText(text)
    except ValueError as error:
        raise wire_error(field, str(error)) from None


def wire_opt_decimal_text(value: Any, field: str) -> DecimalText | None:
    """Decode an optional ``DecimalText`` wire field; ``null`` or absent is ``None``."""
    return (
        None if value is MISSING or value is None else wire_decimal_text(value, field)
    )


__all__ = ["DecimalText", "wire_decimal_text", "wire_opt_decimal_text"]
