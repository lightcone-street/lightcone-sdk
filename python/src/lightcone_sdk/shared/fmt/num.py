"""Number formatting helpers mirroring Rust's shared/fmt/num.rs."""

from decimal import Decimal

from .constants import display_decimals_by


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


def _is_formatted_zero(formatted: str) -> bool:
    value = formatted[1:] if formatted.startswith("-") else formatted
    integer_part, dot, fraction_part = value.partition(".")
    return (
        bool(integer_part)
        and all(char == "0" for char in integer_part)
        and (not dot or all(char == "0" for char in fraction_part))
    )


def display_default_formatted_string(formatted: str) -> str:
    if _is_formatted_zero(formatted):
        return "0"
    return display_formatted_string(formatted)


def _display_decimals(abs_value: float) -> int:
    return display_decimals_by(lambda threshold: abs_value >= float(threshold))


def display(amount: float) -> str:
    """Format a float for display with Rust-style decimal selection."""
    decimals = _display_decimals(abs(amount))
    return display_default_formatted_string(f"{amount:.{decimals}f}")


def display_with_decimals(amount: float, decimals: int) -> str:
    return display_formatted_string(f"{amount:.{decimals}f}")


def to_decimal_value(value: int, decimals: int) -> float:
    return value / (10 ** decimals)


def display_pct(value: float, padding: bool | None = None) -> str:
    """Format a float as a percentage with exactly 2 decimal places (truncated).

    When padding is True (default), always shows 2 decimal places (e.g. "12.30").
    When False, trailing zeros are trimmed (e.g. "12.3").
    """
    if padding is None:
        padding = True
    import math
    truncated = math.trunc(value * 100) / 100

    if padding:
        return display_formatted_string(f"{truncated:.2f}")
    else:
        formatted = f"{truncated:.2f}"
        trimmed = formatted.rstrip("0").rstrip(".")
        return display_formatted_string(trimmed)


def from_decimal_value(value: float, decimals: int) -> int:
    return int(Decimal(str(value)) * (Decimal(10) ** decimals))
