"""State container for websocket deposit-price data."""

from dataclasses import dataclass, field
from typing import Optional

from . import LatestDepositPrice
from .wire import DepositTokenCandle


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


__all__ = ["DepositPriceState"]
