"""Position wire types."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class PositionOutcomeWire:
    conditional_token: str
    balance_idle: int = 0
    balance_on_book: int = 0
    name: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "PositionOutcomeWire":
        return PositionOutcomeWire(
            conditional_token=d.get("conditional_token", ""),
            balance_idle=d.get("balance_idle", 0),
            balance_on_book=d.get("balance_on_book", 0),
            name=d.get("name"),
        )


@dataclass
class PositionEntryWire:
    id: str
    owner: str
    market_pubkey: str
    outcomes: list[PositionOutcomeWire] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "PositionEntryWire":
        return PositionEntryWire(
            id=d.get("id", ""),
            owner=d.get("owner", ""),
            market_pubkey=d.get("market_pubkey", ""),
            outcomes=[PositionOutcomeWire.from_dict(o) for o in d.get("outcomes", [])],
        )


@dataclass
class PositionsResponseWire:
    positions: list[PositionEntryWire] = field(default_factory=list)
    decimals: Optional[dict] = None

    @staticmethod
    def from_dict(d: dict) -> "PositionsResponseWire":
        return PositionsResponseWire(
            positions=[PositionEntryWire.from_dict(p) for p in d.get("positions", [])],
            decimals=d.get("decimals"),
        )


@dataclass
class MarketPositionsResponseWire:
    positions: list[PositionEntryWire] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "MarketPositionsResponseWire":
        return MarketPositionsResponseWire(
            positions=[PositionEntryWire.from_dict(p) for p in d.get("positions", [])],
        )
