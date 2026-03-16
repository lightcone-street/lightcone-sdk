"""Price history state container."""

from dataclasses import dataclass, field
from typing import Optional

from . import LatestDepositPrice, LineData
from .wire import DepositTokenCandle


@dataclass
class PriceHistoryState:
    """State for price history data keyed by (orderbook_id, resolution)."""
    _data: dict[tuple[str, str], list[LineData]] = field(default_factory=dict)

    def key(self, orderbook_id: str, resolution: str) -> tuple[str, str]:
        return (orderbook_id, resolution)

    def apply_snapshot(self, orderbook_id: str, resolution: str, prices: list[LineData]) -> None:
        """Replace all data for the given key."""
        self._data[self.key(orderbook_id, resolution)] = list(prices)

    def apply_update(self, orderbook_id: str, resolution: str, point: LineData) -> None:
        """Append or overwrite the last point if timestamps match."""
        k = self.key(orderbook_id, resolution)
        if k not in self._data:
            self._data[k] = []
        series = self._data[k]
        if series and series[-1].time == point.time:
            series[-1] = point
        else:
            series.append(point)

    def set(self, orderbook_id: str, resolution: str, data: list[LineData]) -> None:
        """Alias for apply_snapshot."""
        self.apply_snapshot(orderbook_id, resolution, data)

    def add(self, orderbook_id: str, resolution: str, point: LineData) -> None:
        """Alias for apply_update."""
        self.apply_update(orderbook_id, resolution, point)

    def get(self, orderbook_id: str, resolution: str) -> list[LineData]:
        return self._data.get(self.key(orderbook_id, resolution), [])

    def clear(self, orderbook_id: Optional[str] = None, resolution: Optional[str] = None) -> None:
        if orderbook_id and resolution:
            self._data.pop(self.key(orderbook_id, resolution), None)
        else:
            self._data.clear()


@dataclass
class DepositPriceState:
    """State for deposit-price data keyed by (deposit_asset, resolution)."""

    _candles: dict[tuple[str, str], list[DepositTokenCandle]] = field(default_factory=dict)
    _latest_price: dict[str, LatestDepositPrice] = field(default_factory=dict)

    def key(self, deposit_asset: str, resolution: str) -> tuple[str, str]:
        return (deposit_asset, resolution)

    def apply_snapshot(
        self,
        deposit_asset: str,
        resolution: str,
        prices: list[DepositTokenCandle],
    ) -> None:
        self._candles[self.key(deposit_asset, resolution)] = list(prices)

    def apply_candle_update(
        self,
        deposit_asset: str,
        resolution: str,
        candle: DepositTokenCandle,
    ) -> None:
        key = self.key(deposit_asset, resolution)
        if key not in self._candles:
            self._candles[key] = []

        series = self._candles[key]
        if series and series[-1].t == candle.t:
            series[-1].tc = candle.tc
            series[-1].c = candle.c
        else:
            series.append(candle)

    def apply_price_tick(self, deposit_asset: str, price: str, event_time: int) -> None:
        self._latest_price[deposit_asset] = LatestDepositPrice(
            price=price,
            event_time=event_time,
        )

    def get_candles(self, deposit_asset: str, resolution: str) -> list[DepositTokenCandle]:
        return self._candles.get(self.key(deposit_asset, resolution), [])

    def get_latest_price(self, deposit_asset: str) -> Optional[LatestDepositPrice]:
        return self._latest_price.get(deposit_asset)

    def clear(self) -> None:
        self._candles.clear()
        self._latest_price.clear()


__all__ = ["PriceHistoryState", "DepositPriceState"]
