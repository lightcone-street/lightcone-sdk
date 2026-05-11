"""Number formatting helpers mirroring Rust's shared/fmt/num.rs."""

from decimal import Decimal
import math

from .constants import (
    SUBSCRIPT_SIGNIFICANT_DIGITS,
    display_format,
    trim_trailing_fraction_zeros,
)


def display_formatted_string(formatted: str) -> str:
    """Add thousands separators while preserving fractional digits."""
    if not formatted:
        return "0"

    sign = ""
    if formatted.startswith("-"):
        sign = "-"
        formatted = formatted[1:]

    integer_part, dot, fraction_part = formatted.partition(".")
    try:
        integer_formatted = f"{int(integer_part):,}"
    except ValueError:
        integer_formatted = integer_part

    if dot:
        return f"{sign}{integer_formatted}.{fraction_part}"
    return f"{sign}{integer_formatted}"


def _leading_zero_count(value: float) -> int:
    exponent = math.floor(math.log10(abs(value)))
    return max(-exponent - 1, 0)


def _display_subscript(value: float, leading_zeros: int) -> str:
    sign = "-" if value < 0 else ""
    scaled = abs(value) * (10 ** (leading_zeros + 1))
    factor = 10 ** (SUBSCRIPT_SIGNIFICANT_DIGITS - 1)
    significant = int(scaled * factor)
    while significant > 0 and significant % 10 == 0:
        significant //= 10
    return f"{sign}0.0({leading_zeros}){significant}"


def display(amount: float) -> str:
    """Format a float for display with Rust-style decimal selection."""
    abs_value = abs(amount)
    leading_zeros = 0 if abs_value == 0.0 else _leading_zero_count(abs_value)
    policy = display_format(
        is_zero=abs_value == 0.0,
        rounds_to_default_nonzero=abs_value >= 0.005,
        leading_zeros=leading_zeros,
    )

    if policy is None:
        return _display_subscript(amount, leading_zeros)

    decimals, trim_tiny_zeros = policy
    formatted = f"{amount:.{decimals}f}"
    if trim_tiny_zeros:
        formatted = trim_trailing_fraction_zeros(formatted)
    return display_formatted_string(formatted)


def display_with_decimals(amount: float, decimals: int) -> str:
    return display_formatted_string(f"{amount:.{decimals}f}")


def to_decimal_value(value: int, decimals: int) -> float:
    return value / (10 ** decimals)


def from_decimal_value(value: float, decimals: int) -> int:
    return int(Decimal(str(value)) * (Decimal(10) ** decimals))
