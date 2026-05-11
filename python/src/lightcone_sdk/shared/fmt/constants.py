"""Shared formatter precision constants."""

DEFAULT_DECIMALS = 2
TINY_SIGNIFICANT_DIGITS = 3
MAX_STANDARD_DECIMALS = 8
SUBSCRIPT_SIGNIFICANT_DIGITS = 4


def display_format(
    *,
    is_zero: bool,
    rounds_to_default_nonzero: bool,
    leading_zeros: int,
) -> tuple[int, bool] | None:
    if is_zero or rounds_to_default_nonzero:
        return DEFAULT_DECIMALS, False

    if leading_zeros + 1 > MAX_STANDARD_DECIMALS:
        return None

    return min(leading_zeros + TINY_SIGNIFICANT_DIGITS, MAX_STANDARD_DECIMALS), True


def trim_trailing_fraction_zeros(formatted: str) -> str:
    if "." not in formatted:
        return formatted
    return formatted.rstrip("0").rstrip(".")
