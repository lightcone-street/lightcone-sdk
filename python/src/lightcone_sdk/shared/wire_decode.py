"""Strict decoders for the committed trading backend's wire encodings.

The backend sends exact integers as JSON strings on REST and in WebSocket
snapshots but as JSON numbers in live WebSocket facts, and an empty string
where a protobuf string field is unset. Python's ``json`` keeps every integer
token exact, so integer decoders accept an ``int`` (never a ``bool`` or
``float``) and, where the Rust SDK accepts both encodings, digit-only text.

Every decoder raises :class:`~lightcone_sdk.error.DeserializationError`
naming the field, so a payload that does not match the contract fails loudly
instead of decoding to a silently wrong value.

Absent and ``null`` are distinct, as in the Rust SDK's serde rules: read
fields with :func:`take`, which returns :data:`MISSING` for an absent key.
``*_or``/``default`` decoders substitute their default only for an absent
field (``#[serde(default)]``) and reject ``null``; ``opt`` decoders map both
absent and ``null`` to ``None`` (``Option<T>``).
"""

from __future__ import annotations

import json
import re
from collections.abc import Callable
from datetime import UTC, datetime, timedelta
from decimal import Decimal
from enum import Enum
from typing import Any, Final, TypeVar

from ..error import DeserializationError
from .types import CommitInfo, FundingSource, OrderBookId, PubkeyStr, Side

T = TypeVar("T")
E = TypeVar("E", bound=Enum)

WireRecord = dict[str, Any]


class _Missing:
    """Sentinel for a field absent from its JSON object."""

    _instance: _Missing | None = None

    def __new__(cls) -> _Missing:
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def __repr__(self) -> str:
        return "MISSING"

    def __bool__(self) -> bool:
        return False


MISSING: Final = _Missing()

U64_MAX: Final = (1 << 64) - 1
I64_MIN: Final = -(1 << 63)
I64_MAX: Final = (1 << 63) - 1
I128_MIN: Final = -(1 << 127)
I128_MAX: Final = (1 << 127) - 1

# Digits in the widest wire integer (i128); longer text is out of range.
_MAX_INTEGER_DIGITS = 39
_DIGITS = re.compile(r"[0-9]+")
_SIGNED_DIGITS = re.compile(r"-?[0-9]+")
_DECIMAL = re.compile(r"-?[0-9]+(?:\.[0-9]+)?")
# chrono's relaxed RFC 3339: `T`, `t`, or a space separator; an offset is required.
_RFC3339 = re.compile(
    r"([0-9]{4}-[0-9]{2}-[0-9]{2})[Tt ]([0-9]{2}:[0-9]{2}:[0-9]{2})(\.[0-9]+)?"
    r"([Zz]|[+-][0-9]{2}:[0-9]{2})"
)
_EPOCH: Final = datetime(1970, 1, 1, tzinfo=UTC)


def _reject_json_constant(token: str) -> Any:
    raise ValueError(f"non-standard JSON constant {token}")


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    data: dict[str, Any] = {}
    for key, value in pairs:
        if key in data:
            raise ValueError(f"duplicate field {key!r}")
        data[key] = value
    return data


def parse_json(text: str, what: str) -> Any:
    """Parse strict JSON text, as the Rust SDK's serde decoding does.

    ``NaN`` and ``Infinity`` are rejected, and so are duplicate object keys
    (which Rust's derived decoders reject rather than let the last one win)
    and nesting too deep to decode. Raises
    :class:`~lightcone_sdk.error.DeserializationError` naming ``what``
    (``"response body"``, ``"WebSocket frame"``) when ``text`` is not JSON.
    """
    try:
        return json.loads(
            text,
            parse_constant=_reject_json_constant,
            object_pairs_hook=_reject_duplicate_keys,
        )
    except RecursionError:
        raise DeserializationError(f"invalid JSON {what}: nesting too deep") from None
    except ValueError as error:
        raise DeserializationError(f"invalid JSON {what}: {error}") from None


def take(data: WireRecord, key: str) -> Any:
    """Read ``key`` from a decoded object, or :data:`MISSING` when absent."""
    return data.get(key, MISSING)


def wire_error(field: str, message: str) -> DeserializationError:
    """Decoding failure that names the offending field."""
    return DeserializationError(f"{field}: {message}")


def _describe(value: Any) -> str:
    if value is MISSING:
        return "nothing (field absent)"
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (dict, list)):
        return "object" if isinstance(value, dict) else "array"
    return repr(value)


def is_decimal_text(text: str) -> bool:
    """True for plain decimal text (``"12.5"``, ``"-0.000001"``); no exponent or padding."""
    return _DECIMAL.fullmatch(text) is not None


# ── Containers and strings ───────────────────────────────────────────────────


def wire_object(value: Any, field: str) -> WireRecord:
    """A JSON object."""
    if not isinstance(value, dict):
        raise wire_error(field, f"expected object, got {_describe(value)}")
    return value


def wire_array(
    value: Any,
    field: str,
    decode: Callable[[Any, str], T],
    *,
    default_empty: bool = False,
) -> list[T]:
    """A JSON array decoded element by element.

    An absent field is ``[]`` when ``default_empty``; ``null`` is rejected.
    """
    if value is MISSING and default_empty:
        return []
    if not isinstance(value, list):
        raise wire_error(field, f"expected array, got {_describe(value)}")
    return [decode(item, f"{field}[{index}]") for index, item in enumerate(value)]


def wire_str(value: Any, field: str) -> str:
    """A required JSON string (empty allowed)."""
    if not isinstance(value, str):
        raise wire_error(field, f"expected string, got {_describe(value)}")
    return value


def wire_str_or(value: Any, field: str, default: str) -> str:
    """A JSON string, or ``default`` when the field is absent (``null`` is rejected)."""
    return default if value is MISSING else wire_str(value, field)


def wire_opt_str(value: Any, field: str) -> str | None:
    """An optional string; ``null`` or absent is ``None`` (an empty string is kept)."""
    return None if value is MISSING or value is None else wire_str(value, field)


def wire_nonempty_str(value: Any, field: str) -> str | None:
    """An optional string where ``null``, absent, and ``""`` all mean unset.

    The backend sends ``""`` for unset protobuf strings (closure reasons,
    snapshot cursors, global-account market keys).
    """
    text = wire_opt_str(value, field)
    return text if text else None


def wire_pubkey(value: Any, field: str) -> PubkeyStr:
    """A base58 public key string (not decoded; the backend owns canonical form)."""
    return PubkeyStr(wire_str(value, field))


def wire_orderbook_id(value: Any, field: str) -> OrderBookId:
    """An orderbook id string."""
    return OrderBookId(wire_str(value, field))


def wire_bool(value: Any, field: str, default: Any = MISSING) -> bool:
    """A JSON boolean, or ``default`` when given and the field is absent."""
    if value is MISSING and default is not MISSING:
        return bool(default)
    if not isinstance(value, bool):
        raise wire_error(field, f"expected boolean, got {_describe(value)}")
    return value


# ── Integers ─────────────────────────────────────────────────────────────────


def _json_integer(value: Any, field: str) -> int:
    """A JSON integer token; text, floats, and booleans are rejected."""
    if isinstance(value, bool) or not isinstance(value, int):
        raise wire_error(
            field, f"expected an exact JSON integer, got {_describe(value)}"
        )
    return int(value)


def _integer_text(value: Any, field: str, signed: bool) -> int:
    """A JSON integer, or digit-only text (with a leading ``-`` when ``signed``)."""
    if isinstance(value, str):
        pattern = _SIGNED_DIGITS if signed else _DIGITS
        if pattern.fullmatch(value) is None:
            raise wire_error(field, f"invalid integer text {value!r}")
        negative = value.startswith("-")
        digits = value.lstrip("-").lstrip("0") or "0"
        if len(digits) > _MAX_INTEGER_DIGITS:
            # Beyond every wire integer range (and Python's int(str) digit cap).
            raise wire_error(field, f"integer out of range: {value[:48]}…")
        # Parse the significant digits only: leading zeros are valid text.
        parsed = int(digits)
        return -parsed if negative else parsed
    if isinstance(value, bool) or not isinstance(value, int):
        raise wire_error(
            field, f"expected integer text or number, got {_describe(value)}"
        )
    if value < I64_MIN or value > U64_MAX:
        # Rust reads a JSON number as u64 or i64; wider values must be text.
        raise wire_error(field, f"JSON number out of range: {value}")
    return int(value)


def _in_range(parsed: int, field: str, low: int, high: int, label: str) -> int:
    if parsed < low or parsed > high:
        raise wire_error(field, f"integer out of {label} range: {parsed}")
    return parsed


def wire_u64(value: Any, field: str) -> int:
    """An exact ``u64`` sent as decimal text or a JSON number."""
    return _in_range(_integer_text(value, field, False), field, 0, U64_MAX, "u64")


def wire_opt_u64(value: Any, field: str) -> int | None:
    """An optional ``u64``; ``null`` or absent is ``None``."""
    return None if value is MISSING or value is None else wire_u64(value, field)


def wire_u64_number(value: Any, field: str) -> int:
    """A plain ``u64`` JSON number (integer text is rejected)."""
    return _in_range(_json_integer(value, field), field, 0, U64_MAX, "u64")


def wire_i64(value: Any, field: str) -> int:
    """An exact signed ``i64`` sent as a JSON number (integer text is rejected)."""
    return _in_range(_json_integer(value, field), field, I64_MIN, I64_MAX, "i64")


def wire_opt_i64(value: Any, field: str) -> int | None:
    """An optional ``i64``; ``null`` or absent is ``None``."""
    return None if value is MISSING or value is None else wire_i64(value, field)


def wire_i64_or(value: Any, field: str, default: int) -> int:
    """``wire_i64``, or ``default`` when the field is absent (``null`` is rejected)."""
    return default if value is MISSING else wire_i64(value, field)


def wire_i128(value: Any, field: str) -> int:
    """An exact signed ``i128`` sent as decimal text (raw signed atoms) or a number."""
    return _in_range(
        _integer_text(value, field, True), field, I128_MIN, I128_MAX, "i128"
    )


def wire_opt_i128(value: Any, field: str) -> int | None:
    """An optional ``i128``; ``null`` or absent is ``None``."""
    return None if value is MISSING or value is None else wire_i128(value, field)


def wire_int(value: Any, field: str, low: int, high: int) -> int:
    """A small JSON integer such as basis points or an index, within ``[low, high]``.

    Integer text is rejected, as the Rust SDK's plain integers do.
    """
    parsed = _json_integer(value, field)
    if parsed < low or parsed > high:
        raise wire_error(field, f"integer out of range [{low}, {high}]: {parsed}")
    return parsed


def wire_opt_int(value: Any, field: str, low: int, high: int) -> int | None:
    """An optional small integer; ``null`` or absent is ``None``."""
    return (
        None if value is MISSING or value is None else wire_int(value, field, low, high)
    )


def wire_int_or(value: Any, field: str, low: int, high: int, default: int) -> int:
    """``wire_int``, or ``default`` when the field is absent (``null`` is rejected)."""
    return default if value is MISSING else wire_int(value, field, low, high)


I16 = (-(1 << 15), (1 << 15) - 1)
I32 = (-(1 << 31), (1 << 31) - 1)
U8 = (0, 255)
U32 = (0, (1 << 32) - 1)


# ── Decimals ─────────────────────────────────────────────────────────────────


def wire_decimal(value: Any, field: str) -> Decimal:
    """Plain decimal text as an exact ``Decimal``; numbers, exponents, and padding are rejected."""
    text = wire_str(value, field)
    if _DECIMAL.fullmatch(text) is None:
        raise wire_error(field, f"invalid decimal text {text!r}")
    return Decimal(text)


def wire_opt_decimal(value: Any, field: str) -> Decimal | None:
    """An optional decimal; ``null`` or absent is ``None``."""
    return None if value is MISSING or value is None else wire_decimal(value, field)


def wire_decimal_or(value: Any, field: str, default: Decimal) -> Decimal:
    """A decimal, or ``default`` when the field is absent (``null`` is rejected)."""
    return default if value is MISSING else wire_decimal(value, field)


# ── Enums and sides ──────────────────────────────────────────────────────────


def wire_side(value: Any, field: str) -> Side:
    """``Side`` from its text form or the numeric engine encoding of live facts (0 bid, 1 ask)."""
    if not isinstance(value, bool) and isinstance(value, int):
        if value == 0:
            return Side.BID
        if value == 1:
            return Side.ASK
        raise wire_error(field, f"unknown side {value!r}")
    return wire_side_text(value, field)


def wire_side_text(value: Any, field: str) -> Side:
    """A REST side: ``"bid"``/``"ask"`` or ``"buy"``/``"sell"`` only."""
    if isinstance(value, str):
        if value in ("bid", "buy"):
            return Side.BID
        if value in ("ask", "sell"):
            return Side.ASK
    raise wire_error(field, f"unknown side {_describe(value)}")


def wire_enum_or_unknown(value: Any, field: str, cls: type[E], unknown: E) -> E:
    """A string enum value mapped to ``unknown`` when this SDK version does not know it,
    so a new backend value does not fail the surrounding payload."""
    text = wire_str(value, field)
    for member in cls:
        if member is not unknown and member.value == text:
            return member
    return unknown


def wire_known_or_none(value: Any, field: str, cls: type[E]) -> E | None:
    """A string enum value, or ``None`` when it is ``null``, absent, or a value
    this SDK version does not know (such as the backend's ``"unsupported"``)."""
    if value is MISSING or value is None:
        return None
    text = wire_str(value, field)
    for member in cls:
        if member.value == text:
            return member
    return None


def wire_funding_source(value: Any, field: str) -> FundingSource:
    """``FundingSource``, mapping a value this SDK version does not know to ``UNKNOWN``."""
    return wire_enum_or_unknown(value, field, FundingSource, FundingSource.UNKNOWN)


# ── Times ────────────────────────────────────────────────────────────────────


def wire_timestamp_ms(value: Any, field: str) -> datetime:
    """Unix epoch milliseconds (a plain ``u64`` JSON number) as an aware UTC ``datetime``."""
    millis = wire_u64_number(value, field)
    try:
        return _EPOCH + timedelta(milliseconds=millis)
    except OverflowError:
        raise wire_error(field, f"timestamp out of range: {millis}") from None


def wire_rfc3339(value: Any, field: str) -> datetime:
    """An RFC 3339 timestamp with an offset as an aware UTC ``datetime``.

    Fractional seconds beyond microseconds are truncated.
    """
    text = wire_str(value, field)
    match = _RFC3339.fullmatch(text)
    if match is None:
        raise wire_error(field, f"invalid RFC 3339 timestamp {text!r}")
    date, time, fraction, offset = match.groups()
    fraction = (fraction or "")[:7]
    offset = "+00:00" if offset in ("Z", "z") else offset
    try:
        return datetime.fromisoformat(f"{date}T{time}{fraction}{offset}").astimezone(
            UTC
        )
    except (ValueError, OverflowError):
        # Invalid fields, or outside Python's datetime range (years 1-9999).
        raise wire_error(field, f"invalid RFC 3339 timestamp {text!r}") from None


# ── Commit metadata ──────────────────────────────────────────────────────────


def wire_commit_info(data: WireRecord, context: str) -> CommitInfo:
    """The commit metadata flattened into a live committed fact."""
    return CommitInfo(
        effect_id=wire_str(take(data, "effect_id"), f"{context}.effect_id"),
        committed_revision=wire_u64(
            take(data, "committed_revision"), f"{context}.committed_revision"
        ),
        projection_generation=wire_u64(
            take(data, "projection_generation"), f"{context}.projection_generation"
        ),
        actionable=wire_bool(
            take(data, "actionable"), f"{context}.actionable", default=False
        ),
    )


__all__ = [
    "I16",
    "I32",
    "I64_MAX",
    "I64_MIN",
    "I128_MAX",
    "I128_MIN",
    "MISSING",
    "U8",
    "U32",
    "U64_MAX",
    "WireRecord",
    "is_decimal_text",
    "parse_json",
    "take",
    "wire_array",
    "wire_bool",
    "wire_commit_info",
    "wire_decimal",
    "wire_decimal_or",
    "wire_enum_or_unknown",
    "wire_error",
    "wire_funding_source",
    "wire_i64",
    "wire_i64_or",
    "wire_i128",
    "wire_int",
    "wire_int_or",
    "wire_known_or_none",
    "wire_nonempty_str",
    "wire_object",
    "wire_opt_decimal",
    "wire_opt_i64",
    "wire_opt_i128",
    "wire_opt_int",
    "wire_opt_str",
    "wire_opt_u64",
    "wire_orderbook_id",
    "wire_pubkey",
    "wire_rfc3339",
    "wire_side",
    "wire_side_text",
    "wire_str",
    "wire_str_or",
    "wire_timestamp_ms",
    "wire_u64",
    "wire_u64_number",
]
