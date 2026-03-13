"""Price utilities used across the Lightcone SDK."""

from decimal import Decimal


def parse_decimal(s: str) -> Decimal:
    """Parse a decimal string to ``Decimal``."""
    return Decimal(s)


def format_decimal(value: Decimal | float | str, precision: int = 6) -> str:
    """Format a value as a decimal string with the specified precision.

    The implementation is ``Decimal``-first to avoid float-driven rounding
    surprises in request payloads and typed domain conversions.
    """
    decimal_value = value if isinstance(value, Decimal) else Decimal(str(value))
    quantized = decimal_value.quantize(Decimal(1).scaleb(-precision))
    return format(quantized, "f")


def is_zero(value: str) -> bool:
    """Check if a decimal string represents zero."""
    try:
        return Decimal(value) == 0
    except Exception:
        return False
