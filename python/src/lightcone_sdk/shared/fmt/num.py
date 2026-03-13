"""Number formatting helpers mirroring Rust's shared/fmt/num.rs."""

from decimal import Decimal
import math


def display_formatted_string(formatted: str) -> str:
    """Trim trailing zeros and add thousands separators."""
    trimmed = formatted
    if "." in trimmed:
        trimmed = trimmed.rstrip("0").rstrip(".")

    if not trimmed:
        return "0"

    sign = ""
    if trimmed.startswith("-"):
        sign = "-"
        trimmed = trimmed[1:]

    integer_part, dot, fraction_part = trimmed.partition(".")
    try:
        integer_formatted = f"{int(integer_part):,}"
    except ValueError:
        integer_formatted = integer_part

    if dot:
        return f"{sign}{integer_formatted}.{fraction_part}"
    return f"{sign}{integer_formatted}"


def _get_decimal_places(value: float) -> int:
    abs_value = abs(value)

    if abs_value >= 100.0:
        return 0
    if abs_value >= 1.0:
        return 2
    if abs_value == 0.0:
        return 2

    exponent = math.floor(math.log10(abs_value))
    return min(abs(exponent) + 2, 8)


def display(amount: float) -> str:
    """Format a float for display with Rust-style decimal selection."""
    return display_with_decimals(amount, _get_decimal_places(amount))


def display_with_decimals(amount: float, decimals: int) -> str:
    return display_formatted_string(f"{amount:.{decimals}f}")


def to_decimal_value(value: int, decimals: int) -> float:
    return value / (10 ** decimals)


def from_decimal_value(value: float, decimals: int) -> int:
    return int(Decimal(str(value)) * (Decimal(10) ** decimals))
