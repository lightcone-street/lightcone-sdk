"""Trade wire-to-domain conversion."""

from __future__ import annotations

from datetime import UTC, datetime, timedelta
from decimal import Decimal

from ...error import ValidationError
from ...shared.wire_decode import is_decimal_text
from . import Trade
from .wire import TradeResponseWire, WsTrade

_EPOCH = datetime(1970, 1, 1, tzinfo=UTC)


def trade_from_wire(wire: TradeResponseWire) -> Trade:
    """Convert a REST trade; raises ``ValidationError`` on a price, size, or
    ``executed_at`` that is not a valid value."""

    def invalid(name: str, value: object) -> ValidationError:
        return ValidationError(f"trade {wire.trade_id}: invalid {name} {value!r}")

    try:
        timestamp = _EPOCH + timedelta(milliseconds=wire.executed_at)
    except OverflowError:
        raise invalid("executed_at", wire.executed_at) from None
    if not is_decimal_text(wire.price):
        raise invalid("price", wire.price)
    if not is_decimal_text(wire.size):
        raise invalid("size", wire.size)
    return Trade(
        orderbook_id=wire.orderbook_id,
        trade_id=wire.trade_id,
        cursor_id=wire.id,
        timestamp=timestamp,
        price=Decimal(wire.price),
        size=Decimal(wire.size),
        side=wire.side,
    )


def trade_from_ws(ws: WsTrade) -> Trade:
    """Convert a committed fill fact; raises ``ValidationError`` when the fill
    has no base amount to price."""
    trade_id = ws.trade_id()
    price = ws.price()
    if price is None:
        raise ValidationError(f"trade {trade_id}: zero base amount has no price")
    return Trade(
        orderbook_id=ws.orderbook_id,
        trade_id=trade_id,
        cursor_id=None,
        timestamp=ws.executed_at,
        price=price,
        size=ws.base_amount,
        side=ws.taker_side,
    )


__all__ = ["trade_from_wire", "trade_from_ws"]
