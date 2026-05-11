"""Number formatting helpers mirroring Rust's shared/fmt/num.rs."""

from decimal import Decimal

from .constants import display_decimals


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


def display(amount: float) -> str:
    """Format a float for display with Rust-style decimal selection."""
    return display_with_decimals(amount, display_decimals(abs(amount)))


def display_with_decimals(amount: float, decimals: int) -> str:
    return display_formatted_string(f"{amount:.{decimals}f}")


def to_decimal_value(value: int, decimals: int) -> float:
    return value / (10 ** decimals)


def from_decimal_value(value: float, decimals: int) -> int:
    return int(Decimal(str(value)) * (Decimal(10) ** decimals))
