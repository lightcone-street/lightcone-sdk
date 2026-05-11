"""Number formatting helpers mirroring Rust's shared/fmt/num.rs."""

from decimal import Decimal
import math

from .constants import (
    DEFAULT_DECIMALS,
    MAX_STANDARD_DECIMALS,
    TINY_SIGNIFICANT_DIGITS,
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


def _get_decimal_places(value: float) -> int:
    abs_value = abs(value)

    if abs_value == 0.0 or abs_value >= 0.005:
        return DEFAULT_DECIMALS

    exponent = math.floor(math.log10(abs_value))
    leading_zeros = max(-exponent - 1, 0)
    return min(leading_zeros + TINY_SIGNIFICANT_DIGITS, MAX_STANDARD_DECIMALS)


def _trim_trailing_fraction_zeros(formatted: str) -> str:
    if "." not in formatted:
        return formatted
    return formatted.rstrip("0").rstrip(".")


def _leading_zero_count(value: float) -> int:
    exponent = math.floor(math.log10(abs(value)))
    return max(-exponent - 1, 0)


def _display_subscript(value: float, leading_zeros: int) -> str:
    sign = "-" if value < 0 else ""
    scaled = abs(value) * (10 ** (leading_zeros + 1))
    significant = _trim_trailing_fraction_zeros(f"{scaled:.3f}").replace(".", "")
    return f"{sign}0.0({leading_zeros}){significant}"


def display(amount: float) -> str:
    """Format a float for display with Rust-style decimal selection."""
    abs_value = abs(amount)
    if abs_value != 0.0 and abs_value < 0.005:
        leading_zeros = _leading_zero_count(abs_value)
        if leading_zeros + 1 > MAX_STANDARD_DECIMALS:
            return _display_subscript(amount, leading_zeros)

    decimals = _get_decimal_places(amount)
    formatted = f"{amount:.{decimals}f}"
    if abs_value != 0.0 and abs_value < 0.005:
        formatted = _trim_trailing_fraction_zeros(formatted)
    return display_formatted_string(formatted)


def display_with_decimals(amount: float, decimals: int) -> str:
    return display_formatted_string(f"{amount:.{decimals}f}")


def to_decimal_value(value: int, decimals: int) -> float:
    return value / (10 ** decimals)


def from_decimal_value(value: float, decimals: int) -> int:
    return int(Decimal(str(value)) * (Decimal(10) ** decimals))
