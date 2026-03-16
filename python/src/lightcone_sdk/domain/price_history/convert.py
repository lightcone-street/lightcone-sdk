"""Price history wire-to-domain conversion."""

from typing import Union

from . import LineData
from .wire import PriceCandle, OrderbookPriceCandle


def line_data_from_candle(candle: Union[PriceCandle, OrderbookPriceCandle]) -> LineData:
    value = candle.m or candle.c or "0"
    return LineData(time=candle.t, value=value)
