"""Trade-related types for the Lightcone REST API."""

from dataclasses import dataclass
from enum import Enum
from typing import Optional

from ..error import DeserializeError


class ApiTradeSide(str, Enum):
    """Trade side enum (serializes as UPPERCASE string)."""

    BID = "BID"
    ASK = "ASK"


@dataclass
class Trade:
    """Executed trade information."""

    id: int
    orderbook_id: str
    taker_pubkey: str
    maker_pubkey: str
    side: ApiTradeSide
    size: str
    price: str
    taker_fee: str
    maker_fee: str
    executed_at: int

    @classmethod
    def from_dict(cls, data: dict) -> "Trade":
        try:
            # Parse side as enum
            side_str = data["side"].upper()
            side = ApiTradeSide(side_str)

            return cls(
                id=data["id"],
                orderbook_id=data["orderbook_id"],
                taker_pubkey=data["taker_pubkey"],
                maker_pubkey=data["maker_pubkey"],
                side=side,
                size=data["size"],
                price=data["price"],
                taker_fee=data["taker_fee"],
                maker_fee=data["maker_fee"],
                executed_at=data["executed_at"],
            )
        except (KeyError, ValueError) as e:
            raise DeserializeError(f"Invalid field in Trade: {e}")


@dataclass
class TradesParams:
    """Query parameters for GET /api/trades."""

    orderbook_id: str
    user_pubkey: Optional[str] = None
    from_timestamp: Optional[int] = None
    to_timestamp: Optional[int] = None
    cursor: Optional[int] = None
    limit: Optional[int] = None

    @classmethod
    def new(cls, orderbook_id: str) -> "TradesParams":
        """Create new params with required orderbook_id."""
        return cls(orderbook_id=orderbook_id)

    def with_user(self, user_pubkey: str) -> "TradesParams":
        """Set user pubkey filter."""
        self.user_pubkey = user_pubkey
        return self

    def with_time_range(self, from_ts: int, to_ts: int) -> "TradesParams":
        """Set time range."""
        self.from_timestamp = from_ts
        self.to_timestamp = to_ts
        return self

    def with_cursor(self, cursor: int) -> "TradesParams":
        """Set pagination cursor."""
        self.cursor = cursor
        return self

    def with_limit(self, limit: int) -> "TradesParams":
        """Set result limit."""
        self.limit = limit
        return self

    def to_query_params(self) -> dict:
        """Convert to query parameters dict."""
        params = {"orderbook_id": self.orderbook_id}
        if self.user_pubkey is not None:
            params["user_pubkey"] = self.user_pubkey
        if self.from_timestamp is not None:
            params["from"] = self.from_timestamp
        if self.to_timestamp is not None:
            params["to"] = self.to_timestamp
        if self.cursor is not None:
            params["cursor"] = self.cursor
        if self.limit is not None:
            params["limit"] = self.limit
        return params


@dataclass
class TradesResponse:
    """Response for GET /api/trades."""

    orderbook_id: str
    trades: list[Trade]
    has_more: bool
    next_cursor: Optional[int] = None

    @classmethod
    def from_dict(cls, data: dict) -> "TradesResponse":
        try:
            return cls(
                orderbook_id=data["orderbook_id"],
                trades=[Trade.from_dict(t) for t in data.get("trades", [])],
                has_more=data["has_more"],
                next_cursor=data.get("next_cursor"),
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in TradesResponse: {e}")
