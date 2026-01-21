"""Admin-related types for the Lightcone REST API."""

from dataclasses import dataclass
from typing import Optional


@dataclass
class AdminResponse:
    """Response for GET /api/admin/test."""

    status: str
    message: str

    @classmethod
    def from_dict(cls, data: dict) -> "AdminResponse":
        return cls(
            status=data["status"],
            message=data["message"],
        )


@dataclass
class CreateOrderbookRequest:
    """Request for POST /api/admin/create-orderbook."""

    market_pubkey: str
    base_token: str
    quote_token: str
    tick_size: Optional[int] = None

    @classmethod
    def new(
        cls,
        market_pubkey: str,
        base_token: str,
        quote_token: str,
    ) -> "CreateOrderbookRequest":
        """Create a new request with required fields."""
        return cls(
            market_pubkey=market_pubkey,
            base_token=base_token,
            quote_token=quote_token,
        )

    def with_tick_size(self, tick_size: int) -> "CreateOrderbookRequest":
        """Set custom tick size."""
        self.tick_size = tick_size
        return self

    def to_dict(self) -> dict:
        d = {
            "market_pubkey": self.market_pubkey,
            "base_token": self.base_token,
            "quote_token": self.quote_token,
        }
        if self.tick_size is not None:
            d["tick_size"] = self.tick_size
        return d


@dataclass
class CreateOrderbookResponse:
    """Response for POST /api/admin/create-orderbook."""

    status: str
    orderbook_id: str
    market_pubkey: str
    base_token: str
    quote_token: str
    tick_size: int
    message: str

    @classmethod
    def from_dict(cls, data: dict) -> "CreateOrderbookResponse":
        return cls(
            status=data["status"],
            orderbook_id=data["orderbook_id"],
            market_pubkey=data["market_pubkey"],
            base_token=data["base_token"],
            quote_token=data["quote_token"],
            tick_size=data["tick_size"],
            message=data["message"],
        )
