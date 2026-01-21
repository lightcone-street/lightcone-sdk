"""Position-related types for the Lightcone REST API."""

from dataclasses import dataclass

from ..error import DeserializeError


@dataclass
class OutcomeBalance:
    """Outcome balance in a position."""

    outcome_index: int
    conditional_token: str
    balance: str
    balance_idle: str
    balance_on_book: str

    @classmethod
    def from_dict(cls, data: dict) -> "OutcomeBalance":
        try:
            return cls(
                outcome_index=data["outcome_index"],
                conditional_token=data["conditional_token"],
                balance=data["balance"],
                balance_idle=data["balance_idle"],
                balance_on_book=data["balance_on_book"],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in OutcomeBalance: {e}")


@dataclass
class Position:
    """User position in a market."""

    id: int
    position_pubkey: str
    owner: str
    market_pubkey: str
    outcomes: list[OutcomeBalance]
    created_at: str
    updated_at: str

    @classmethod
    def from_dict(cls, data: dict) -> "Position":
        try:
            return cls(
                id=data["id"],
                position_pubkey=data["position_pubkey"],
                owner=data["owner"],
                market_pubkey=data["market_pubkey"],
                outcomes=[OutcomeBalance.from_dict(o) for o in data.get("outcomes", [])],
                created_at=data["created_at"],
                updated_at=data["updated_at"],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in Position: {e}")


@dataclass
class PositionsResponse:
    """Response for GET /api/users/{user_pubkey}/positions."""

    owner: str
    total_markets: int
    positions: list[Position]

    @classmethod
    def from_dict(cls, data: dict) -> "PositionsResponse":
        try:
            return cls(
                owner=data["owner"],
                total_markets=data["total_markets"],
                positions=[Position.from_dict(p) for p in data.get("positions", [])],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in PositionsResponse: {e}")


@dataclass
class MarketPositionsResponse:
    """Response for GET /api/users/{user_pubkey}/markets/{market_pubkey}/positions."""

    owner: str
    market_pubkey: str
    positions: list[Position]

    @classmethod
    def from_dict(cls, data: dict) -> "MarketPositionsResponse":
        try:
            return cls(
                owner=data["owner"],
                market_pubkey=data["market_pubkey"],
                positions=[Position.from_dict(p) for p in data.get("positions", [])],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in MarketPositionsResponse: {e}")
