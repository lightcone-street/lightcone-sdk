"""Wire types for committed order state (REST) and the WS ``user`` channel.

Quantities named ``*_base`` are ``Decimal`` in base-token units. Order amounts
(``amount_in``/``amount_out``, ``maker_amount``/``taker_amount``) are in the
units of the token each side gives or receives: a bid gives quote and receives
base, an ask gives base and receives quote. Revisions and sequence numbers are
exact ``int`` values that arrive as JSON strings on REST and in WS snapshots
but as JSON numbers in live WS facts; both are accepted.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from decimal import Decimal
from enum import Enum, IntEnum
from typing import Any

from ...shared.price import quote_per_base
from ...shared.types import (
    CommitInfo,
    FundingSource,
    OrderBookId,
    PubkeyStr,
    Side,
    TimeInForce,
)
from ...shared.wire_decode import (
    I16,
    MISSING,
    WireRecord,
    take,
    wire_array,
    wire_bool,
    wire_commit_info,
    wire_decimal,
    wire_enum_or_unknown,
    wire_error,
    wire_funding_source,
    wire_i64_or,
    wire_i128,
    wire_int,
    wire_int_or,
    wire_nonempty_str,
    wire_object,
    wire_opt_str,
    wire_orderbook_id,
    wire_pubkey,
    wire_rfc3339,
    wire_side,
    wire_side_text,
    wire_str,
    wire_str_or,
    wire_timestamp_ms,
    wire_u64,
)
from ..notification import Notification
from ..position.wire import FundingAccount, FundingUpdate
from ..trade.wire import WsTrade
from .model import OrderStatus

_TIME_IN_FORCE = {
    "GTC": TimeInForce.GTC,
    "IOC": TimeInForce.IOC,
    "FOK": TimeInForce.FOK,
}


def wire_time_in_force(value: Any, name: str) -> TimeInForce:
    """A time-in-force wire name (``"GTC"``, ``"IOC"``, or ``"FOK"``, case-sensitive)."""
    text = wire_str(value, name)
    tif = _TIME_IN_FORCE.get(text)
    if tif is None:
        raise wire_error(name, f"unknown time in force {text!r}")
    return tif


def wire_known_time_in_force(value: Any, name: str) -> TimeInForce | None:
    """A time-in-force, or ``None`` when ``null``, absent, or a policy this SDK
    version does not know (the backend sends ``"unsupported"``)."""
    if value is MISSING or value is None:
        return None
    return _TIME_IN_FORCE.get(wire_str(value, name))


def _closed_reason(data: WireRecord, name: str) -> str | None:
    return wire_nonempty_str(take(data, "closed_reason"), f"{name}.closed_reason")


def _base(data: WireRecord, key: str, name: str) -> Decimal:
    return wire_decimal(take(data, key), f"{name}.{key}")


# ─── Committed order state (REST) ────────────────────────────────────────────


@dataclass(frozen=True, kw_only=True)
class OrderState:
    """Current committed cumulative state of an order, returned by order
    submission. All ``*_base`` quantities are in base-token units.

    ``original_base = confirmed_base + pending_base + open_base + cancelled_base``.
    """

    #: Base size of the signed order.
    original_base: Decimal
    #: Base filled by authenticated (on-chain confirmed) executions.
    confirmed_base: Decimal
    #: Base matched but awaiting on-chain confirmation.
    pending_base: Decimal
    #: Base resting on the book.
    open_base: Decimal
    #: Base of ``open_base`` the engine will currently match.
    executable_base: Decimal
    #: Base cancelled: explicitly, by expiry, IOC/FOK remainder, or closure.
    cancelled_base: Decimal
    #: Whether the order is live for matching at this revision.
    ready: bool
    #: Why the order stopped resting, when it did.
    closed_reason: str | None
    committed_revision: int
    #: Engine acceptance sequence (orders the wallet's accepted orders).
    accepted_seq: int

    @staticmethod
    def from_dict(value: Any, name: str = "state") -> OrderState:
        data = wire_object(value, name)
        return OrderState(
            original_base=_base(data, "original_base", name),
            confirmed_base=_base(data, "confirmed_base", name),
            pending_base=_base(data, "pending_base", name),
            open_base=_base(data, "open_base", name),
            executable_base=_base(data, "executable_base", name),
            cancelled_base=_base(data, "cancelled_base", name),
            ready=wire_bool(take(data, "ready"), f"{name}.ready"),
            closed_reason=_closed_reason(data, name),
            committed_revision=wire_u64(
                take(data, "committed_revision"), f"{name}.committed_revision"
            ),
            accepted_seq=wire_u64(take(data, "accepted_seq"), f"{name}.accepted_seq"),
        )


@dataclass(frozen=True, kw_only=True)
class RecordedOrderState:
    """Order quantities recorded at one database snapshot (REST user orders).

    Unlike :class:`OrderState` it makes no claim about live executability.
    All ``*_base`` quantities are in base-token units.
    """

    #: Base size of the signed order.
    original_base: Decimal
    #: Base filled by confirmed executions.
    confirmed_base: Decimal
    #: Base matched but awaiting on-chain confirmation.
    pending_base: Decimal
    #: Base still resting on the book.
    open_base: Decimal
    #: Base cancelled (explicitly, by expiry, or by closure).
    cancelled_base: Decimal
    closed_reason: str | None
    #: Committed database revision of this snapshot.
    committed_revision: int
    #: Engine acceptance sequence; closures cut off by it.
    accepted_seq: int

    @staticmethod
    def from_dict(value: Any, name: str = "state") -> RecordedOrderState:
        data = wire_object(value, name)
        return RecordedOrderState(
            original_base=_base(data, "original_base", name),
            confirmed_base=_base(data, "confirmed_base", name),
            pending_base=_base(data, "pending_base", name),
            open_base=_base(data, "open_base", name),
            cancelled_base=_base(data, "cancelled_base", name),
            closed_reason=_closed_reason(data, name),
            committed_revision=wire_u64(
                take(data, "committed_revision"), f"{name}.committed_revision"
            ),
            accepted_seq=wire_u64(take(data, "accepted_seq"), f"{name}.accepted_seq"),
        )


class InitialCohortState(str, Enum):
    """Progress of the executions the engine selected when it accepted an order."""

    #: The order did not cross the book at acceptance.
    NO_INITIAL_EXECUTIONS = "no_initial_executions"
    #: Some initial executions still await on-chain confirmation.
    PENDING = "pending"
    #: Every initial execution is confirmed or failed.
    COMPLETE = "complete"
    #: A state this SDK version does not know.
    UNKNOWN = "unknown"


@dataclass(frozen=True, kw_only=True)
class InitialCohort:
    """The fixed set of executions selected at acceptance. All ``*_base``
    fields are in base-token units."""

    state: InitialCohortState
    #: Base selected for immediate execution.
    selected_base: Decimal
    #: Base of the selection known to be confirmed on-chain.
    known_confirmed_base: Decimal
    #: Base of the selection whose settlement failed and returned unfilled.
    known_failed_unfilled_base: Decimal
    #: Base of the selection not yet resolved.
    unresolved_base: Decimal

    @staticmethod
    def from_dict(value: Any, name: str = "initial_cohort") -> InitialCohort:
        data = wire_object(value, name)
        return InitialCohort(
            state=wire_enum_or_unknown(
                take(data, "state"),
                f"{name}.state",
                InitialCohortState,
                InitialCohortState.UNKNOWN,
            ),
            selected_base=_base(data, "selected_base", name),
            known_confirmed_base=_base(data, "known_confirmed_base", name),
            known_failed_unfilled_base=_base(data, "known_failed_unfilled_base", name),
            unresolved_base=_base(data, "unresolved_base", name),
        )


# ─── REST user orders ────────────────────────────────────────────────────────


@dataclass(frozen=True, kw_only=True)
class UserOrder:
    """One open or pending order from ``GET /api/users/orders``."""

    order_hash: str
    market_pubkey: PubkeyStr
    orderbook_id: OrderBookId
    side: Side
    #: Amount the maker gives: quote units for bids, base units for asks.
    amount_in: Decimal
    #: Amount the maker receives: base units for bids, quote units for asks.
    amount_out: Decimal
    #: Implied limit price, quote units per base unit.
    price: Decimal
    created_at: datetime
    #: Unix seconds; ``0`` means no expiration.
    expiration: int
    base_mint: PubkeyStr
    quote_mint: PubkeyStr
    #: Outcome index of the base token; the committed backend reports ``-1``.
    outcome_index: int
    #: Recorded quantities; ``None`` if the engine omitted them.
    state: RecordedOrderState | None
    #: ``None`` when the backend reports a policy this SDK version does not
    #: know (it sends ``"unsupported"``).
    tif: TimeInForce | None
    #: Custody source that funds the order.
    source: FundingSource
    #: Always ``"limit"``.
    order_type: str

    @staticmethod
    def from_dict(value: Any, name: str = "order") -> UserOrder:
        data = wire_object(value, name)
        state = take(data, "state")
        return UserOrder(
            order_hash=wire_str(take(data, "order_hash"), f"{name}.order_hash"),
            market_pubkey=wire_pubkey(
                take(data, "market_pubkey"), f"{name}.market_pubkey"
            ),
            orderbook_id=wire_orderbook_id(
                take(data, "orderbook_id"), f"{name}.orderbook_id"
            ),
            side=wire_side_text(take(data, "side"), f"{name}.side"),
            amount_in=_base(data, "amount_in", name),
            amount_out=_base(data, "amount_out", name),
            price=_base(data, "price", name),
            created_at=wire_timestamp_ms(
                take(data, "created_at"), f"{name}.created_at"
            ),
            expiration=wire_i64_or(take(data, "expiration"), f"{name}.expiration", 0),
            base_mint=wire_pubkey(take(data, "base_mint"), f"{name}.base_mint"),
            quote_mint=wire_pubkey(take(data, "quote_mint"), f"{name}.quote_mint"),
            outcome_index=wire_int_or(
                take(data, "outcome_index"), f"{name}.outcome_index", *I16, -1
            ),
            state=(
                None
                if state is MISSING or state is None
                else RecordedOrderState.from_dict(state, f"{name}.state")
            ),
            tif=wire_known_time_in_force(take(data, "tif"), f"{name}.tif"),
            source=wire_funding_source(take(data, "source"), f"{name}.source"),
            order_type=wire_str_or(take(data, "order_type"), f"{name}.order_type", ""),
        )


# ─── REST user order fills ───────────────────────────────────────────────────


class Role(str, Enum):
    """Whether the order was the maker or taker of a fill."""

    MAKER = "maker"
    TAKER = "taker"
    #: A role this SDK version does not know.
    UNKNOWN = "unknown"


@dataclass(frozen=True, kw_only=True)
class OrderFillEvent:
    """A single authenticated fill of an order."""

    #: ``"<execution_id>:<leg_index>:<projection_generation>"``, the same id as
    #: REST ``trade_id``.
    fill_id: str
    counterparty: PubkeyStr
    counterparty_order_hash: str
    #: This order's role in the fill.
    role: Role
    #: Filled size in base-token units.
    base_amount: Decimal
    #: Filled notional in quote-token units.
    quote_amount: Decimal
    #: Signed fee estimate in raw atoms of ``fee_mint`` (negative = rebate). An
    #: estimate from the captured rate, never a balance delta.
    fee_estimate_atoms: int
    fee_mint: PubkeyStr
    #: Maker fee rate captured for the fill, in basis points (negative = rebate).
    maker_fee_bps: int
    #: Taker fee rate captured for the fill, in basis points.
    taker_fee_bps: int
    tx_signature: str
    filled_at: datetime

    def price(self) -> Decimal | None:
        """Fill price in quote units per base unit (``quote_amount / base_amount``)."""
        return quote_per_base(self.quote_amount, self.base_amount)

    @staticmethod
    def from_dict(value: Any, name: str = "fill") -> OrderFillEvent:
        data = wire_object(value, name)
        return OrderFillEvent(
            fill_id=wire_str(take(data, "fill_id"), f"{name}.fill_id"),
            counterparty=wire_pubkey(
                take(data, "counterparty"), f"{name}.counterparty"
            ),
            counterparty_order_hash=wire_str(
                take(data, "counterparty_order_hash"),
                f"{name}.counterparty_order_hash",
            ),
            role=wire_enum_or_unknown(
                take(data, "role"), f"{name}.role", Role, Role.UNKNOWN
            ),
            base_amount=_base(data, "base_amount", name),
            quote_amount=_base(data, "quote_amount", name),
            fee_estimate_atoms=wire_i128(
                take(data, "fee_estimate_atoms"), f"{name}.fee_estimate_atoms"
            ),
            fee_mint=wire_pubkey(take(data, "fee_mint"), f"{name}.fee_mint"),
            maker_fee_bps=wire_int(
                take(data, "maker_fee_bps"), f"{name}.maker_fee_bps", *I16
            ),
            taker_fee_bps=wire_int(
                take(data, "taker_fee_bps"), f"{name}.taker_fee_bps", *I16
            ),
            tx_signature=wire_str(take(data, "tx_signature"), f"{name}.tx_signature"),
            filled_at=wire_timestamp_ms(take(data, "filled_at"), f"{name}.filled_at"),
        )


@dataclass(frozen=True, kw_only=True)
class UserOrderFill:
    """An order the wallet participated in, with its first page of fills.

    All ``*_base`` quantities are in base-token units.
    """

    order_hash: str
    market_pubkey: PubkeyStr
    orderbook_id: OrderBookId
    side: Side
    base_mint: PubkeyStr
    quote_mint: PubkeyStr
    original_base: Decimal
    confirmed_base: Decimal
    pending_base: Decimal
    open_base: Decimal
    cancelled_base: Decimal
    #: Quote exchanged by confirmed fills, in quote-token units.
    confirmed_quote: Decimal
    status: OrderStatus
    closed_reason: str | None
    created_at: datetime
    #: Oldest-first page of at most 16 fills.
    fills: list[OrderFillEvent] = field(default_factory=list)
    fills_has_more: bool
    #: Continue with ``order_hash`` + ``fill_cursor`` on the order-fills endpoint.
    fills_next_cursor: str | None

    @staticmethod
    def from_dict(value: Any, name: str = "order") -> UserOrderFill:
        data = wire_object(value, name)
        return UserOrderFill(
            order_hash=wire_str(take(data, "order_hash"), f"{name}.order_hash"),
            market_pubkey=wire_pubkey(
                take(data, "market_pubkey"), f"{name}.market_pubkey"
            ),
            orderbook_id=wire_orderbook_id(
                take(data, "orderbook_id"), f"{name}.orderbook_id"
            ),
            side=wire_side_text(take(data, "side"), f"{name}.side"),
            base_mint=wire_pubkey(take(data, "base_mint"), f"{name}.base_mint"),
            quote_mint=wire_pubkey(take(data, "quote_mint"), f"{name}.quote_mint"),
            original_base=_base(data, "original_base", name),
            confirmed_base=_base(data, "confirmed_base", name),
            pending_base=_base(data, "pending_base", name),
            open_base=_base(data, "open_base", name),
            cancelled_base=_base(data, "cancelled_base", name),
            confirmed_quote=_base(data, "confirmed_quote", name),
            status=wire_enum_or_unknown(
                take(data, "status"), f"{name}.status", OrderStatus, OrderStatus.UNKNOWN
            ),
            closed_reason=_closed_reason(data, name),
            created_at=wire_timestamp_ms(
                take(data, "created_at"), f"{name}.created_at"
            ),
            fills=wire_array(
                take(data, "fills"), f"{name}.fills", OrderFillEvent.from_dict
            ),
            fills_has_more=wire_bool(
                take(data, "fills_has_more"), f"{name}.fills_has_more", default=False
            ),
            fills_next_cursor=wire_opt_str(
                take(data, "fills_next_cursor"), f"{name}.fills_next_cursor"
            ),
        )


@dataclass(frozen=True, kw_only=True)
class UserOrderFillsResponse:
    """Response from ``GET /api/users/order-fills`` and
    ``GET /api/users/{wallet}/order-fills``."""

    orders: list[UserOrderFill] = field(default_factory=list)
    #: Summary cursor; pass back as ``cursor`` for the next page of orders.
    next_cursor: str | None
    has_more: bool
    #: Database revision of the page.
    committed_revision: int

    @staticmethod
    def from_dict(value: Any) -> UserOrderFillsResponse:
        data = wire_object(value, "order_fills")
        return UserOrderFillsResponse(
            orders=wire_array(
                take(data, "orders"), "order_fills.orders", UserOrderFill.from_dict
            ),
            next_cursor=wire_opt_str(
                take(data, "next_cursor"), "order_fills.next_cursor"
            ),
            has_more=wire_bool(take(data, "has_more"), "order_fills.has_more"),
            committed_revision=wire_u64(
                take(data, "committed_revision"), "order_fills.committed_revision"
            ),
        )


# ─── WS user channel ─────────────────────────────────────────────────────────


def limit_price(
    side: Side, maker_amount: Decimal, taker_amount: Decimal
) -> Decimal | None:
    """Limit price of signed amounts: a bid gives quote for base, an ask gives
    base for quote. ``None`` when the base amount is zero or the quotient is
    outside the exact decimal range."""
    if side == Side.BID:
        return quote_per_base(maker_amount, taker_amount)
    return quote_per_base(taker_amount, maker_amount)


@dataclass(frozen=True, kw_only=True)
class UserSnapshotOrder:
    """One open or pending order in the WS ``user`` snapshot.

    Quantities are the same recorded values as REST, flattened into the order.
    """

    order_hash: str
    market_pubkey: PubkeyStr
    orderbook_id: OrderBookId
    side: Side
    #: Amount the maker gives: quote units for bids, base units for asks.
    maker_amount: Decimal
    #: Amount the maker receives: base units for bids, quote units for asks.
    taker_amount: Decimal
    original_base: Decimal
    confirmed_base: Decimal
    pending_base: Decimal
    open_base: Decimal
    cancelled_base: Decimal
    #: Empty on the wire when the order is not closed.
    closed_reason: str | None
    committed_revision: int
    accepted_seq: int
    #: Implied limit price, quote units per base unit.
    price: Decimal
    created_at: datetime
    #: Unix seconds; ``0`` means no expiration.
    expiration: int
    base_mint: PubkeyStr
    quote_mint: PubkeyStr
    tif: TimeInForce
    funding_source: FundingSource

    @staticmethod
    def from_dict(value: Any, name: str = "order") -> UserSnapshotOrder:
        data = wire_object(value, name)
        return UserSnapshotOrder(
            order_hash=wire_str(take(data, "order_hash"), f"{name}.order_hash"),
            market_pubkey=wire_pubkey(
                take(data, "market_pubkey"), f"{name}.market_pubkey"
            ),
            orderbook_id=wire_orderbook_id(
                take(data, "orderbook_id"), f"{name}.orderbook_id"
            ),
            side=wire_side(take(data, "side"), f"{name}.side"),
            maker_amount=_base(data, "maker_amount", name),
            taker_amount=_base(data, "taker_amount", name),
            original_base=_base(data, "original_base", name),
            confirmed_base=_base(data, "confirmed_base", name),
            pending_base=_base(data, "pending_base", name),
            open_base=_base(data, "open_base", name),
            cancelled_base=_base(data, "cancelled_base", name),
            closed_reason=_closed_reason(data, name),
            committed_revision=wire_u64(
                take(data, "committed_revision"), f"{name}.committed_revision"
            ),
            accepted_seq=wire_u64(take(data, "accepted_seq"), f"{name}.accepted_seq"),
            price=_base(data, "price", name),
            created_at=wire_timestamp_ms(
                take(data, "created_at"), f"{name}.created_at"
            ),
            expiration=wire_i64_or(take(data, "expiration"), f"{name}.expiration", 0),
            base_mint=wire_pubkey(take(data, "base_mint"), f"{name}.base_mint"),
            quote_mint=wire_pubkey(take(data, "quote_mint"), f"{name}.quote_mint"),
            tif=wire_time_in_force(take(data, "tif"), f"{name}.tif"),
            funding_source=wire_funding_source(
                take(data, "funding_source"), f"{name}.funding_source"
            ),
        )


def _notification(value: Any, name: str) -> Notification:
    return Notification.from_dict(wire_object(value, name))


@dataclass(frozen=True, kw_only=True)
class UserSnapshot:
    """WS ``user`` snapshot, sent once per subscription (``event_type: "snapshot"``).

    It holds the first bounded page of open/pending orders and funding
    accounts; fetch further pages over REST with the cursors.
    """

    orders: list[UserSnapshotOrder] = field(default_factory=list)
    funding_accounts: list[FundingAccount] = field(default_factory=list)
    #: Order cursor for ``GET /api/users/orders`` (empty on the wire when absent).
    next_cursor: str | None
    has_more: bool
    #: Funding-account cursor (empty on the wire when absent).
    next_funding_cursor: str | None
    funding_has_more: bool
    #: Database revision of the snapshot (a string here, a number in live facts).
    committed_revision: int
    projection_generation: int
    notifications: list[Notification] = field(default_factory=list)

    @staticmethod
    def from_dict(value: Any, name: str = "snapshot") -> UserSnapshot:
        """Decode a ``snapshot`` payload (the ``event_type`` tag is ignored)."""
        data = wire_object(value, name)
        return UserSnapshot(
            orders=wire_array(
                take(data, "orders"), f"{name}.orders", UserSnapshotOrder.from_dict
            ),
            funding_accounts=wire_array(
                take(data, "funding_accounts"),
                f"{name}.funding_accounts",
                FundingAccount.from_dict,
            ),
            next_cursor=wire_nonempty_str(
                take(data, "next_cursor"), f"{name}.next_cursor"
            ),
            has_more=wire_bool(
                take(data, "has_more"), f"{name}.has_more", default=False
            ),
            next_funding_cursor=wire_nonempty_str(
                take(data, "next_funding_cursor"), f"{name}.next_funding_cursor"
            ),
            funding_has_more=wire_bool(
                take(data, "funding_has_more"),
                f"{name}.funding_has_more",
                default=False,
            ),
            committed_revision=wire_u64(
                take(data, "committed_revision"), f"{name}.committed_revision"
            ),
            projection_generation=wire_u64(
                take(data, "projection_generation"), f"{name}.projection_generation"
            ),
            notifications=wire_array(
                take(data, "notifications"),
                f"{name}.notifications",
                _notification,
                default_empty=True,
            ),
        )


@dataclass(frozen=True, kw_only=True)
class OrderUpdate:
    """Live committed order fact on the WS ``user`` channel (``event_type: "order"``).

    Each fact is the order's complete cumulative state at ``commit``; apply it
    by replacing the stored order (see :meth:`UserOpenLimitOrders.apply`).
    """

    commit: CommitInfo
    user_pubkey: PubkeyStr
    order_hash: str
    market_pubkey: PubkeyStr
    orderbook_id: OrderBookId
    base_mint: PubkeyStr
    quote_mint: PubkeyStr
    #: Numeric (``0`` bid, ``1`` ask) on the wire.
    side: Side
    #: Amount the maker gives: quote units for bids, base units for asks.
    maker_amount: Decimal
    #: Amount the maker receives: base units for bids, quote units for asks.
    taker_amount: Decimal
    original_base: Decimal
    confirmed_base: Decimal
    pending_base: Decimal
    open_base: Decimal
    #: Base of ``open_base`` the engine will currently match.
    executable_base: Decimal
    cancelled_base: Decimal
    accepted_seq: int
    accepted_at: datetime
    #: Unix seconds; ``0`` means no expiration.
    expiration: int
    tif: TimeInForce
    funding_source: FundingSource
    funding_account: PubkeyStr
    closed_reason: str | None
    #: Whether the order is live for matching (``False`` when not actionable).
    ready: bool

    def price(self) -> Decimal | None:
        """Limit price in quote units per base unit, from the signed amounts."""
        return limit_price(self.side, self.maker_amount, self.taker_amount)

    @staticmethod
    def from_dict(value: Any, name: str = "order") -> OrderUpdate:
        """Decode a live ``order`` fact (the ``event_type`` tag is ignored)."""
        data = wire_object(value, name)
        return OrderUpdate(
            commit=wire_commit_info(data, name),
            user_pubkey=wire_pubkey(take(data, "user_pubkey"), f"{name}.user_pubkey"),
            order_hash=wire_str(take(data, "order_hash"), f"{name}.order_hash"),
            market_pubkey=wire_pubkey(
                take(data, "market_pubkey"), f"{name}.market_pubkey"
            ),
            orderbook_id=wire_orderbook_id(
                take(data, "orderbook_id"), f"{name}.orderbook_id"
            ),
            base_mint=wire_pubkey(take(data, "base_mint"), f"{name}.base_mint"),
            quote_mint=wire_pubkey(take(data, "quote_mint"), f"{name}.quote_mint"),
            side=wire_side(take(data, "side"), f"{name}.side"),
            maker_amount=_base(data, "maker_amount", name),
            taker_amount=_base(data, "taker_amount", name),
            original_base=_base(data, "original_base", name),
            confirmed_base=_base(data, "confirmed_base", name),
            pending_base=_base(data, "pending_base", name),
            open_base=_base(data, "open_base", name),
            executable_base=_base(data, "executable_base", name),
            cancelled_base=_base(data, "cancelled_base", name),
            accepted_seq=wire_u64(take(data, "accepted_seq"), f"{name}.accepted_seq"),
            accepted_at=wire_rfc3339(take(data, "accepted_at"), f"{name}.accepted_at"),
            expiration=wire_i64_or(take(data, "expiration"), f"{name}.expiration", 0),
            tif=wire_time_in_force(take(data, "tif"), f"{name}.tif"),
            funding_source=wire_funding_source(
                take(data, "funding_source"), f"{name}.funding_source"
            ),
            funding_account=wire_pubkey(
                take(data, "funding_account"), f"{name}.funding_account"
            ),
            closed_reason=_closed_reason(data, name),
            ready=wire_bool(take(data, "ready"), f"{name}.ready", default=False),
        )


class ClosureScope(IntEnum):
    """Scope kinds of a committed closure (:attr:`ClosureUpdate.scope_kind`)."""

    #: Every order on the exchange.
    EXCHANGE = 0
    #: Orders of one market (``scope_key`` = market pubkey).
    MARKET = 1
    #: Orders of one orderbook (``scope_key`` = orderbook id).
    BOOK = 2
    #: Orders collateralized by one deposit token (``scope_key`` = mint).
    DEPOSIT_TOKEN = 3
    #: Orders of one wallet (``scope_key`` = wallet).
    WALLET = 4
    #: Orders of one wallet on one book (``scope_key`` = ``"<wallet>:<book>"``).
    WALLET_BOOK = 5
    #: One custody account.
    ACCOUNT = 6
    #: One settlement profile.
    SETTLEMENT_PROFILE = 7

    @classmethod
    def from_code(cls, code: int) -> ClosureScope | None:
        """The scope for a raw code, or ``None`` for a code this SDK version does not know."""
        try:
            return cls(code)
        except ValueError:
            return None


@dataclass(frozen=True, kw_only=True)
class ClosureUpdate:
    """Committed accepted-order cutoff (``event_type: "closure"``).

    Every order in scope with ``accepted_seq <= accepted_seq`` is closed.
    Wallet-scoped closures go to that wallet; other scopes are broadcast to
    every user subscription. Individual order facts may follow later.
    """

    commit: CommitInfo
    #: Raw scope code; see :meth:`scope`.
    scope_kind: int
    scope_key: str
    #: Inclusive acceptance-sequence cutoff.
    accepted_seq: int
    reason: str

    def scope(self) -> ClosureScope | None:
        """Typed scope, or ``None`` for an unknown code."""
        return ClosureScope.from_code(self.scope_kind)

    @staticmethod
    def from_dict(value: Any, name: str = "closure") -> ClosureUpdate:
        """Decode a committed closure cutoff (the ``event_type`` tag is ignored)."""
        data = wire_object(value, name)
        return ClosureUpdate(
            commit=wire_commit_info(data, name),
            scope_kind=wire_int(take(data, "scope_kind"), f"{name}.scope_kind", *I16),
            scope_key=wire_str(take(data, "scope_key"), f"{name}.scope_key"),
            accepted_seq=wire_u64(take(data, "accepted_seq"), f"{name}.accepted_seq"),
            reason=wire_str(take(data, "reason"), f"{name}.reason"),
        )


@dataclass(frozen=True, kw_only=True)
class RecoveryCompleted:
    """The engine finished rebuilding committed state
    (``event_type: "recovery_completed"``).

    Cached user state from before the rebuild is stale and no snapshot
    follows: resubscribe or refetch.
    """

    commit: CommitInfo
    #: Projection generation the engine rebuilt to (compare with
    #: ``commit.projection_generation``).
    recovered_generation: int

    @staticmethod
    def from_dict(value: Any, name: str = "recovery_completed") -> RecoveryCompleted:
        data = wire_object(value, name)
        return RecoveryCompleted(
            commit=wire_commit_info(data, name),
            recovered_generation=wire_u64(
                take(data, "recovered_generation"), f"{name}.recovered_generation"
            ),
        )


@dataclass(frozen=True, kw_only=True)
class NotificationUpdate:
    """WS notification push event."""

    notification: Notification

    @staticmethod
    def from_dict(value: Any, name: str = "notification") -> NotificationUpdate:
        data = wire_object(value, name)
        return NotificationUpdate(
            notification=_notification(
                take(data, "notification"), f"{name}.notification"
            )
        )


UserUpdateData = (
    UserSnapshot
    | OrderUpdate
    | FundingUpdate
    | WsTrade
    | ClosureUpdate
    | RecoveryCompleted
    | NotificationUpdate
)

_USER_EVENTS: dict[str, Any] = {
    "snapshot": UserSnapshot.from_dict,
    "order": OrderUpdate.from_dict,
    "funding": FundingUpdate.from_dict,
    # A fill involving the wallet as maker or taker (same payload as the
    # ``trades`` channel).
    "fill": WsTrade.from_dict,
    "closure": ClosureUpdate.from_dict,
    "recovery_completed": RecoveryCompleted.from_dict,
    "notification": NotificationUpdate.from_dict,
}


@dataclass(frozen=True, kw_only=True)
class UserUpdate:
    """WS ``user`` channel payload, discriminated by ``event_type``.

    ``event_type`` is one of ``"snapshot"`` (:class:`UserSnapshot`),
    ``"order"`` (:class:`OrderUpdate`), ``"funding"``
    (:class:`~lightcone_sdk.domain.position.wire.FundingUpdate`), ``"fill"``
    (:class:`~lightcone_sdk.domain.trade.wire.WsTrade`), ``"closure"``
    (:class:`ClosureUpdate`), ``"recovery_completed"``
    (:class:`RecoveryCompleted`), or ``"notification"``
    (:class:`NotificationUpdate`), with the decoded payload in ``data``.

    An event type this SDK version does not know (including the retired
    ``nonce``, ``market_balance_update``, and ``global_deposit_update``)
    decodes as ``event_type == "unknown"`` with ``data`` ``None`` and the wire
    tag in ``raw_event_type``; it is safe to ignore.
    """

    event_type: str
    data: UserUpdateData | None
    #: The wire tag of an ``"unknown"`` event; ``None`` for known events.
    raw_event_type: str | None = None

    @staticmethod
    def from_dict(value: Any) -> UserUpdate:
        """Decode one WS ``user`` channel payload."""
        data = wire_object(value, "user")
        event_type = wire_str(take(data, "event_type"), "user.event_type")
        decode = _USER_EVENTS.get(event_type)
        if decode is None:
            return UserUpdate(
                event_type="unknown", data=None, raw_event_type=event_type
            )
        return UserUpdate(
            event_type=event_type, data=decode(data, f"user.{event_type}")
        )


@dataclass
class AuthUpdate:
    """WS ``auth`` channel payload."""

    status: str = "anonymous"
    authenticated: bool = False
    wallet_address: str | None = None
    reason: str | None = None

    @staticmethod
    def from_dict(d: dict) -> AuthUpdate:
        status = d.get("status")
        if status is None:
            authenticated = d.get("authenticated", False)
            status = "authenticated" if authenticated else "anonymous"
        else:
            authenticated = status == "authenticated"

        return AuthUpdate(
            status=status,
            authenticated=authenticated,
            wallet_address=d.get("wallet", d.get("wallet_address")),
            reason=d.get("reason"),
        )


__all__ = [
    "AuthUpdate",
    "ClosureScope",
    "ClosureUpdate",
    "InitialCohort",
    "InitialCohortState",
    "NotificationUpdate",
    "OrderFillEvent",
    "OrderState",
    "OrderUpdate",
    "RecordedOrderState",
    "RecoveryCompleted",
    "Role",
    "UserOrder",
    "UserOrderFill",
    "UserOrderFillsResponse",
    "UserSnapshot",
    "UserSnapshotOrder",
    "UserUpdate",
    "UserUpdateData",
    "limit_price",
    "wire_known_time_in_force",
    "wire_time_in_force",
]
