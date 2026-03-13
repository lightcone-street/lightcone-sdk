"""Wire types for the `deposit_price` websocket channel."""

from dataclasses import dataclass, field


@dataclass
class DepositTokenCandle:
    t: int = 0
    tc: int = 0
    c: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "DepositTokenCandle":
        return DepositTokenCandle(
            t=d.get("t", 0),
            tc=d.get("tc", 0),
            c=str(d.get("c", "0")),
        )


@dataclass
class DepositPriceSnapshot:
    deposit_asset: str = ""
    resolution: str = "1m"
    prices: list[DepositTokenCandle] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "DepositPriceSnapshot":
        return DepositPriceSnapshot(
            deposit_asset=d.get("deposit_asset", ""),
            resolution=d.get("resolution", "1m"),
            prices=[DepositTokenCandle.from_dict(c) for c in d.get("prices", [])],
        )


@dataclass
class DepositPriceTick:
    deposit_asset: str = ""
    price: str = "0"
    event_time: int = 0

    @staticmethod
    def from_dict(d: dict) -> "DepositPriceTick":
        return DepositPriceTick(
            deposit_asset=d.get("deposit_asset", ""),
            price=str(d.get("price", "0")),
            event_time=d.get("event_time", 0),
        )


@dataclass
class DepositPriceCandleUpdate:
    deposit_asset: str = ""
    resolution: str = "1m"
    t: int = 0
    tc: int = 0
    c: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "DepositPriceCandleUpdate":
        return DepositPriceCandleUpdate(
            deposit_asset=d.get("deposit_asset", ""),
            resolution=d.get("resolution", "1m"),
            t=d.get("t", 0),
            tc=d.get("tc", 0),
            c=str(d.get("c", "0")),
        )


__all__ = [
    "DepositTokenCandle",
    "DepositPriceSnapshot",
    "DepositPriceTick",
    "DepositPriceCandleUpdate",
]
