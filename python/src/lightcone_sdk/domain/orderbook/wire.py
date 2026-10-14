"""Orderbook wire types."""

from dataclasses import dataclass, field
from typing import Optional

from ...error import _require
from ...shared.scaling import OrderbookRules, TradingRules


@dataclass
class PriceLevel:
    price: str
    size: str
    orders: Optional[int] = None

    @staticmethod
    def from_dict(d: dict) -> "PriceLevel":
        return PriceLevel(price=d.get("price", "0"), size=d.get("size", "0"), orders=d.get("orders"))

    @staticmethod
    def from_list(lst: list) -> "PriceLevel":
        return PriceLevel(price=str(lst[0]) if lst else "0", size=str(lst[1]) if len(lst) > 1 else "0")


@dataclass
class OrderbookDepthDecimals:
    """Price/size display decimals from the depth endpoint. Distinct from
    ``DecimalsResponse`` (the ``/decimals`` endpoint)."""

    price: int
    size: int

    @staticmethod
    def from_dict(d: dict) -> "OrderbookDepthDecimals":
        price = _require(d, "price", "OrderbookDepthDecimals")
        size = _require(d, "size", "OrderbookDepthDecimals")
        if (
            not isinstance(price, int)
            or isinstance(price, bool)
            or price < 0
            or not isinstance(size, int)
            or isinstance(size, bool)
            or size < 0
        ):
            from ...error import DeserializationError

            raise DeserializationError(
                "OrderbookDepthDecimals price and size must be non-negative integers"
            )
        return OrderbookDepthDecimals(price=price, size=size)


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

    @staticmethod
    def from_dict(d: dict) -> "OrderbookDepthResponse":
        decimals_raw = _require(d, "decimals", "OrderbookDepthResponse")
        if not isinstance(decimals_raw, dict):
            from ...error import DeserializationError

            raise DeserializationError("OrderbookDepthResponse decimals must be an object")
        revision = _require(d, "revision", "OrderbookDepthResponse")
        captured_at_ms = _require(d, "captured_at_ms", "OrderbookDepthResponse")
        if (
            not isinstance(revision, int)
            or isinstance(revision, bool)
            or revision < 0
            or not isinstance(captured_at_ms, int)
            or isinstance(captured_at_ms, bool)
            or captured_at_ms < 0
        ):
            from ...error import DeserializationError

            raise DeserializationError(
                "OrderbookDepthResponse revision and captured_at_ms must be non-negative integers"
            )
        rules_raw = _require(d, "trading_rules", "OrderbookDepthResponse")
        price_quantum = _require(d, "price_quantum", "OrderbookDepthResponse")
        bids_truncated = d.get("bids_truncated", False)
        asks_truncated = d.get("asks_truncated", False)
        if (
            not isinstance(price_quantum, str)
            or not isinstance(bids_truncated, bool)
            or not isinstance(asks_truncated, bool)
        ):
            from ...error import DeserializationError

            raise DeserializationError(
                "OrderbookDepthResponse quantum and truncation metadata have invalid types"
            )
        return OrderbookDepthResponse(
            bids=[PriceLevel.from_dict(b) if isinstance(b, dict) else PriceLevel.from_list(b) for b in d.get("bids", [])],
            asks=[PriceLevel.from_dict(a) if isinstance(a, dict) else PriceLevel.from_list(a) for a in d.get("asks", [])],
            orderbook_id=_require(d, "orderbook_id", "OrderbookDepthResponse"),
            market_pubkey=d.get("market_pubkey"),
            best_bid=d.get("best_bid"),
            best_ask=d.get("best_ask"),
            spread=d.get("spread"),
            tick_size=d.get("tick_size"),
            price_quantum=price_quantum,
            trading_rules=_trading_rules_from_dict(rules_raw),
            bids_truncated=bids_truncated,
            asks_truncated=asks_truncated,
            revision=revision,
            captured_at_ms=captured_at_ms,
            decimals=OrderbookDepthDecimals.from_dict(decimals_raw),
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
    real engine depth revision and is gated within one subscription generation.
    """
    orderbook_id: str
    seq: int
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
        ob_id = d.get("orderbook_id") or d.get("id")
        if ob_id is None:
            from ...error import DeserializationError
            raise DeserializationError("Missing required field 'orderbook_id' in WsOrderBook")
        seq = _require(d, "seq", "WsOrderBook")
        if not isinstance(seq, int) or isinstance(seq, bool) or seq < 0:
            from ...error import DeserializationError

            raise DeserializationError("WsOrderBook seq must be a non-negative integer")
        bids_truncated = d.get("bids_truncated", False)
        asks_truncated = d.get("asks_truncated", False)
        if not isinstance(bids_truncated, bool) or not isinstance(asks_truncated, bool):
            from ...error import DeserializationError

            raise DeserializationError("WsOrderBook truncation flags must be booleans")
        return WsOrderBook(
            orderbook_id=ob_id,
            is_snapshot=d.get("is_snapshot", False),
            seq=seq,
            resync=d.get("resync", False),
            timestamp=d.get("timestamp"),
            bids=[WsBookLevel.from_dict(b) for b in d.get("bids", [])],
            asks=[WsBookLevel.from_dict(a) for a in d.get("asks", [])],
            bids_truncated=bids_truncated,
            asks_truncated=asks_truncated,
            n_sig_figs=d.get("n_sig_figs"),
            mantissa=d.get("mantissa"),
        )


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
