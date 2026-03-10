"""Position wire types."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class VaultBalance:
    """Vault balance wire type."""
    mint: str = ""
    amount: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "VaultBalance":
        return VaultBalance(
            mint=d.get("mint", ""),
            amount=str(d.get("amount", "0")),
        )


@dataclass
class PositionOutcomeWire:
    conditional_token: str
    outcome_index: int = 0
    balance_idle: int = 0
    balance_on_book: int = 0
    balance: str = "0"
    name: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "PositionOutcomeWire":
        return PositionOutcomeWire(
            conditional_token=d.get("conditional_token", ""),
            outcome_index=d.get("outcome_index", 0),
            balance_idle=d.get("balance_idle", 0),
            balance_on_book=d.get("balance_on_book", 0),
            balance=str(d.get("balance", "0")),
            name=d.get("name"),
        )


@dataclass
class PositionEntryWire:
    id: str
    owner: str
    market_pubkey: str
    position_pubkey: str = ""
    outcomes: list[PositionOutcomeWire] = field(default_factory=list)
    vault_balances: list[VaultBalance] = field(default_factory=list)
    created_at: Optional[str] = None
    updated_at: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "PositionEntryWire":
        return PositionEntryWire(
            id=d.get("id", ""),
            owner=d.get("owner", ""),
            market_pubkey=d.get("market_pubkey", ""),
            position_pubkey=d.get("position_pubkey", ""),
            outcomes=[PositionOutcomeWire.from_dict(o) for o in d.get("outcomes", [])],
            vault_balances=[VaultBalance.from_dict(v) for v in d.get("vault_balances", [])],
            created_at=d.get("created_at"),
            updated_at=d.get("updated_at"),
        )


@dataclass
class PositionsResponseWire:
    positions: list[PositionEntryWire] = field(default_factory=list)
    owner: str = ""
    total_markets: int = 0
    decimals: dict[str, int] = field(default_factory=dict)

    @staticmethod
    def from_dict(d: dict) -> "PositionsResponseWire":
        return PositionsResponseWire(
            positions=[PositionEntryWire.from_dict(p) for p in d.get("positions", [])],
            owner=d.get("owner", ""),
            total_markets=d.get("total_markets", 0),
            decimals=d.get("decimals") or {},
        )


@dataclass
class MarketPositionsResponseWire:
    positions: list[PositionEntryWire] = field(default_factory=list)
    owner: str = ""
    market_pubkey: str = ""
    decimals: dict[str, int] = field(default_factory=dict)

    @staticmethod
    def from_dict(d: dict) -> "MarketPositionsResponseWire":
        return MarketPositionsResponseWire(
            positions=[PositionEntryWire.from_dict(p) for p in d.get("positions", [])],
            owner=d.get("owner", ""),
            market_pubkey=d.get("market_pubkey", ""),
            decimals=d.get("decimals") or {},
        )
