"""WebSocket module for the Lightcone SDK."""

import json
from dataclasses import dataclass, field
from enum import Enum, IntEnum
from typing import Any, Callable, Optional, Union


# ---------------------------------------------------------------------------
# Message types (outgoing)
# ---------------------------------------------------------------------------


def ping() -> dict:
    """Create a ping message."""
    return {"method": "ping"}


def subscribe_books(orderbook_ids: list[str]) -> dict:
    """Create a book_update subscribe message."""
    return {
        "method": "subscribe",
        "params": {"type": "book_update", "orderbook_ids": orderbook_ids},
    }


def unsubscribe_books(orderbook_ids: list[str]) -> dict:
    """Create a book_update unsubscribe message."""
    return {
        "method": "unsubscribe",
        "params": {"type": "book_update", "orderbook_ids": orderbook_ids},
    }


def subscribe_trades(orderbook_ids: list[str]) -> dict:
    """Create a trades subscribe message."""
    return {
        "method": "subscribe",
        "params": {"type": "trades", "orderbook_ids": orderbook_ids},
    }


def unsubscribe_trades(orderbook_ids: list[str]) -> dict:
    """Create a trades unsubscribe message."""
    return {
        "method": "unsubscribe",
        "params": {"type": "trades", "orderbook_ids": orderbook_ids},
    }


def subscribe_user(wallet_address: str) -> dict:
    """Create a user subscribe message."""
    return {
        "method": "subscribe",
        "params": {"type": "user", "wallet_address": wallet_address},
    }


def unsubscribe_user(wallet_address: str) -> dict:
    """Create a user unsubscribe message."""
    return {
        "method": "unsubscribe",
        "params": {"type": "user", "wallet_address": wallet_address},
    }


def subscribe_price_history(
    orderbook_id: str,
    resolution: str,
    include_ohlcv: bool = False,
) -> dict:
    """Create a price_history subscribe message."""
    return {
        "method": "subscribe",
        "params": {
            "type": "price_history",
            "orderbook_id": orderbook_id,
            "resolution": resolution,
            "include_ohlcv": include_ohlcv,
        },
    }


def unsubscribe_price_history(orderbook_id: str, resolution: str) -> dict:
    """Create a price_history unsubscribe message."""
    return {
        "method": "unsubscribe",
        "params": {
            "type": "price_history",
            "orderbook_id": orderbook_id,
            "resolution": resolution,
        },
    }


def subscribe_ticker(orderbook_ids: list[str]) -> dict:
    """Create a ticker subscribe message."""
    return {
        "method": "subscribe",
        "params": {"type": "ticker", "orderbook_ids": orderbook_ids},
    }


def unsubscribe_ticker(orderbook_ids: list[str]) -> dict:
    """Create a ticker unsubscribe message."""
    return {
        "method": "unsubscribe",
        "params": {"type": "ticker", "orderbook_ids": orderbook_ids},
    }


def subscribe_market(market_pubkey: str) -> dict:
    """Create a market subscribe message."""
    return {
        "method": "subscribe",
        "params": {"type": "market", "market_pubkey": market_pubkey},
    }


def unsubscribe_market(market_pubkey: str) -> dict:
    """Create a market unsubscribe message."""
    return {
        "method": "unsubscribe",
        "params": {"type": "market", "market_pubkey": market_pubkey},
    }


# ---------------------------------------------------------------------------
# Message types (incoming)
# ---------------------------------------------------------------------------


class MessageInType(str, Enum):
    """Incoming WebSocket message types."""

    BOOK_UPDATE = "book_update"
    USER = "user"
    PONG = "pong"
    ERROR = "error"
    PRICE_HISTORY = "price_history"
    TRADES = "trades"
    AUTH = "auth"
    TICKER = "ticker"
    MARKET = "market"


@dataclass
class MessageIn:
    """Parsed incoming WebSocket message."""

    type: str
    data: Any = None
    version: Optional[float] = None

    @staticmethod
    def from_dict(d: dict) -> "MessageIn":
        return MessageIn(
            type=d.get("type", ""),
            data=d.get("data"),
            version=d.get("version"),
        )


def parse_message_in(text: str) -> MessageIn:
    """Parse a raw WebSocket text message."""
    data = json.loads(text)
    return MessageIn.from_dict(data)


# ---------------------------------------------------------------------------
# WebSocket config
# ---------------------------------------------------------------------------


@dataclass
class WsConfig:
    """WebSocket client configuration."""

    url: str = "wss://tws.lightcone.xyz/ws"
    reconnect: bool = True
    max_reconnect_attempts: int = 10
    base_reconnect_delay_ms: int = 1000
    ping_interval_ms: int = 30_000
    pong_timeout_ms: int = 1_000


WS_DEFAULT_CONFIG = WsConfig()


# ---------------------------------------------------------------------------
# Events
# ---------------------------------------------------------------------------


class WsEventType(str, Enum):
    """WebSocket event types."""

    MESSAGE = "message"
    CONNECTED = "connected"
    DISCONNECTED = "disconnected"
    ERROR = "error"
    MAX_RECONNECT_REACHED = "max_reconnect_reached"


@dataclass
class WsEvent:
    """WebSocket event."""

    type: WsEventType
    message: Optional[MessageIn] = None
    error: Optional[str] = None
    code: Optional[int] = None
    reason: str = ""


# ---------------------------------------------------------------------------
# ReadyState
# ---------------------------------------------------------------------------


class ReadyState(IntEnum):
    """WebSocket ready state."""

    CONNECTING = 0
    OPEN = 1
    CLOSING = 2
    CLOSED = 3


# ---------------------------------------------------------------------------
# WsError type (message-level, not exception)
# ---------------------------------------------------------------------------


@dataclass
class WsErrorData:
    """Error data from WebSocket error message."""

    error: str = ""
    code: Optional[str] = None
    orderbook_id: Optional[str] = None
    wallet_address: Optional[str] = None
    hint: Optional[str] = None
    details: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "WsErrorData":
        return WsErrorData(
            error=d.get("error", ""),
            code=d.get("code"),
            orderbook_id=d.get("orderbook_id"),
            wallet_address=d.get("wallet_address"),
            hint=d.get("hint"),
            details=d.get("details"),
        )


__all__ = [
    # Outgoing message helpers
    "ping",
    "subscribe_books",
    "unsubscribe_books",
    "subscribe_trades",
    "unsubscribe_trades",
    "subscribe_user",
    "unsubscribe_user",
    "subscribe_price_history",
    "unsubscribe_price_history",
    "subscribe_ticker",
    "unsubscribe_ticker",
    "subscribe_market",
    "unsubscribe_market",
    # Incoming message types
    "MessageInType",
    "MessageIn",
    "parse_message_in",
    # Config
    "WsConfig",
    "WS_DEFAULT_CONFIG",
    # Events
    "WsEventType",
    "WsEvent",
    # ReadyState
    "ReadyState",
    # Error data
    "WsErrorData",
]
