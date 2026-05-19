"""Decimal formatting helpers mirroring Rust's shared/fmt/decimal.rs."""

from __future__ import annotations

from decimal import Decimal, ROUND_DOWN, ROUND_HALF_UP

from .constants import display_decimals_by
from .num import display_default_formatted_string, display_formatted_string

_THOUSAND = Decimal("1000")
_MILLION = Decimal("1000000")
_BILLION = Decimal("1000000000")
_TRILLION = Decimal("1000000000000")


def _display_decimals(abs_value: Decimal) -> int:
    return display_decimals_by(lambda threshold: abs_value >= Decimal(threshold))


def display(value: Decimal) -> str:
    """Format a Decimal using the Rust display rules."""
    decimals = _display_decimals(abs(value))
    quantizer = Decimal(1).scaleb(-decimals)
    rounded = value.quantize(quantizer, rounding=ROUND_HALF_UP)
    return display_default_formatted_string(format(rounded, "f"))


def abbr_number(amount: Decimal, digits: int | None = None, show_sign: bool | None = None) -> str:
    digits = 2 if digits is None else digits
    show_sign = True if show_sign is None else show_sign
    sign = "-" if show_sign and amount < 0 else ""
    abs_amount = abs(amount)

    if abs_amount >= _TRILLION:
        return f"{sign}{abs_amount / _TRILLION:.{digits}f}t"
    if abs_amount >= _BILLION:
        return f"{sign}{abs_amount / _BILLION:.{digits}f}b"
    if abs_amount >= _MILLION:
        return f"{sign}{abs_amount / _MILLION:.{digits}f}m"
    if abs_amount >= _THOUSAND:
        return f"{sign}{abs_amount / _THOUSAND:.{digits}f}k"
    return f"{sign}{abs_amount:.{digits}f}"


def display_pct(value: Decimal, padding: bool | None = None) -> str:
    """Format a Decimal as a percentage with exactly 2 decimal places (truncated).

    When padding is True (default), always shows 2 decimal places (e.g. "12.30").
    When False, trailing zeros are trimmed (e.g. "12.3").
    """
    if padding is None:
        padding = True
    truncated = value.quantize(Decimal("0.01"), rounding=ROUND_DOWN)

    if padding:
        return display_formatted_string(f"{truncated:.2f}")
    else:
        normalized = truncated.normalize()
        return display_formatted_string(format(normalized, "f"))


def to_base_units(value: Decimal, decimals: int) -> int | None:
    scaled = value * (Decimal(10) ** decimals)
    if scaled < 0:
        return None
    try:
        return int(scaled)
    except (ArithmeticError, ValueError):
        return None
