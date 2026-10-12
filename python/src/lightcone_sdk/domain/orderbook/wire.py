"""Orderbook wire types."""

from dataclasses import dataclass, field
from typing import Any, Optional

from ...error import _require
from ...shared.scaling import OrderbookRules, TradingRules
from ...shared.wire_decode import (
    MISSING,
    U8,
    U32,
    take,
    wire_array,
    wire_bool,
    wire_decimal,
    wire_int,
    wire_object,
    wire_opt_int,
    wire_opt_str,
    wire_orderbook_id,
    wire_str,
    wire_u64_number,
)


def _opt_decimal_text(value: Any, name: str) -> str | None:
    """Optional plain decimal text, validated and kept as text."""
    if value is MISSING or value is None:
        return None
    wire_decimal(value, name)
    return value


@dataclass
class PriceLevel:
    """One REST depth level. ``price`` (quote per base) and ``size`` (base
    units) are plain decimal text."""

    price: str
    size: str
    orders: Optional[int] = None

    @staticmethod
    def from_dict(value: Any, name: str = "level") -> "PriceLevel":
        data = wire_object(value, name)
        price = wire_str(take(data, "price"), f"{name}.price")
        size = wire_str(take(data, "size"), f"{name}.size")
        wire_decimal(price, f"{name}.price")
        wire_decimal(size, f"{name}.size")
        return PriceLevel(
            price=price,
            size=size,
            orders=wire_opt_int(take(data, "orders"), f"{name}.orders", *U32),
        )


@dataclass
class OrderbookDepthDecimals:
    """Price/size display decimals from the depth endpoint. Distinct from
    ``DecimalsResponse`` (the ``/decimals`` endpoint)."""

    price: int
    size: int

    @staticmethod
    def from_dict(value: Any, name: str = "decimals") -> "OrderbookDepthDecimals":
        data = wire_object(value, name)
        return OrderbookDepthDecimals(
            price=wire_int(take(data, "price"), f"{name}.price", *U8),
            size=wire_int(take(data, "size"), f"{name}.size", *U8),
        )


@dataclass
class OrderbookDepthResponse:
    """REST depth response. Depth is capped server-side at 20 levels per side."""

    orderbook_id: str
    price_quantum: str
    trading_rules: TradingRules
    revision: int
    captured_at_ms: int
    decimals: OrderbookDepthDecimals
    bids: list[PriceLevel] = field(default_factory=list)
    asks: list[PriceLevel] = field(default_factory=list)
    market_pubkey: Optional[str] = None
    best_bid: Optional[str] = None
    best_ask: Optional[str] = None
    spread: Optional[str] = None
    #: Deprecated backend alias. Never use for order admission.
    tick_size: Optional[str] = None
    bids_truncated: bool = False
    asks_truncated: bool = False
    #: Whether the engine's committed book is ready (matching live). A book
    #: that is not ready may show stale or empty depth.
    ready: bool = False

    @staticmethod
    def from_dict(value: Any) -> "OrderbookDepthResponse":
        """Decode ``GET /api/orderbook/{id}``.

        A body that does not match the Rust SDK's ``OrderbookDepthResponse``
        raises :class:`~lightcone_sdk.error.DeserializationError` naming the
        field.
        """
        name = "orderbook_depth"
        data = wire_object(value, name)
        return OrderbookDepthResponse(
            orderbook_id=wire_orderbook_id(
                take(data, "orderbook_id"), f"{name}.orderbook_id"
            ),
            market_pubkey=wire_opt_str(
                take(data, "market_pubkey"), f"{name}.market_pubkey"
            ),
            best_bid=_opt_decimal_text(take(data, "best_bid"), f"{name}.best_bid"),
            best_ask=_opt_decimal_text(take(data, "best_ask"), f"{name}.best_ask"),
            spread=_opt_decimal_text(take(data, "spread"), f"{name}.spread"),
            tick_size=wire_opt_str(take(data, "tick_size"), f"{name}.tick_size"),
            price_quantum=wire_str(
                take(data, "price_quantum"), f"{name}.price_quantum"
            ),
            trading_rules=_trading_rules_from_dict(
                wire_object(take(data, "trading_rules"), f"{name}.trading_rules")
            ),
            bids_truncated=wire_bool(
                take(data, "bids_truncated"), f"{name}.bids_truncated", default=False
            ),
            asks_truncated=wire_bool(
                take(data, "asks_truncated"), f"{name}.asks_truncated", default=False
            ),
            revision=wire_u64_number(take(data, "revision"), f"{name}.revision"),
            captured_at_ms=wire_u64_number(
                take(data, "captured_at_ms"), f"{name}.captured_at_ms"
            ),
            bids=wire_array(take(data, "bids"), f"{name}.bids", PriceLevel.from_dict),
            asks=wire_array(take(data, "asks"), f"{name}.asks", PriceLevel.from_dict),
            decimals=OrderbookDepthDecimals.from_dict(
                take(data, "decimals"), f"{name}.decimals"
            ),
            ready=wire_bool(take(data, "ready"), f"{name}.ready", default=False),
        )


@dataclass
class OrderbookResponse:
    """Full REST orderbook response."""
    id: int = 0
    market_pubkey: str = ""
    orderbook_id: str = ""
    base_token: str = ""
    quote_token: str = ""
    outcome_index: int = 0
    tick_size: Optional[str] = None
    total_bids: int = 0
    total_asks: int = 0
    last_trade_price: Optional[str] = None
    last_trade_time: Optional[str] = None
    active: bool = True
    created_at: Optional[str] = None
    updated_at: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "OrderbookResponse":
        return OrderbookResponse(
            id=d.get("id", 0),
            market_pubkey=d.get("market_pubkey", ""),
            orderbook_id=d.get("orderbook_id", ""),
            base_token=d.get("base_token", ""),
            quote_token=d.get("quote_token", ""),
            outcome_index=d.get("outcome_index", 0),
            tick_size=d.get("tick_size"),
            total_bids=d.get("total_bids", 0),
            total_asks=d.get("total_asks", 0),
            last_trade_price=d.get("last_trade_price"),
            last_trade_time=d.get("last_trade_time"),
            active=d.get("active", True),
            created_at=d.get("created_at"),
            updated_at=d.get("updated_at"),
        )


@dataclass
class OrderbooksResponse:
    """Paginated orderbooks response."""
    orderbooks: list[OrderbookResponse] = field(default_factory=list)
    total: int = 0

    @staticmethod
    def from_dict(d: dict) -> "OrderbooksResponse":
        return OrderbooksResponse(
            orderbooks=[OrderbookResponse.from_dict(o) for o in d.get("orderbooks", [])],
            total=d.get("total", 0),
        )


@dataclass
class WsBookLevel:
    """WebSocket book level with exact quote-token liquidity."""
    side: int
    price: str
    size: str
    #: Exact quote amount at underlying maker prices, not grouped price * size.
    quote_notional: str

    @staticmethod
    def from_dict(d: dict) -> "WsBookLevel":
        quote_notional = _require(d, "quote_notional", "WsBookLevel")
        if not isinstance(quote_notional, str):
            from ...error import DeserializationError

            raise DeserializationError(
                "WsBookLevel quote_notional must be a decimal string"
            )
        return WsBookLevel(
            side=d.get("side", 0),
            price=str(d.get("price", "0")),
            size=str(d.get("size", "0")),
            quote_notional=quote_notional,
        )


@dataclass
class WsOrderBook:
    """WebSocket orderbook snapshot frame.

    The stream is snapshot-only: every data frame carries the full top-20
    levels per side and replaces the previous book wholesale. ``seq`` is the
    real engine depth revision and is gated within one subscription
    generation. Resync frames (``resync: true``) carry no ``seq``, ``ready``,
    or levels; they read as ``0``, ``False``, and ``[]``.
    """
    orderbook_id: str
    seq: int = 0
    #: Whether the engine's committed book is ready (matching live). Absent
    #: (``False``) on resync frames.
    ready: bool = False
    is_snapshot: bool = False
    resync: bool = False
    timestamp: Optional[str] = None
    bids: list[WsBookLevel] = field(default_factory=list)
    asks: list[WsBookLevel] = field(default_factory=list)
    bids_truncated: bool = False
    asks_truncated: bool = False
    #: Aggregation tags echoed by the backend (``None`` = full precision).
    #: Always normalized server-side ((5, none) arrives as (5, 1)).
    n_sig_figs: Optional[int] = None
    mantissa: Optional[int] = None

    def aggregation(self) -> "BookAggregation":
        """The aggregation view this frame belongs to (untagged = full
        precision). Use it to key per-``(orderbook_id, aggregation)`` book
        state when one connection holds multiple aggregation views of the
        same orderbook."""
        from .aggregation import BookAggregation

        return BookAggregation.from_frame(self.n_sig_figs, self.mantissa)

    @staticmethod
    def from_dict(d: dict) -> "WsOrderBook":
        name = "book_update"
        ob_id = d.get("orderbook_id") or d.get("id")
        if ob_id is None:
            from ...error import DeserializationError
            raise DeserializationError("Missing required field 'orderbook_id' in WsOrderBook")
        seq = take(d, "seq")
        return WsOrderBook(
            orderbook_id=ob_id,
            is_snapshot=d.get("is_snapshot", False),
            seq=0 if seq is MISSING else wire_u64_number(seq, f"{name}.seq"),
            ready=wire_bool(take(d, "ready"), f"{name}.ready", default=False),
            resync=d.get("resync", False),
            timestamp=d.get("timestamp"),
            bids=wire_array(
                take(d, "bids"), f"{name}.bids", _ws_book_level, default_empty=True
            ),
            asks=wire_array(
                take(d, "asks"), f"{name}.asks", _ws_book_level, default_empty=True
            ),
            bids_truncated=wire_bool(
                take(d, "bids_truncated"), f"{name}.bids_truncated", default=False
            ),
            asks_truncated=wire_bool(
                take(d, "asks_truncated"), f"{name}.asks_truncated", default=False
            ),
            n_sig_figs=d.get("n_sig_figs"),
            mantissa=d.get("mantissa"),
        )


def _ws_book_level(value: Any, name: str) -> WsBookLevel:
    return WsBookLevel.from_dict(wire_object(value, name))


@dataclass
class DecimalsResponse:
    orderbook_id: str
    base_decimals: int
    quote_decimals: int
    price_decimals: int
    trading_rules: TradingRules

    @staticmethod
    def from_dict(d: dict) -> "DecimalsResponse":
        rules = _require(d, "trading_rules", "DecimalsResponse")
        orderbook_id = _require(d, "orderbook_id", "DecimalsResponse")
        if not isinstance(orderbook_id, str) or not isinstance(rules, dict):
            from ...error import DeserializationError

            raise DeserializationError("DecimalsResponse has invalid required field types")
        return DecimalsResponse(
            orderbook_id=orderbook_id,
            base_decimals=_non_negative_int(d, "base_decimals", "DecimalsResponse"),
            quote_decimals=_non_negative_int(d, "quote_decimals", "DecimalsResponse"),
            price_decimals=_non_negative_int(d, "price_decimals", "DecimalsResponse"),
            trading_rules=_trading_rules_from_dict(rules),
        )

    def to_rules(self) -> OrderbookRules:
        return OrderbookRules(
            orderbook_id=self.orderbook_id,
            base_decimals=self.base_decimals,
            quote_decimals=self.quote_decimals,
            price_decimals=self.price_decimals,
            trading_rules=self.trading_rules,
        )


def _trading_rules_from_dict(d: dict) -> TradingRules:
    if not isinstance(d, dict):
        from ...error import DeserializationError

        raise DeserializationError("TradingRules must be an object")
    price_quantum_raw = _require(d, "price_quantum_raw", "TradingRules")
    base_size_quantum_raw = _require(d, "base_size_quantum_raw", "TradingRules")
    price_quantum = _require(d, "price_quantum", "TradingRules")
    base_size_quantum = _require(d, "base_size_quantum", "TradingRules")
    if (
        not isinstance(price_quantum_raw, str)
        or not price_quantum_raw
        or any(char not in "0123456789" for char in price_quantum_raw)
        or not isinstance(base_size_quantum_raw, str)
        or not base_size_quantum_raw
        or any(char not in "0123456789" for char in base_size_quantum_raw)
        or not isinstance(price_quantum, str)
        or not isinstance(base_size_quantum, str)
    ):
        from ...error import DeserializationError

        raise DeserializationError(
            "TradingRules quantum fields must be decimal strings"
        )
    integer_prices_always_allowed = _require(
        d, "integer_prices_always_allowed", "TradingRules"
    )
    if not isinstance(integer_prices_always_allowed, bool):
        from ...error import DeserializationError

        raise DeserializationError(
            "TradingRules integer_prices_always_allowed must be a boolean"
        )
    return TradingRules(
        base_size_decimals=_non_negative_int(d, "base_size_decimals", "TradingRules"),
        max_price_decimals=_non_negative_int(d, "max_price_decimals", "TradingRules"),
        max_price_significant_figures=_non_negative_int(
            d, "max_price_significant_figures", "TradingRules"
        ),
        integer_prices_always_allowed=integer_prices_always_allowed,
        price_quantum=price_quantum,
        price_quantum_raw=int(price_quantum_raw),
        base_size_quantum=base_size_quantum,
        base_size_quantum_raw=int(base_size_quantum_raw),
    )


def _non_negative_int(d: dict, key: str, context: str) -> int:
    value = _require(d, key, context)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        from ...error import DeserializationError

        raise DeserializationError(
            f"{context} field '{key}' must be a non-negative integer"
        )
    return value


@dataclass
class WsTickerData:
    orderbook_id: str
    best_bid: Optional[str] = None
    best_ask: Optional[str] = None
    mid_price: Optional[str] = None
    last_trade_price: Optional[str] = None

    @staticmethod
    def from_dict(d: dict) -> "WsTickerData":
        return WsTickerData(
            orderbook_id=d.get("orderbook_id", ""),
            best_bid=d.get("best_bid"),
            best_ask=d.get("best_ask"),
            mid_price=d.get("mid_price") or d.get("mid"),
            last_trade_price=d.get("last_trade_price"),
        )
