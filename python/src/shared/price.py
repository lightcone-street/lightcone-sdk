"""Price utilities used across the Lightcone SDK."""

from decimal import Decimal


def parse_decimal(s: str) -> float:
    """Parse a decimal string to a float.

    Used for parsing price values from the API.
    """
    return float(Decimal(s))


def format_decimal(value: float, precision: int = 6) -> str:
    """Format a float as a decimal string with the specified precision.

    Used for formatting price values for API requests.
    """
    return f"{value:.{precision}f}"


def is_zero(value: str) -> bool:
    """Check if a decimal string represents zero."""
    try:
        return Decimal(value) == 0
    except Exception:
        return False
