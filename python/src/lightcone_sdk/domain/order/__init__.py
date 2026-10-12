"""Order domain types: request bodies, submission and cancellation responses,
REST order pages, the WS ``user`` channel, and the open-order container."""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum
from typing import Any

from ...shared.price import quote_per_base
from ...shared.types import OrderBookId, PubkeyStr
from ...shared.wire_decode import (
    I16,
    MISSING,
    take,
    wire_array,
    wire_bool,
    wire_decimal,
    wire_enum_or_unknown,
    wire_i128,
    wire_int,
    wire_nonempty_str,
    wire_object,
    wire_opt_str,
    wire_opt_u64,
    wire_pubkey,
    wire_str,
    wire_str_or,
    wire_u64,
)
from ..position.wire import FundingAccount
from .model import LimitOrder, OrderStatus, OrderType
from .wire import (
    AuthUpdate,
    ClosureScope,
    ClosureUpdate,
    InitialCohort,
    InitialCohortState,
    NotificationUpdate,
    OrderFillEvent,
    OrderState,
    OrderUpdate,
    RecordedOrderState,
    RecoveryCompleted,
    Role,
    UserOrder,
    UserOrderFill,
    UserOrderFillsResponse,
    UserSnapshot,
    UserSnapshotOrder,
    UserUpdate,
    UserUpdateData,
    limit_price,
)

# ─── Request types ───────────────────────────────────────────────────────────


@dataclass
class CancelBody:
    order_hash: str
    maker: str
    signature: str

    def to_dict(self) -> dict:
        return {
            "order_hash": self.order_hash,
            "maker": self.maker,
            "signature": self.signature,
        }


@dataclass
class CancelAllBody:
    user_pubkey: str
    orderbook_id: str
    signature: str
    timestamp: int
    salt: str

    def to_dict(self) -> dict:
        return {
            "user_pubkey": self.user_pubkey,
            "orderbook_id": self.orderbook_id,
            "signature": self.signature,
            "timestamp": self.timestamp,
            "salt": self.salt,
        }


# ─── Response types ──────────────────────────────────────────────────────────


class SubmitOrderStatus(str, Enum):
    """Outcome of an accepted order submission."""

    #: Accepted; no fill awaits confirmation. The order may rest, or (IOC) its
    #: unfilled remainder is already cancelled.
    ACCEPTED = "accepted"
    #: Accepted with matched base awaiting on-chain confirmation, or the
    #: acceptance is durable but its committed view was not yet readable (then
    #: ``state`` and ``initial_cohort`` are ``None``).
    ACCEPTED_PENDING = "accepted_pending"
    #: The whole original base is confirmed filled.
    FILLED = "filled"
    #: An accepted status this SDK version does not know.
    UNKNOWN = "unknown"


@dataclass(frozen=True, kw_only=True)
class FillInfo:
    """An authenticated fill captured with a submission response."""

    #: ``"<execution_id>:<leg_index>:<projection_generation>"``, the same id as
    #: REST ``trade_id``.
    fill_id: str
    execution_id: str
    counterparty: PubkeyStr
    counterparty_order_hash: str
    #: Filled size in base-token units.
    base_amount: Decimal
    #: Filled notional in quote-token units.
    quote_amount: Decimal
    #: Whether the submitted order was the maker of this fill.
    is_maker: bool
    #: Signed fee estimate in raw atoms of ``fee_mint`` (negative = rebate).
    fee_estimate_atoms: int
    #: Captured fee rate in basis points (negative = rebate).
    fee_bps: int
    fee_mint: PubkeyStr

    def price(self) -> Decimal | None:
        """Fill price in quote units per base unit (``quote_amount / base_amount``)."""
        return quote_per_base(self.quote_amount, self.base_amount)

    @staticmethod
    def from_dict(value: Any, name: str = "fill") -> FillInfo:
        data = wire_object(value, name)
        return FillInfo(
            fill_id=wire_str(take(data, "fill_id"), f"{name}.fill_id"),
            execution_id=wire_str(take(data, "execution_id"), f"{name}.execution_id"),
            counterparty=wire_pubkey(
                take(data, "counterparty"), f"{name}.counterparty"
            ),
            counterparty_order_hash=wire_str(
                take(data, "counterparty_order_hash"),
                f"{name}.counterparty_order_hash",
            ),
            base_amount=wire_decimal(take(data, "base_amount"), f"{name}.base_amount"),
            quote_amount=wire_decimal(
                take(data, "quote_amount"), f"{name}.quote_amount"
            ),
            is_maker=wire_bool(take(data, "is_maker"), f"{name}.is_maker"),
            fee_estimate_atoms=wire_i128(
                take(data, "fee_estimate_atoms"), f"{name}.fee_estimate_atoms"
            ),
            fee_bps=wire_int(take(data, "fee_bps"), f"{name}.fee_bps", *I16),
            fee_mint=wire_pubkey(take(data, "fee_mint"), f"{name}.fee_mint"),
        )


@dataclass(frozen=True, kw_only=True)
class SubmitOrderResponse:
    """Response of ``POST /api/orders/submit`` for an accepted order.

    Business rejections raise :class:`~lightcone_sdk.error.ApiRejected`
    with a :class:`~lightcone_sdk.shared.RejectionCode`. An
    ``ENGINE_UNAVAILABLE`` error means the outcome is unknown: see
    :meth:`Orders.submit <lightcone_sdk.domain.order.client.Orders.submit>`
    for how to resolve it.
    """

    order_hash: str
    status: SubmitOrderStatus
    #: Committed cumulative state; ``None`` when the view is not yet readable.
    state: OrderState | None
    #: Executions selected at acceptance; ``None`` alongside ``state``.
    initial_cohort: InitialCohort | None
    #: Up to 16 authenticated fills, oldest first.
    fills: list[FillInfo] = field(default_factory=list)
    #: ``False`` when more fills exist or a newer revision prevented capture.
    fills_complete: bool
    #: Continue with :meth:`Orders.get_order_fill_page` (order hash + this cursor).
    fills_next_cursor: str | None

    def filled_base(self) -> Decimal | None:
        """Base filled by confirmed fills, when the committed view is available."""
        return None if self.state is None else self.state.confirmed_base

    def open_base(self) -> Decimal | None:
        """Base still resting on the book, when the committed view is available."""
        return None if self.state is None else self.state.open_base

    @staticmethod
    def from_dict(value: Any) -> SubmitOrderResponse:
        """Decode ``POST /api/orders/submit``."""
        data = wire_object(value, "submit")
        state = take(data, "state")
        cohort = take(data, "initial_cohort")
        return SubmitOrderResponse(
            order_hash=wire_str(take(data, "order_hash"), "submit.order_hash"),
            status=wire_enum_or_unknown(
                take(data, "status"),
                "submit.status",
                SubmitOrderStatus,
                SubmitOrderStatus.UNKNOWN,
            ),
            state=(
                None
                if state is MISSING or state is None
                else OrderState.from_dict(state, "submit.state")
            ),
            initial_cohort=(
                None
                if cohort is MISSING or cohort is None
                else InitialCohort.from_dict(cohort, "submit.initial_cohort")
            ),
            fills=wire_array(
                take(data, "fills"),
                "submit.fills",
                FillInfo.from_dict,
                default_empty=True,
            ),
            fills_complete=wire_bool(
                take(data, "fills_complete"), "submit.fills_complete", default=False
            ),
            fills_next_cursor=wire_opt_str(
                take(data, "fills_next_cursor"), "submit.fills_next_cursor"
            ),
        )


class CancelStatus(str, Enum):
    """Disposition of a single-order cancellation."""

    #: The open remainder was cancelled by this request.
    CANCELLED = "cancelled"
    #: The order was already closed (cancelled, expired, or cut off).
    ALREADY_CLOSED = "already_closed"
    #: The order was already completely filled.
    ALREADY_FILLED = "already_filled"
    #: A disposition this SDK version does not know.
    UNKNOWN = "unknown"


@dataclass(frozen=True, kw_only=True)
class CancelQuantities:
    """Cancellation quantities in raw base-token atoms (``quantity_unit`` is
    ``"base_atoms"``), sent as integer strings."""

    #: Base atoms cancelled by this request.
    newly_cancelled_base: int
    #: Base atoms confirmed filled.
    confirmed_base: int
    #: Base atoms matched and awaiting on-chain confirmation.
    pending_base: int
    #: Base atoms still open after the request.
    remaining_open_base: int

    @staticmethod
    def from_dict(value: Any, name: str = "quantities") -> CancelQuantities:
        data = wire_object(value, name)
        return CancelQuantities(
            newly_cancelled_base=wire_u64(
                take(data, "newly_cancelled_base"), f"{name}.newly_cancelled_base"
            ),
            confirmed_base=wire_u64(
                take(data, "confirmed_base"), f"{name}.confirmed_base"
            ),
            pending_base=wire_u64(take(data, "pending_base"), f"{name}.pending_base"),
            remaining_open_base=wire_u64(
                take(data, "remaining_open_base"), f"{name}.remaining_open_base"
            ),
        )


@dataclass(frozen=True, kw_only=True)
class CancelSuccess:
    """Response of ``POST /api/orders/cancel``."""

    status: CancelStatus
    order_hash: str
    #: Present on every successful cancellation (an unknown hash is the
    #: ``ORDER_NOT_FOUND`` rejection instead); ``None`` only if a backend
    #: omits it, never invented zeroes.
    quantities: CancelQuantities | None
    #: Unit of ``quantities``: always ``"base_atoms"``.
    quantity_unit: str
    #: Committed revision of the cancellation.
    revision: int
    #: Why the order is closed; empty on the wire when none.
    closed_reason: str | None

    @staticmethod
    def from_dict(value: Any) -> CancelSuccess:
        """Decode ``POST /api/orders/cancel``."""
        data = wire_object(value, "cancel")
        quantities = take(data, "quantities")
        return CancelSuccess(
            status=wire_enum_or_unknown(
                take(data, "status"),
                "cancel.status",
                CancelStatus,
                CancelStatus.UNKNOWN,
            ),
            order_hash=wire_str(take(data, "order_hash"), "cancel.order_hash"),
            quantities=(
                None
                if quantities is MISSING or quantities is None
                else CancelQuantities.from_dict(quantities, "cancel.quantities")
            ),
            quantity_unit=wire_str(take(data, "quantity_unit"), "cancel.quantity_unit"),
            revision=wire_u64(take(data, "revision"), "cancel.revision"),
            closed_reason=wire_nonempty_str(
                take(data, "closed_reason"), "cancel.closed_reason"
            ),
        )


@dataclass(frozen=True, kw_only=True)
class ClosureAck:
    """Committed accepted-order cutoff returned by cancel-all."""

    #: Closure operation id (UUID).
    operation_id: str
    committed_revision: int
    #: ``"Wallet:<wallet>"`` or ``"WalletBook:<wallet>:<orderbook>"``.
    scope: str
    #: Orders with ``accepted_seq`` at or below this are closed; ``None`` when
    #: the scope had no accepted orders.
    accepted_seq_cutoff: int | None
    #: True while bounded per-order cleanup is still running; individual order
    #: facts follow on the ``user`` channel.
    cleanup_pending: bool

    @staticmethod
    def from_dict(value: Any, name: str = "closure") -> ClosureAck:
        data = wire_object(value, name)
        return ClosureAck(
            operation_id=wire_str(take(data, "operation_id"), f"{name}.operation_id"),
            committed_revision=wire_u64(
                take(data, "committed_revision"), f"{name}.committed_revision"
            ),
            scope=wire_str(take(data, "scope"), f"{name}.scope"),
            accepted_seq_cutoff=wire_opt_u64(
                take(data, "accepted_seq_cutoff"), f"{name}.accepted_seq_cutoff"
            ),
            cleanup_pending=wire_bool(
                take(data, "cleanup_pending"), f"{name}.cleanup_pending"
            ),
        )


@dataclass(frozen=True, kw_only=True)
class CancelAllSuccess:
    """Response of ``POST /api/orders/cancel-all``."""

    #: Always ``"success"``.
    status: str
    user_pubkey: PubkeyStr
    #: Empty for a wallet-wide cancel-all.
    orderbook_id: OrderBookId
    closure: ClosureAck | None
    message: str

    @staticmethod
    def from_dict(value: Any) -> CancelAllSuccess:
        """Decode ``POST /api/orders/cancel-all``."""
        data = wire_object(value, "cancel_all")
        closure = take(data, "closure")
        return CancelAllSuccess(
            status=wire_str(take(data, "status"), "cancel_all.status"),
            user_pubkey=wire_pubkey(
                take(data, "user_pubkey"), "cancel_all.user_pubkey"
            ),
            orderbook_id=OrderBookId(
                wire_str_or(take(data, "orderbook_id"), "cancel_all.orderbook_id", "")
            ),
            closure=(
                None
                if closure is MISSING or closure is None
                else ClosureAck.from_dict(closure, "cancel_all.closure")
            ),
            message=wire_str(take(data, "message"), "cancel_all.message"),
        )


@dataclass(frozen=True, kw_only=True)
class UserOrdersResponse:
    """Response of ``GET /api/users/orders``: one page of the wallet's open and
    pending orders plus one page of its funding accounts.

    A page can be empty while ``has_more`` is true. Funding pages continue via
    ``positions().positions_page(cursor=next_funding_cursor)``, which reads the
    same committed funding view.
    """

    user_pubkey: PubkeyStr
    orders: list[UserOrder] = field(default_factory=list)
    funding_accounts: list[FundingAccount] = field(default_factory=list)
    #: Opaque order cursor (``"<accepted_seq>:<hash>"``).
    next_cursor: str | None
    has_more: bool
    next_funding_cursor: str | None
    funding_has_more: bool
    #: Database revision of this page; later pages may be newer.
    committed_revision: int
    projection_generation: int

    @staticmethod
    def from_dict(value: Any) -> UserOrdersResponse:
        """Decode ``GET /api/users/orders``."""
        data = wire_object(value, "user_orders")
        return UserOrdersResponse(
            user_pubkey=wire_pubkey(
                take(data, "user_pubkey"), "user_orders.user_pubkey"
            ),
            orders=wire_array(
                take(data, "orders"), "user_orders.orders", UserOrder.from_dict
            ),
            funding_accounts=wire_array(
                take(data, "funding_accounts"),
                "user_orders.funding_accounts",
                FundingAccount.from_dict,
                default_empty=True,
            ),
            next_cursor=wire_opt_str(
                take(data, "next_cursor"), "user_orders.next_cursor"
            ),
            has_more=wire_bool(take(data, "has_more"), "user_orders.has_more"),
            next_funding_cursor=wire_opt_str(
                take(data, "next_funding_cursor"), "user_orders.next_funding_cursor"
            ),
            funding_has_more=wire_bool(
                take(data, "funding_has_more"),
                "user_orders.funding_has_more",
                default=False,
            ),
            committed_revision=wire_u64(
                take(data, "committed_revision"), "user_orders.committed_revision"
            ),
            projection_generation=wire_u64(
                take(data, "projection_generation"),
                "user_orders.projection_generation",
            ),
        )


from .convert import (  # noqa: E402 — convert and state import the types above
    convert_snapshot_orders,
    limit_order_from_snapshot,
    limit_order_from_update,
    limit_order_from_user_order,
)
from .state import ApplyOutcome, UserOpenLimitOrders  # noqa: E402

__all__ = [
    # Model
    "LimitOrder",
    "OrderStatus",
    "OrderType",
    # Requests
    "CancelAllBody",
    "CancelBody",
    # Responses
    "CancelAllSuccess",
    "CancelQuantities",
    "CancelStatus",
    "CancelSuccess",
    "ClosureAck",
    "FillInfo",
    "SubmitOrderResponse",
    "SubmitOrderStatus",
    "UserOrdersResponse",
    # REST wire
    "InitialCohort",
    "InitialCohortState",
    "OrderFillEvent",
    "OrderState",
    "RecordedOrderState",
    "Role",
    "UserOrder",
    "UserOrderFill",
    "UserOrderFillsResponse",
    # WS user channel
    "AuthUpdate",
    "ClosureScope",
    "ClosureUpdate",
    "NotificationUpdate",
    "OrderUpdate",
    "RecoveryCompleted",
    "UserSnapshot",
    "UserSnapshotOrder",
    "UserUpdate",
    "UserUpdateData",
    "limit_price",
    # Conversion and state
    "ApplyOutcome",
    "UserOpenLimitOrders",
    "convert_snapshot_orders",
    "limit_order_from_snapshot",
    "limit_order_from_update",
    "limit_order_from_user_order",
]
