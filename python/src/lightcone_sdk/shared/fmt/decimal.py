"""Decimal formatting helpers mirroring Rust's shared/fmt/decimal.rs."""

from __future__ import annotations

from decimal import Decimal, ROUND_HALF_UP

from .num import display_formatted_string

_HUNDRED = Decimal("100")
_THOUSAND = Decimal("1000")
_MILLION = Decimal("1000000")
_BILLION = Decimal("1000000000")
_TRILLION = Decimal("1000000000000")


def _leading_zero_count(value: Decimal) -> int:
    normalized = format(value.normalize(), "f")
    fraction = normalized.split(".", 1)[1] if "." in normalized else ""
    zeros = 0
    for char in fraction:
        if char == "0":
            zeros += 1
        else:
            break
    return zeros


def display(value: Decimal) -> str:
    """Format a Decimal using the Rust display rules."""
    if value == 0:
        return "0"

    abs_value = abs(value)
    if abs_value >= _HUNDRED:
        rounded = value.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
        return display_formatted_string(format(rounded, "f"))
    if abs_value >= Decimal("1"):
        rounded = value.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
        return display_formatted_string(format(rounded, "f"))

    leading_zeros = _leading_zero_count(abs_value)
    if leading_zeros > 5:
        digits = abs_value.scaleb(leading_zeros + 1)
        significant = format(digits.normalize(), "f").replace(".", "")[:4].rstrip("0") or "0"
        prefix = "-" if value.is_signed() else ""
        return f"{prefix}0.0({leading_zeros}){significant}"

    decimals = min(leading_zeros + 3, 8)
    quantizer = Decimal(1).scaleb(-decimals)
    rounded = value.quantize(quantizer, rounding=ROUND_HALF_UP)
    return display_formatted_string(format(rounded, "f"))


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


def to_base_units(value: Decimal, decimals: int) -> int | None:
    scaled = value * (Decimal(10) ** decimals)
    if scaled < 0:
        return None
    try:
        return int(scaled)
    except (ArithmeticError, ValueError):
        return None
