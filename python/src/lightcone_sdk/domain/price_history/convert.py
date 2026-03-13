"""Price history wire-to-domain conversion."""

from . import LineData
from .wire import PriceCandle


def line_data_from_candle(candle: PriceCandle) -> LineData:
    value = candle.m or candle.c or "0"
    return LineData(time=candle.t, value=value)
