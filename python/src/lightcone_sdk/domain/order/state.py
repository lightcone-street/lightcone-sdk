"""Open-order container for committed ``user`` channel state."""

from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass, field, replace
from decimal import Decimal
from enum import Enum

from ...shared.price import add_decimals
from ...shared.types import OrderBookId, PubkeyStr
from .convert import limit_order_from_update
from .model import LimitOrder, OrderStatus
from .wire import ClosureScope, ClosureUpdate, OrderUpdate

# Closure scopes that cannot be evaluated from order data alone.
_UNEVALUATED_SCOPES = frozenset(
    {ClosureScope.DEPOSIT_TOKEN, ClosureScope.ACCOUNT, ClosureScope.SETTLEMENT_PROFILE}
)


class ApplyOutcome(str, Enum):
    """Result of applying committed order state to :class:`UserOpenLimitOrders`."""

    #: A live order that was not tracked yet.
    INSERTED = "inserted"
    #: A tracked order replaced by newer (or equal) committed state.
    UPDATED = "updated"
    #: A tracked order that is no longer live was removed.
    REMOVED = "removed"
    #: Older than the tracked state (e.g. a live fact racing a snapshot).
    STALE = "stale"
    #: Neither live nor tracked; nothing changed.
    IGNORED = "ignored"


@dataclass
class UserOpenLimitOrders:
    """A wallet's live limit orders (resting, or with fills awaiting on-chain
    confirmation), grouped ``market_pubkey -> orderbook_id``.

    Seed it from the WS ``user`` snapshot (:func:`convert_snapshot_orders`) or
    REST pages, then :meth:`apply` every live ``order`` fact. Each fact carries
    the order's complete state, so application is a revision-guarded replace.
    The container remembers the revision at which each order stopped being
    live, and every applied closure, so older state (a REST page or snapshot
    racing live facts) cannot reopen a closed order.

    Stored orders and per-orderbook lists are never changed in place: every
    change replaces them, so a reference returned earlier keeps the state it
    was read at.
    """

    orders: dict[PubkeyStr, dict[OrderBookId, list[LimitOrder]]] = field(
        default_factory=dict
    )
    #: Committed revision at which each untracked order stopped being live.
    _retired: dict[str, int] = field(default_factory=dict, repr=False)
    #: Applied closures, re-applied to older state inserted later.
    _closures: list[tuple[ClosureScope, ClosureUpdate]] = field(
        default_factory=list, repr=False
    )

    def get(
        self, market_pubkey: PubkeyStr, orderbook_id: OrderBookId
    ) -> list[LimitOrder] | None:
        market_orders = self.orders.get(market_pubkey)
        if market_orders is None:
            return None
        return market_orders.get(orderbook_id)

    def get_by_market(
        self, market_pubkey: PubkeyStr
    ) -> dict[OrderBookId, list[LimitOrder]] | None:
        return self.orders.get(market_pubkey)

    def get_by_hash(self, order_hash: str) -> LimitOrder | None:
        """Find a tracked order by hash."""
        for order in self._iter():
            if order.order_hash == order_hash:
                return order
        return None

    def all(self) -> list[LimitOrder]:
        """Every tracked order."""
        return list(self._iter())

    def apply(self, update: OrderUpdate) -> ApplyOutcome:
        """Apply a live WS ``order`` fact.

        Raises :class:`~lightcone_sdk.error.ValidationError`, changing
        nothing, when the fact's signed amounts define no limit price.
        """
        return self.apply_order(limit_order_from_update(update))

    def apply_order(self, source: LimitOrder) -> ApplyOutcome:
        """Apply converted committed order state (live fact, snapshot, or REST).

        State older than the tracked ``committed_revision``, or not newer than
        the revision at which the order stopped being live, is ignored. A
        closure committed after the state still closes it. A live order
        replaces the tracked copy; one that is no longer live is removed. The
        order is copied, never mutated.
        """
        order = replace(source)
        existing = self.get_by_hash(order.order_hash)
        if existing is not None:
            if order.committed_revision < existing.committed_revision:
                return ApplyOutcome.STALE
        else:
            retired_at = self._retired.get(order.order_hash)
            if retired_at is not None and order.committed_revision <= retired_at:
                return ApplyOutcome.STALE
        for scope, closure in self._closures:
            if closure.commit.committed_revision > order.committed_revision:
                closed = _close_covered(order, closure, scope)
                if closed is not None:
                    order = closed
        was_tracked = self._take(order.order_hash) is not None
        if not order.is_live():
            self._retire(order)
            return ApplyOutcome.REMOVED if was_tracked else ApplyOutcome.IGNORED
        self._retired.pop(order.order_hash, None)
        by_orderbook = self.orders.setdefault(order.market_pubkey, {})
        by_orderbook[order.orderbook_id] = [
            *by_orderbook.get(order.orderbook_id, []),
            order,
        ]
        return ApplyOutcome.UPDATED if was_tracked else ApplyOutcome.INSERTED

    def apply_closure(self, closure: ClosureUpdate) -> int | None:
        """Apply a committed closure cutoff locally.

        Orders in scope with ``accepted_seq <= closure.accepted_seq`` stop
        resting (their open base becomes cancelled) and are removed unless
        fills still await confirmation. The container is assumed to hold one
        wallet's orders.

        Returns the number of orders closed, or ``None`` when the scope cannot
        be evaluated from order data (deposit-token, account,
        settlement-profile, or unknown scopes): refetch the orders in that
        case. Later per-order facts remain authoritative.
        """
        scope = closure.scope()
        if scope is None or scope in _UNEVALUATED_SCOPES:
            return None
        closed = 0
        for by_orderbook in self.orders.values():
            for orderbook_id, orders in list(by_orderbook.items()):
                kept: list[LimitOrder] = []
                for order in orders:
                    closed_order = _close_covered(order, closure, scope)
                    if closed_order is None:
                        kept.append(order)
                        continue
                    closed += 1
                    if closed_order.is_live():
                        kept.append(closed_order)
                    else:
                        self._retire(closed_order)
                by_orderbook[orderbook_id] = kept
        self._closures.append((scope, closure))
        return closed

    def remove(self, order_hash: str) -> LimitOrder | None:
        """Stop tracking an order without recording why; older state can add it
        again. Returns the removed order, if it was tracked."""
        return self._take(order_hash)

    def clear(self) -> None:
        """Forget every order, retired revision, and closure (e.g. before
        reseeding from a new snapshot)."""
        self.orders.clear()
        self._retired.clear()
        self._closures.clear()

    def is_empty(self) -> bool:
        return all(
            len(orders) == 0
            for by_orderbook in self.orders.values()
            for orders in by_orderbook.values()
        )

    def _iter(self) -> Iterator[LimitOrder]:
        for by_orderbook in self.orders.values():
            for orders in by_orderbook.values():
                yield from orders

    def _take(self, order_hash: str) -> LimitOrder | None:
        for by_orderbook in self.orders.values():
            for orderbook_id, orders in by_orderbook.items():
                for index, order in enumerate(orders):
                    if order.order_hash == order_hash:
                        by_orderbook[orderbook_id] = [
                            *orders[:index],
                            *orders[index + 1 :],
                        ]
                        return order
        return None

    def _retire(self, order: LimitOrder) -> None:
        """Record the revision at which ``order`` stopped being live."""
        previous = self._retired.get(order.order_hash)
        if previous is None or order.committed_revision > previous:
            self._retired[order.order_hash] = order.committed_revision


def _close_covered(
    order: LimitOrder, closure: ClosureUpdate, scope: ClosureScope
) -> LimitOrder | None:
    """A closed copy of ``order`` when ``closure`` covers it (in scope,
    accepted at or before the cutoff, and still resting); ``None`` otherwise.
    Never mutates ``order``."""
    key = closure.scope_key
    if scope == ClosureScope.MARKET:
        in_scope = order.market_pubkey == key
    elif scope == ClosureScope.BOOK:
        in_scope = order.orderbook_id == key
    elif scope == ClosureScope.WALLET_BOOK:
        _wallet, separator, book = key.partition(":")
        in_scope = separator != "" and order.orderbook_id == book
    else:
        in_scope = True
    if not (
        order.accepted_seq <= closure.accepted_seq
        and in_scope
        and order.remaining_size > 0
    ):
        return None
    return replace(
        order,
        cancelled_size=add_decimals(order.cancelled_size, order.remaining_size),
        remaining_size=Decimal(0),
        closed_reason=(
            order.closed_reason if order.closed_reason is not None else closure.reason
        ),
        status=OrderStatus.CLOSED,
        # Older order facts must not reopen the closed order.
        committed_revision=max(
            order.committed_revision, closure.commit.committed_revision
        ),
    )


__all__ = ["ApplyOutcome", "UserOpenLimitOrders"]
