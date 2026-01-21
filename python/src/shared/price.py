"""Price utilities used by API and WebSocket modules."""

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


def is_zero_size(size: str) -> bool:
    """Check if a size string represents zero.

    Used for filtering out zero-size entries in orderbooks.
    """
    try:
        return float(size) == 0.0
    except (ValueError, TypeError):
        return True
