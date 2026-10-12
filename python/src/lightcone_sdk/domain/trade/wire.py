"""Wire types for trade responses (REST and WS)."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from decimal import Decimal
from typing import Any

from ...shared.price import quote_per_base
from ...shared.types import CommitInfo, OrderBookId, PubkeyStr, Side
from ...shared.wire_decode import (
    I16,
    I32,
    MISSING,
    U8,
    take,
    wire_array,
    wire_bool,
    wire_commit_info,
    wire_decimal,
    wire_i64,
    wire_int,
    wire_object,
    wire_opt_decimal,
    wire_opt_i64,
    wire_opt_int,
    wire_orderbook_id,
    wire_pubkey,
    wire_rfc3339,
    wire_side,
    wire_side_text,
    wire_str,
)

# ─── REST wire types ─────────────────────────────────────────────────────────


@dataclass(frozen=True, kw_only=True)
class TradeResponseWire:
    """A single trade from the REST API."""

    #: Fill-fact row id (i64); pass as ``cursor`` to page backwards.
    id: int
    #: ``"<execution_id>:<leg_index>:<projection_generation>"``; see
    #: :meth:`WsTrade.trade_id` for the matching live id.
    trade_id: str
    orderbook_id: OrderBookId
    taker_pubkey: str
    maker_pubkey: str
    #: Taker side.
    side: Side
    #: Size in base-token units (decimal text, validated on conversion).
    size: str
    #: Maker price in quote units per base unit (truncated to price decimals).
    price: str
    #: Signed taker fee estimate (negative = rebate), scaled with the quote token's decimals.
    taker_fee_estimate: Decimal | None
    #: Signed maker fee estimate (negative = rebate), scaled with the quote token's decimals.
    maker_fee_estimate: Decimal | None
    #: Unix milliseconds.
    executed_at: int

    @staticmethod
    def from_dict(value: Any, name: str = "trade") -> TradeResponseWire:
        data = wire_object(value, name)
        return TradeResponseWire(
            id=wire_i64(take(data, "id"), f"{name}.id"),
            trade_id=wire_str(take(data, "trade_id"), f"{name}.trade_id"),
            orderbook_id=wire_orderbook_id(
                take(data, "orderbook_id"), f"{name}.orderbook_id"
            ),
            taker_pubkey=wire_str(take(data, "taker_pubkey"), f"{name}.taker_pubkey"),
            maker_pubkey=wire_str(take(data, "maker_pubkey"), f"{name}.maker_pubkey"),
            side=wire_side_text(take(data, "side"), f"{name}.side"),
            size=wire_str(take(data, "size"), f"{name}.size"),
            price=wire_str(take(data, "price"), f"{name}.price"),
            taker_fee_estimate=wire_opt_decimal(
                take(data, "taker_fee_estimate"), f"{name}.taker_fee_estimate"
            ),
            maker_fee_estimate=wire_opt_decimal(
                take(data, "maker_fee_estimate"), f"{name}.maker_fee_estimate"
            ),
            executed_at=wire_i64(take(data, "executed_at"), f"{name}.executed_at"),
        )


@dataclass(frozen=True, kw_only=True)
class TradesDecimals:
    """Decimal precision metadata for trade fields."""

    price: int | None = None
    size: int | None = None
    fee: int | None = None

    @staticmethod
    def from_dict(value: Any, name: str = "trades.decimals") -> TradesDecimals:
        data = wire_object(value, name)
        return TradesDecimals(
            price=wire_opt_int(take(data, "price"), f"{name}.price", *U8),
            size=wire_opt_int(take(data, "size"), f"{name}.size", *U8),
            fee=wire_opt_int(take(data, "fee"), f"{name}.fee", *U8),
        )


@dataclass(frozen=True, kw_only=True)
class TradesResponseWire:
    """REST response for an orderbook's trades."""

    orderbook_id: OrderBookId
    trades: list[TradeResponseWire] = field(default_factory=list)
    next_cursor: int | None = None
    has_more: bool = False
    decimals: TradesDecimals | None = None

    @staticmethod
    def from_dict(value: Any) -> TradesResponseWire:
        data = wire_object(value, "trades")
        decimals = take(data, "decimals")
        return TradesResponseWire(
            orderbook_id=wire_orderbook_id(
                take(data, "orderbook_id"), "trades.orderbook_id"
            ),
            trades=wire_array(
                take(data, "trades"), "trades.trades", TradeResponseWire.from_dict
            ),
            next_cursor=wire_opt_i64(take(data, "next_cursor"), "trades.next_cursor"),
            has_more=wire_bool(
                take(data, "has_more"), "trades.has_more", default=False
            ),
            decimals=(
                None
                if decimals is None or decimals is MISSING
                else TradesDecimals.from_dict(decimals)
            ),
        )


@dataclass(frozen=True, kw_only=True)
class MarketTradesResponseWire:
    """REST response for market-level trades (all orderbooks in a market)."""

    market_pubkey: PubkeyStr
    trades: list[TradeResponseWire] = field(default_factory=list)
    next_cursor: int | None = None
    has_more: bool = False

    @staticmethod
    def from_dict(value: Any) -> MarketTradesResponseWire:
        data = wire_object(value, "market_trades")
        return MarketTradesResponseWire(
            market_pubkey=wire_pubkey(
                take(data, "market_pubkey"), "market_trades.market_pubkey"
            ),
            trades=wire_array(
                take(data, "trades"),
                "market_trades.trades",
                TradeResponseWire.from_dict,
            ),
            next_cursor=wire_opt_i64(
                take(data, "next_cursor"), "market_trades.next_cursor"
            ),
            has_more=wire_bool(
                take(data, "has_more"), "market_trades.has_more", default=False
            ),
        )


# ─── WS wire types ───────────────────────────────────────────────────────────


@dataclass(frozen=True, kw_only=True)
class WsTrade:
    """Committed fill fact on the WS ``trades`` channel, also delivered to the
    maker and taker as a ``user`` channel ``fill`` event.

    The payload carries exact base and quote amounts rather than a price; use
    :meth:`price`. Amount fields are ``Decimal`` in the named token's units.
    """

    commit: CommitInfo
    #: Relay dedup id: ``"<evidence_id>:<event_index>:<leg_index>"``. Not the
    #: REST ``trade_id``; see :meth:`trade_id`.
    fill_id: str
    execution_id: str
    evidence_id: str
    #: Index of the settlement event within the confirming evidence.
    event_index: int
    #: Index of this leg within the execution.
    leg_index: int
    orderbook_id: OrderBookId
    market_pubkey: PubkeyStr
    base_mint: PubkeyStr
    quote_mint: PubkeyStr
    #: Filled size in base-token units.
    base_amount: Decimal
    #: Filled notional in quote-token units.
    quote_amount: Decimal
    maker_order_hash: str
    taker_order_hash: str
    maker_wallet: PubkeyStr
    taker_wallet: PubkeyStr
    #: Taker side; numeric (``0`` bid, ``1`` ask) on the wire.
    taker_side: Side
    #: Signed maker fee estimate in ``fee_mint`` units (negative = rebate).
    maker_fee_estimate: Decimal
    #: Signed taker fee estimate in ``fee_mint`` units.
    taker_fee_estimate: Decimal
    fee_mint: PubkeyStr
    #: Maker fee rate in basis points (negative = rebate).
    maker_fee_bps: int
    #: Taker fee rate in basis points.
    taker_fee_bps: int
    executed_at: datetime

    def price(self) -> Decimal | None:
        """Execution price in quote units per base unit (``quote_amount / base_amount``)."""
        return quote_per_base(self.quote_amount, self.base_amount)

    def trade_id(self) -> str:
        """The REST ``trade_id`` of this fill
        (``"<execution_id>:<leg_index>:<projection_generation>"``), for merging
        live trades with REST history."""
        return (
            f"{self.execution_id}:{self.leg_index}:"
            f"{self.commit.projection_generation}"
        )

    @staticmethod
    def from_dict(value: Any, name: str = "trade") -> WsTrade:
        """Decode a committed fill fact (the relayed ``event_type`` is ignored)."""
        data = wire_object(value, name)
        return WsTrade(
            commit=wire_commit_info(data, name),
            fill_id=wire_str(take(data, "fill_id"), f"{name}.fill_id"),
            execution_id=wire_str(take(data, "execution_id"), f"{name}.execution_id"),
            evidence_id=wire_str(take(data, "evidence_id"), f"{name}.evidence_id"),
            event_index=wire_int(
                take(data, "event_index"), f"{name}.event_index", *I32
            ),
            leg_index=wire_int(take(data, "leg_index"), f"{name}.leg_index", *I32),
            orderbook_id=wire_orderbook_id(
                take(data, "orderbook_id"), f"{name}.orderbook_id"
            ),
            market_pubkey=wire_pubkey(
                take(data, "market_pubkey"), f"{name}.market_pubkey"
            ),
            base_mint=wire_pubkey(take(data, "base_mint"), f"{name}.base_mint"),
            quote_mint=wire_pubkey(take(data, "quote_mint"), f"{name}.quote_mint"),
            base_amount=wire_decimal(take(data, "base_amount"), f"{name}.base_amount"),
            quote_amount=wire_decimal(
                take(data, "quote_amount"), f"{name}.quote_amount"
            ),
            maker_order_hash=wire_str(
                take(data, "maker_order_hash"), f"{name}.maker_order_hash"
            ),
            taker_order_hash=wire_str(
                take(data, "taker_order_hash"), f"{name}.taker_order_hash"
            ),
            maker_wallet=wire_pubkey(
                take(data, "maker_wallet"), f"{name}.maker_wallet"
            ),
            taker_wallet=wire_pubkey(
                take(data, "taker_wallet"), f"{name}.taker_wallet"
            ),
            taker_side=wire_side(take(data, "taker_side"), f"{name}.taker_side"),
            maker_fee_estimate=wire_decimal(
                take(data, "maker_fee_estimate"), f"{name}.maker_fee_estimate"
            ),
            taker_fee_estimate=wire_decimal(
                take(data, "taker_fee_estimate"), f"{name}.taker_fee_estimate"
            ),
            fee_mint=wire_pubkey(take(data, "fee_mint"), f"{name}.fee_mint"),
            maker_fee_bps=wire_int(
                take(data, "maker_fee_bps"), f"{name}.maker_fee_bps", *I16
            ),
            taker_fee_bps=wire_int(
                take(data, "taker_fee_bps"), f"{name}.taker_fee_bps", *I16
            ),
            executed_at=wire_rfc3339(take(data, "executed_at"), f"{name}.executed_at"),
        )


__all__ = [
    "MarketTradesResponseWire",
    "TradeResponseWire",
    "TradesDecimals",
    "TradesResponseWire",
    "WsTrade",
]
