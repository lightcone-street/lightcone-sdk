"""Orderbook wire-to-domain conversion."""

from .wire import DecimalsResponse
from ...shared.scaling import OrderbookDecimals


def decimals_from_wire(wire: DecimalsResponse) -> OrderbookDecimals:
    return OrderbookDecimals(
        base_decimals=wire.base_decimals,
        quote_decimals=wire.quote_decimals,
        price_decimals=wire.price_decimals,
    )
