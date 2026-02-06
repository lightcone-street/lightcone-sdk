"""Decimals-related types for the Lightcone REST API."""

from dataclasses import dataclass

from ..error import DeserializeError


@dataclass
class DecimalsResponse:
    """Response for GET /api/orderbook/{id}/decimals."""

    orderbook_id: str
    base_decimals: int
    quote_decimals: int
    price_decimals: int

    @classmethod
    def from_dict(cls, data: dict) -> "DecimalsResponse":
        try:
            return cls(
                orderbook_id=data["orderbook_id"],
                base_decimals=data["base_decimals"],
                quote_decimals=data["quote_decimals"],
                price_decimals=data["price_decimals"],
            )
        except KeyError as e:
            raise DeserializeError(f"Missing required field in DecimalsResponse: {e}")
