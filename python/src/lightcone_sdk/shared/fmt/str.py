"""String formatting helpers."""


def shorten(value: str, qty: int) -> str:
    """Shorten a string to its first and last ``qty // 2`` characters joined
    by an ellipsis — e.g. ``shorten("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR", 8)``
    → ``"FRGk...WcPR"``. Strings of ``qty`` characters or fewer are returned
    unchanged.
    """
    if len(value) > qty:
        chars_to_show = qty // 2
        return f"{value[:chars_to_show]}...{value[len(value) - chars_to_show:]}"
    return value
