"""Position wire types."""

from dataclasses import dataclass, field
from typing import Optional

from ...error import _require


@dataclass
class GlobalDeposit:
    """Global deposit balance for a deposit mint."""
    deposit_mint: str = ""
    symbol: str = ""
    balance: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "GlobalDeposit":
        return GlobalDeposit(
            deposit_mint=d.get("deposit_mint", ""),
            symbol=d.get("symbol", ""),
            balance=str(d.get("balance", "0")),
        )


@dataclass
class PositionOutcomeWire:
    conditional_token: str
    outcome_index: int = 0
    balance: str = "0"
    balance_idle: str = "0"
    balance_on_book: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "PositionOutcomeWire":
        return PositionOutcomeWire(
            conditional_token=_require(d, "conditional_token", "PositionOutcomeWire"),
            outcome_index=d.get("outcome_index", 0),
            balance=str(d.get("balance", "0")),
            balance_idle=str(d.get("balance_idle", "0")),
            balance_on_book=str(d.get("balance_on_book", "0")),
        )


@dataclass
class VaultBalance:
    """Vault balance for a deposit mint within a position."""
    deposit_mint: str = ""
    vault: str = ""
    balance: str = "0"

    @staticmethod
    def from_dict(d: dict) -> "VaultBalance":
        return VaultBalance(
            deposit_mint=d.get("deposit_mint", ""),
            vault=d.get("vault", ""),
            balance=str(d.get("balance", "0")),
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
            id=_require(d, "id", "PositionEntryWire"),
            owner=_require(d, "owner", "PositionEntryWire"),
            market_pubkey=_require(d, "market_pubkey", "PositionEntryWire"),
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
    global_deposits: list[GlobalDeposit] = field(default_factory=list)
    decimals: dict[str, int] = field(default_factory=dict)

    @staticmethod
    def from_dict(d: dict) -> "PositionsResponseWire":
        return PositionsResponseWire(
            positions=[PositionEntryWire.from_dict(p) for p in d.get("positions", [])],
            owner=d.get("owner", ""),
            total_markets=d.get("total_markets", 0),
            global_deposits=[GlobalDeposit.from_dict(g) for g in d.get("global_deposits", [])],
            decimals=d.get("decimals") or {},
        )


@dataclass
class MarketPositionsResponseWire:
    positions: list[PositionEntryWire] = field(default_factory=list)
    owner: str = ""
    market_pubkey: str = ""
    global_deposits: list[GlobalDeposit] = field(default_factory=list)
    decimals: dict[str, int] = field(default_factory=dict)

    @staticmethod
    def from_dict(d: dict) -> "MarketPositionsResponseWire":
        return MarketPositionsResponseWire(
            positions=[PositionEntryWire.from_dict(p) for p in d.get("positions", [])],
            owner=d.get("owner", ""),
            market_pubkey=d.get("market_pubkey", ""),
            global_deposits=[GlobalDeposit.from_dict(g) for g in d.get("global_deposits", [])],
            decimals=d.get("decimals") or {},
        )
