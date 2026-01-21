"""Message types for the Lightcone WebSocket protocol."""

from dataclasses import dataclass, field
from enum import Enum, IntEnum
from typing import Optional, Any, Union

# ============================================================================
# REQUEST TYPES (Client -> Server)
# ============================================================================


@dataclass
class WsRequest:
    """Subscribe/Unsubscribe request wrapper."""

    method: str
    params: Optional[dict] = None

    @classmethod
    def subscribe(cls, params: dict) -> "WsRequest":
        """Create a subscribe request."""
        return cls(method="subscribe", params=params)

    @classmethod
    def unsubscribe(cls, params: dict) -> "WsRequest":
        """Create an unsubscribe request."""
        return cls(method="unsubscribe", params=params)

    @classmethod
    def ping(cls) -> "WsRequest":
        """Create a ping request."""
        return cls(method="ping")

    def to_dict(self) -> dict:
        d = {"method": self.method}
        if self.params is not None:
            d["params"] = self.params
        return d


class SubscribeType(str, Enum):
    """Subscription types."""

    BOOK_UPDATE = "book_update"
    TRADES = "trades"
    USER = "user"
    PRICE_HISTORY = "price_history"
    MARKET = "market"


def book_update_params(orderbook_ids: list[str]) -> dict:
    """Create book update subscription params."""
    return {"type": "book_update", "orderbook_ids": orderbook_ids}


def trades_params(orderbook_ids: list[str]) -> dict:
    """Create trades subscription params."""
    return {"type": "trades", "orderbook_ids": orderbook_ids}


def user_params(user: str) -> dict:
    """Create user subscription params."""
    return {"type": "user", "user": user}


def price_history_params(
    orderbook_id: str,
    resolution: str,
    include_ohlcv: bool = False,
) -> dict:
    """Create price history subscription params."""
    return {
        "type": "price_history",
        "orderbook_id": orderbook_id,
        "resolution": resolution,
        "include_ohlcv": include_ohlcv,
    }


def market_params(market_pubkey: str) -> dict:
    """Create market subscription params."""
    return {"type": "market", "market_pubkey": market_pubkey}


# ============================================================================
# RESPONSE TYPES (Server -> Client)
# ============================================================================


@dataclass
class RawWsMessage:
    """Raw message wrapper for initial parsing."""

    type_: str
    version: float
    data: Any

    @classmethod
    def from_dict(cls, data: dict) -> "RawWsMessage":
        return cls(
            type_=data.get("type", ""),
            version=data.get("version", 0.1),
            data=data.get("data", {}),
        )


# ============================================================================
# BOOK UPDATE TYPES
# ============================================================================


@dataclass
class PriceLevel:
    """Price level in the orderbook."""

    side: str
    price: str
    size: str

    @classmethod
    def from_dict(cls, data: dict) -> "PriceLevel":
        return cls(
            side=data["side"],
            price=data["price"],
            size=data["size"],
        )


@dataclass
class BookUpdateData:
    """Orderbook snapshot/delta data."""

    orderbook_id: str
    timestamp: str = ""
    seq: int = 0
    bids: list[PriceLevel] = field(default_factory=list)
    asks: list[PriceLevel] = field(default_factory=list)
    is_snapshot: bool = False
    resync: bool = False
    message: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "BookUpdateData":
        return cls(
            orderbook_id=data["orderbook_id"],
            timestamp=data.get("timestamp", ""),
            seq=data.get("seq", 0),
            bids=[PriceLevel.from_dict(b) for b in data.get("bids", [])],
            asks=[PriceLevel.from_dict(a) for a in data.get("asks", [])],
            is_snapshot=data.get("is_snapshot", False),
            resync=data.get("resync", False),
            message=data.get("message"),
        )


# ============================================================================
# TRADE TYPES
# ============================================================================


@dataclass
class TradeData:
    """Trade execution data."""

    orderbook_id: str
    price: str
    size: str
    side: str
    timestamp: str
    trade_id: str

    @classmethod
    def from_dict(cls, data: dict) -> "TradeData":
        return cls(
            orderbook_id=data["orderbook_id"],
            price=data["price"],
            size=data["size"],
            side=data["side"],
            timestamp=data["timestamp"],
            trade_id=data["trade_id"],
        )


# ============================================================================
# USER EVENT TYPES
# ============================================================================


@dataclass
class OutcomeBalance:
    """Individual outcome balance."""

    outcome_index: int
    mint: str
    idle: str
    on_book: str

    @classmethod
    def from_dict(cls, data: dict) -> "OutcomeBalance":
        return cls(
            outcome_index=data["outcome_index"],
            mint=data["mint"],
            idle=data["idle"],
            on_book=data["on_book"],
        )


@dataclass
class Balance:
    """Balance containing outcome balances."""

    outcomes: list[OutcomeBalance]

    @classmethod
    def from_dict(cls, data: dict) -> "Balance":
        return cls(
            outcomes=[OutcomeBalance.from_dict(o) for o in data.get("outcomes", [])],
        )


@dataclass
class BalanceEntry:
    """Balance entry from user snapshot."""

    market_pubkey: str
    deposit_mint: str
    outcomes: list[OutcomeBalance]

    @classmethod
    def from_dict(cls, data: dict) -> "BalanceEntry":
        return cls(
            market_pubkey=data["market_pubkey"],
            deposit_mint=data["deposit_mint"],
            outcomes=[OutcomeBalance.from_dict(o) for o in data.get("outcomes", [])],
        )


@dataclass
class Order:
    """User order from snapshot."""

    order_hash: str
    market_pubkey: str
    orderbook_id: str
    side: int
    maker_amount: str
    taker_amount: str
    remaining: str
    filled: str
    price: str
    created_at: int
    expiration: int

    @classmethod
    def from_dict(cls, data: dict) -> "Order":
        return cls(
            order_hash=data["order_hash"],
            market_pubkey=data["market_pubkey"],
            orderbook_id=data["orderbook_id"],
            side=data["side"],
            maker_amount=data["maker_amount"],
            taker_amount=data["taker_amount"],
            remaining=data["remaining"],
            filled=data["filled"],
            price=data["price"],
            created_at=data["created_at"],
            expiration=data["expiration"],
        )


@dataclass
class OrderUpdate:
    """Order update from real-time event."""

    order_hash: str
    price: str
    fill_amount: str
    remaining: str
    filled: str
    side: int
    is_maker: bool
    created_at: int
    balance: Optional[Balance] = None

    @classmethod
    def from_dict(cls, data: dict) -> "OrderUpdate":
        balance = None
        if "balance" in data and data["balance"]:
            balance = Balance.from_dict(data["balance"])
        return cls(
            order_hash=data["order_hash"],
            price=data["price"],
            fill_amount=data["fill_amount"],
            remaining=data["remaining"],
            filled=data["filled"],
            side=data["side"],
            is_maker=data["is_maker"],
            created_at=data["created_at"],
            balance=balance,
        )


@dataclass
class UserEventData:
    """User event data (snapshot, order_update, balance_update)."""

    event_type: str
    orders: list[Order] = field(default_factory=list)
    balances: dict[str, BalanceEntry] = field(default_factory=dict)
    order: Optional[OrderUpdate] = None
    balance: Optional[Balance] = None
    market_pubkey: Optional[str] = None
    orderbook_id: Optional[str] = None
    deposit_mint: Optional[str] = None
    timestamp: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "UserEventData":
        orders = [Order.from_dict(o) for o in data.get("orders", [])]
        balances = {
            k: BalanceEntry.from_dict(v) for k, v in data.get("balances", {}).items()
        }
        order = OrderUpdate.from_dict(data["order"]) if data.get("order") else None
        balance = Balance.from_dict(data["balance"]) if data.get("balance") else None

        return cls(
            event_type=data["event_type"],
            orders=orders,
            balances=balances,
            order=order,
            balance=balance,
            market_pubkey=data.get("market_pubkey"),
            orderbook_id=data.get("orderbook_id"),
            deposit_mint=data.get("deposit_mint"),
            timestamp=data.get("timestamp"),
        )


# ============================================================================
# PRICE HISTORY TYPES
# ============================================================================


@dataclass
class Candle:
    """OHLCV candle data."""

    t: int  # Timestamp (Unix ms)
    o: Optional[str] = None  # Open
    h: Optional[str] = None  # High
    l: Optional[str] = None  # Low
    c: Optional[str] = None  # Close
    v: Optional[str] = None  # Volume
    m: Optional[str] = None  # Midpoint
    bb: Optional[str] = None  # Best bid
    ba: Optional[str] = None  # Best ask

    @classmethod
    def from_dict(cls, data: dict) -> "Candle":
        return cls(
            t=data["t"],
            o=data.get("o"),
            h=data.get("h"),
            l=data.get("l"),
            c=data.get("c"),
            v=data.get("v"),
            m=data.get("m"),
            bb=data.get("bb"),
            ba=data.get("ba"),
        )


@dataclass
class PriceHistoryData:
    """Price history data (snapshot, update, heartbeat)."""

    event_type: str
    orderbook_id: Optional[str] = None
    resolution: Optional[str] = None
    include_ohlcv: Optional[bool] = None
    prices: list[Candle] = field(default_factory=list)
    last_timestamp: Optional[int] = None
    server_time: Optional[int] = None
    last_processed: Optional[int] = None
    # For updates (inline candle data)
    t: Optional[int] = None
    o: Optional[str] = None
    h: Optional[str] = None
    l: Optional[str] = None
    c: Optional[str] = None
    v: Optional[str] = None
    m: Optional[str] = None
    bb: Optional[str] = None
    ba: Optional[str] = None

    def to_candle(self) -> Optional[Candle]:
        """Convert inline candle data to a Candle struct (for update events)."""
        if self.t is None:
            return None
        return Candle(
            t=self.t,
            o=self.o,
            h=self.h,
            l=self.l,
            c=self.c,
            v=self.v,
            m=self.m,
            bb=self.bb,
            ba=self.ba,
        )

    @classmethod
    def from_dict(cls, data: dict) -> "PriceHistoryData":
        prices = [Candle.from_dict(p) for p in data.get("prices", [])]
        return cls(
            event_type=data["event_type"],
            orderbook_id=data.get("orderbook_id"),
            resolution=data.get("resolution"),
            include_ohlcv=data.get("include_ohlcv"),
            prices=prices,
            last_timestamp=data.get("last_timestamp"),
            server_time=data.get("server_time"),
            last_processed=data.get("last_processed"),
            t=data.get("t"),
            o=data.get("o"),
            h=data.get("h"),
            l=data.get("l"),
            c=data.get("c"),
            v=data.get("v"),
            m=data.get("m"),
            bb=data.get("bb"),
            ba=data.get("ba"),
        )


# ============================================================================
# MARKET EVENT TYPES
# ============================================================================


@dataclass
class MarketEventData:
    """Market event data."""

    event_type: str
    market_pubkey: str
    timestamp: str
    orderbook_id: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "MarketEventData":
        return cls(
            event_type=data["event_type"],
            market_pubkey=data["market_pubkey"],
            timestamp=data["timestamp"],
            orderbook_id=data.get("orderbook_id"),
        )


class MarketEventType(str, Enum):
    """Market event types."""

    ORDERBOOK_CREATED = "orderbook_created"
    SETTLED = "settled"
    OPENED = "opened"
    PAUSED = "paused"
    UNKNOWN = "unknown"

    @classmethod
    def from_str(cls, s: str) -> "MarketEventType":
        try:
            return cls(s)
        except ValueError:
            return cls.UNKNOWN


# ============================================================================
# ERROR TYPES
# ============================================================================


@dataclass
class ErrorData:
    """Error response from server."""

    error: str
    code: str
    orderbook_id: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> "ErrorData":
        return cls(
            error=data["error"],
            code=data["code"],
            orderbook_id=data.get("orderbook_id"),
        )


class ErrorCode(str, Enum):
    """Server error codes."""

    ENGINE_UNAVAILABLE = "ENGINE_UNAVAILABLE"
    INVALID_JSON = "INVALID_JSON"
    INVALID_METHOD = "INVALID_METHOD"
    RATE_LIMITED = "RATE_LIMITED"
    UNKNOWN = "UNKNOWN"

    @classmethod
    def from_str(cls, s: str) -> "ErrorCode":
        try:
            return cls(s)
        except ValueError:
            return cls.UNKNOWN


# ============================================================================
# CLIENT EVENTS
# ============================================================================


class WsEventType(str, Enum):
    """Event types emitted by the WebSocket client."""

    CONNECTED = "connected"
    DISCONNECTED = "disconnected"
    BOOK_UPDATE = "book_update"
    TRADE = "trade"
    USER_UPDATE = "user_update"
    PRICE_UPDATE = "price_update"
    MARKET_EVENT = "market_event"
    ERROR = "error"
    RESYNC_REQUIRED = "resync_required"
    PONG = "pong"
    RECONNECTING = "reconnecting"


@dataclass
class WsEvent:
    """Events emitted by the WebSocket client."""

    type: WsEventType
    orderbook_id: Optional[str] = None
    is_snapshot: Optional[bool] = None
    trade: Optional[TradeData] = None
    event_type: Optional[str] = None
    user: Optional[str] = None
    resolution: Optional[str] = None
    market_pubkey: Optional[str] = None
    error: Optional[Exception] = None
    reason: Optional[str] = None
    attempt: Optional[int] = None

    @classmethod
    def connected(cls) -> "WsEvent":
        return cls(type=WsEventType.CONNECTED)

    @classmethod
    def disconnected(cls, reason: str) -> "WsEvent":
        return cls(type=WsEventType.DISCONNECTED, reason=reason)

    @classmethod
    def book_update(cls, orderbook_id: str, is_snapshot: bool) -> "WsEvent":
        return cls(
            type=WsEventType.BOOK_UPDATE,
            orderbook_id=orderbook_id,
            is_snapshot=is_snapshot,
        )

    @classmethod
    def trade(cls, orderbook_id: str, trade: TradeData) -> "WsEvent":
        return cls(type=WsEventType.TRADE, orderbook_id=orderbook_id, trade=trade)

    @classmethod
    def user_update(cls, event_type: str, user: str) -> "WsEvent":
        return cls(type=WsEventType.USER_UPDATE, event_type=event_type, user=user)

    @classmethod
    def price_update(cls, orderbook_id: str, resolution: str) -> "WsEvent":
        return cls(
            type=WsEventType.PRICE_UPDATE,
            orderbook_id=orderbook_id,
            resolution=resolution,
        )

    @classmethod
    def market_event(cls, event_type: str, market_pubkey: str) -> "WsEvent":
        return cls(
            type=WsEventType.MARKET_EVENT,
            event_type=event_type,
            market_pubkey=market_pubkey,
        )

    @classmethod
    def error(cls, error: Exception) -> "WsEvent":
        return cls(type=WsEventType.ERROR, error=error)

    @classmethod
    def resync_required(cls, orderbook_id: str) -> "WsEvent":
        return cls(type=WsEventType.RESYNC_REQUIRED, orderbook_id=orderbook_id)

    @classmethod
    def pong(cls) -> "WsEvent":
        return cls(type=WsEventType.PONG)

    @classmethod
    def reconnecting(cls, attempt: int) -> "WsEvent":
        return cls(type=WsEventType.RECONNECTING, attempt=attempt)


# ============================================================================
# MESSAGE TYPE ENUM
# ============================================================================


class MessageType(str, Enum):
    """All possible server message types."""

    BOOK_UPDATE = "book_update"
    TRADES = "trades"
    USER = "user"
    PRICE_HISTORY = "price_history"
    MARKET = "market"
    ERROR = "error"
    PONG = "pong"
    UNKNOWN = "unknown"

    @classmethod
    def from_str(cls, s: str) -> "MessageType":
        try:
            return cls(s)
        except ValueError:
            return cls.UNKNOWN


# ============================================================================
# SIDE HELPERS
# ============================================================================


class Side(IntEnum):
    """Order side enum for user events."""

    BUY = 0
    SELL = 1


class PriceLevelSide(str, Enum):
    """Price level side (from orderbook updates)."""

    BID = "bid"
    ASK = "ask"
