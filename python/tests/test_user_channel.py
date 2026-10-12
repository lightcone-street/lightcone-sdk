"""Committed WS ``user`` channel: decoding, conversion, and UserOpenLimitOrders."""

import json
from dataclasses import replace
from datetime import UTC, datetime
from decimal import Decimal, getcontext, localcontext

import pytest

from lightcone_sdk.domain.order import (
    ApplyOutcome,
    ClosureScope,
    ClosureUpdate,
    OrderStatus,
    OrderUpdate,
    RecoveryCompleted,
    UserOpenLimitOrders,
    UserSnapshot,
    UserSnapshotOrder,
    UserUpdate,
    convert_snapshot_orders,
    limit_order_from_update,
)
from lightcone_sdk.domain.position.wire import FundingUpdate
from lightcone_sdk.domain.trade import WsTrade
from lightcone_sdk.error import DeserializationError, ValidationError
from lightcone_sdk.shared.types import FundingSource, Side, TimeInForce
from lightcone_sdk.ws import WsEventType, parse_message_in
from lightcone_sdk.ws.client import WsClient

WALLET = "Wallet1111111111111111111111111111111111111"
HASH = "4f1a4b1ab1c0c0ffee0000000000000000000000000000000000000000000001"
MARKET = "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9"
BOOK = "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a"


def user(data: dict) -> UserUpdate:
    """Decode a ``user`` frame through the WebSocket entry point."""
    message = parse_message_in(
        json.dumps({"type": "user", "version": 0.1, "data": data})
    )
    assert message.type == "user"
    assert isinstance(message.data, UserUpdate)
    return message.data


def live_order_fact(revision, open_base, pending_base, closed_reason=None):
    """Live order fact as ``redis_listener::handle_committed`` relays it:
    envelope numbers, numeric side, scaled quantities."""
    return {
        "effect_id": f"{revision}:3",
        "committed_revision": revision,
        "projection_generation": 1,
        "actionable": True,
        "event_type": "order",
        "user_pubkey": WALLET,
        "order_hash": HASH,
        "market_pubkey": MARKET,
        "orderbook_id": BOOK,
        "base_mint": "BaseMint111111111111111111111111111111111111",
        "quote_mint": "QuoteMint11111111111111111111111111111111111",
        "side": 0,
        "maker_amount": "5.500000",
        "taker_amount": "10.00000000",
        "original_base": "10.00000000",
        "confirmed_base": "2.00000000",
        "pending_base": pending_base,
        "open_base": open_base,
        "executable_base": open_base,
        "cancelled_base": "0.00000000",
        "accepted_seq": 44,
        "accepted_at": "2026-09-29T12:00:00+00:00",
        "expiration": 0,
        "tif": "GTC",
        "funding_source": "global",
        "funding_account": "Acct1111111111111111111111111111111111111111",
        "closed_reason": closed_reason,
        "ready": True,
    }


def live_order(revision, open_base, pending_base, closed_reason=None) -> OrderUpdate:
    update = user(live_order_fact(revision, open_base, pending_base, closed_reason))
    assert update.event_type == "order"
    assert isinstance(update.data, OrderUpdate)
    return update.data


# ── Decoding ─────────────────────────────────────────────────────────────────


def test_decodes_a_live_order_fact_with_a_numeric_side_and_envelope_numbers():
    order = live_order(812, "7.00000000", "1.00000000")
    assert order.commit.committed_revision == 812
    assert order.commit.projection_generation == 1
    assert order.commit.actionable is True
    assert order.commit.effect_id == "812:3"
    assert order.side is Side.BID
    assert order.accepted_seq == 44
    assert order.accepted_at == datetime(2026, 9, 29, 12, tzinfo=UTC)
    assert order.open_base == Decimal("7.00000000")
    assert order.funding_source is FundingSource.GLOBAL
    assert order.tif is TimeInForce.GTC
    assert order.price() == Decimal("0.55")
    assert order.ready is True
    assert order.closed_reason is None


def test_decodes_a_snapshot_with_string_revisions_and_empty_markers():
    update = user(
        {
            "event_type": "snapshot",
            "orders": [
                {
                    "order_hash": HASH,
                    "market_pubkey": MARKET,
                    "orderbook_id": BOOK,
                    "side": "ask",
                    "maker_amount": "4.00000000",
                    "taker_amount": "2.200000",
                    "original_base": "4.00000000",
                    "confirmed_base": "0.00000000",
                    "pending_base": "0.00000000",
                    "open_base": "4.00000000",
                    "cancelled_base": "0.00000000",
                    "closed_reason": "",
                    "committed_revision": "812",
                    "accepted_seq": "45",
                    "price": "0.550000",
                    "created_at": 1790685521784,
                    "expiration": 0,
                    "base_mint": "BaseMint111111111111111111111111111111111111",
                    "quote_mint": "QuoteMint11111111111111111111111111111111111",
                    "tif": "IOC",
                    "funding_source": "conditional",
                }
            ],
            "funding_accounts": [],
            "next_cursor": "",
            "has_more": False,
            "next_funding_cursor": "",
            "funding_has_more": False,
            "committed_revision": "812",
            "projection_generation": "1",
            "notifications": [],
        }
    )
    assert update.event_type == "snapshot"
    snapshot = update.data
    assert isinstance(snapshot, UserSnapshot)
    assert snapshot.committed_revision == 812
    assert snapshot.next_cursor is None
    assert snapshot.next_funding_cursor is None
    order = snapshot.orders[0]
    assert order.side is Side.ASK
    assert order.closed_reason is None
    assert order.accepted_seq == 45
    assert order.tif is TimeInForce.IOC
    assert order.funding_source is FundingSource.CONDITIONAL


def test_snapshot_defaults_only_absent_fields():
    snapshot = UserSnapshot.from_dict(
        {
            "orders": [],
            "funding_accounts": [],
            "committed_revision": "1",
            "projection_generation": "1",
        }
    )
    assert snapshot.notifications == []
    assert snapshot.has_more is False
    for key in ("notifications", "has_more"):
        with pytest.raises(DeserializationError, match=rf"snapshot\.{key}"):
            UserSnapshot.from_dict(
                {
                    "orders": [],
                    "funding_accounts": [],
                    "committed_revision": "1",
                    "projection_generation": "1",
                    key: None,
                }
            )


def test_keeps_a_funding_facts_wide_aggregates_exact():
    update = user(
        {
            "effect_id": "7:2",
            "committed_revision": 7,
            "projection_generation": 1,
            "actionable": False,
            "event_type": "funding",
            "user_pubkey": WALLET,
            "account": "Acct1111111111111111111111111111111111111111",
            "mint": "Cond1111111111111111111111111111111111111111",
            "source": "conditional",
            "market_pubkey": MARKET,
            "raw_observed": "18446744073709.551615",
            "usable_observed": "1.000000",
            "order_reserved": "340282366920938463463374607431768.211455",
            "execution_reserved": "0.000001",
            "signed_remaining": "-0.000001",
            "available": "0.000000",
            "ready": False,
            "accepted_boundary": "boundary",
            "slot": "7",
            "blockhash": None,
        }
    )
    assert update.event_type == "funding"
    funding = update.data
    assert isinstance(funding, FundingUpdate)
    assert funding.commit.actionable is False
    assert funding.ready is False
    assert str(funding.available) == "0.000000"
    assert funding.raw_observed == Decimal("18446744073709.551615")
    assert funding.order_reserved == "340282366920938463463374607431768.211455"
    assert funding.signed_remaining is not None
    assert funding.signed_remaining.is_negative()
    assert funding.signed_remaining.to_decimal() == Decimal("-0.000001")
    assert funding.slot == 7
    assert funding.blockhash is None


def test_decodes_a_global_funding_fact_without_a_market():
    update = user(
        {
            "effect_id": "8:0",
            "committed_revision": 8,
            "projection_generation": 1,
            "actionable": True,
            "event_type": "funding",
            "user_pubkey": WALLET,
            "account": "Glob1111111111111111111111111111111111111111",
            "mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
            "source": "global",
            "market_pubkey": None,
            "raw_observed": None,
            "usable_observed": None,
            "order_reserved": "0.000000",
            "execution_reserved": "0.000000",
            "signed_remaining": None,
            "available": "0.000000",
            "ready": False,
            "accepted_boundary": None,
            "slot": None,
            "blockhash": None,
        }
    )
    funding = update.data
    assert isinstance(funding, FundingUpdate)
    assert funding.source is FundingSource.GLOBAL
    assert funding.market_pubkey is None
    assert funding.raw_observed is None
    assert funding.slot is None


def test_decodes_closure_and_recovery_facts():
    closure = user(
        {
            "effect_id": "9:0",
            "committed_revision": 9,
            "projection_generation": 1,
            "actionable": True,
            "event_type": "closure",
            "scope_kind": 5,
            "scope_key": f"{WALLET}:{BOOK}",
            "accepted_seq": 44,
            "reason": "cancel_all",
        }
    )
    assert closure.event_type == "closure"
    assert isinstance(closure.data, ClosureUpdate)
    assert closure.data.scope() is ClosureScope.WALLET_BOOK
    assert closure.data.accepted_seq == 44

    recovery = user(
        {
            "effect_id": "10:0",
            "committed_revision": 10,
            "projection_generation": 2,
            "actionable": True,
            "event_type": "recovery_completed",
            "recovered_generation": 2,
        }
    )
    assert recovery.event_type == "recovery_completed"
    assert isinstance(recovery.data, RecoveryCompleted)
    assert recovery.data.recovered_generation == 2
    assert recovery.data.commit.projection_generation == 2


def test_delivers_a_user_fill_with_the_trades_payload():
    update = UserUpdate.from_dict(
        {
            "effect_id": "812:1",
            "committed_revision": 812,
            "projection_generation": 3,
            "actionable": True,
            "event_type": "fill",
            "fill_id": "f",
            "execution_id": "e",
            "evidence_id": "v",
            "event_index": 2,
            "leg_index": 0,
            "orderbook_id": BOOK,
            "market_pubkey": MARKET,
            "base_mint": "b",
            "quote_mint": "q",
            "base_amount": "4",
            "quote_amount": "2.2",
            "maker_order_hash": "aa",
            "taker_order_hash": "bb",
            "maker_wallet": "m",
            "taker_wallet": "t",
            "taker_side": 1,
            "maker_fee_estimate": "0",
            "taker_fee_estimate": "0",
            "fee_mint": "q",
            "maker_fee_bps": 0,
            "taker_fee_bps": 0,
            "executed_at": "2026-09-29T12:00:00Z",
        }
    )
    assert update.event_type == "fill"
    assert isinstance(update.data, WsTrade)
    assert update.data.taker_side is Side.ASK
    assert update.data.trade_id() == "e:0:3"


def test_decodes_notifications():
    update = user(
        {
            "event_type": "notification",
            "notification": {
                "id": "n1",
                "notification_type": "global",
                "title": "Maintenance",
                "message": "Soon",
            },
        }
    )
    assert update.event_type == "notification"
    assert update.data.notification.id == "n1"
    with pytest.raises(DeserializationError, match=r"user\.notification\.notification"):
        user({"event_type": "notification", "notification": None})


@pytest.mark.parametrize(
    "event_type",
    ["nonce", "market_balance_update", "global_deposit_update", "future_event"],
)
def test_decodes_removed_and_future_user_events_as_unknown(event_type):
    update = user(
        {"event_type": event_type, "new_nonce": 3, "timestamp": "2026-06-19T12:34:56Z"}
    )
    assert update == UserUpdate(
        event_type="unknown", data=None, raw_event_type=event_type
    )


@pytest.mark.parametrize(
    ("data", "field"),
    [
        ({"event_type": 3}, r"user\.event_type"),
        ({}, r"user\.event_type"),
        ([], r"^user: expected object"),
        (
            {**live_order_fact(1, "1", "0"), "accepted_seq": "-1"},
            r"user\.order\.accepted_seq",
        ),
        (
            {**live_order_fact(1, "1", "0"), "committed_revision": True},
            r"committed_revision",
        ),
        (
            {**live_order_fact(1, "1", "0"), "committed_revision": 812.0},
            r"committed_revision",
        ),
        ({**live_order_fact(1, "1", "0"), "side": 2}, r"user\.order\.side"),
        ({**live_order_fact(1, "1", "0"), "side": True}, r"user\.order\.side"),
        ({**live_order_fact(1, "1", "0"), "tif": "ALO"}, r"user\.order\.tif"),
        ({**live_order_fact(1, "1", "0"), "tif": "gtc"}, r"user\.order\.tif"),
        ({**live_order_fact(1, "1", "0"), "ready": None}, r"user\.order\.ready"),
        (
            {**live_order_fact(1, "1", "0"), "accepted_at": "2026-09-29T12:00:00"},
            r"accepted_at",
        ),
    ],
)
def test_surfaces_a_malformed_fact_as_a_field_named_decode_error(data, field):
    with pytest.raises(DeserializationError, match=field):
        UserUpdate.from_dict(data)


def test_parse_message_in_rejects_non_object_user_data_and_non_json():
    with pytest.raises(DeserializationError, match="user"):
        parse_message_in(json.dumps({"type": "user", "data": [1]}))
    with pytest.raises(DeserializationError, match="invalid JSON WebSocket frame"):
        parse_message_in('{"type":"user","data":{"committed_revision":NaN}}')
    with pytest.raises(DeserializationError, match="frame"):
        parse_message_in("[]")


def test_ws_client_emits_an_error_event_for_a_malformed_frame_and_keeps_going():
    client = WsClient()
    events = []
    client.on(events.append)
    client._handle_text(json.dumps({"type": "user", "data": {"event_type": 1}}))
    client._handle_text(
        json.dumps({"type": "user", "data": live_order_fact(5, "1.0", "0")})
    )
    assert [event.type for event in events] == [WsEventType.ERROR, WsEventType.MESSAGE]
    assert events[0].error.startswith("Deserialization error: user.event_type")
    assert events[0].message is None
    assert isinstance(events[1].message.data.data, OrderUpdate)


# ── Conversion ───────────────────────────────────────────────────────────────


def snapshot_order(
    order_hash: str, open_base: int, pending_base: int
) -> UserSnapshotOrder:
    return UserSnapshotOrder(
        order_hash=order_hash,
        market_pubkey="mkt1",
        orderbook_id="ob1",
        side=Side.ASK,
        maker_amount=Decimal(10),
        taker_amount=Decimal(5),
        original_base=Decimal(10),
        confirmed_base=Decimal(10 - open_base - pending_base),
        pending_base=Decimal(pending_base),
        open_base=Decimal(open_base),
        cancelled_base=Decimal(0),
        closed_reason=None,
        committed_revision=3,
        accepted_seq=1,
        price=Decimal("0.5"),
        created_at=datetime.fromtimestamp(1_790_685_521.784, UTC),
        expiration=0,
        base_mint="b",
        quote_mint="q",
        tif=TimeInForce.GTC,
        funding_source=FundingSource.CONDITIONAL,
    )


def test_converts_a_live_order_facts_quantities_and_price():
    order = limit_order_from_update(live_order(812, "7.00000000", "1.00000000"))
    assert order.order_hash == HASH
    assert order.side is Side.BID
    assert order.price == Decimal("0.55")
    assert order.size == Decimal("10.00000000")
    assert order.filled_size == Decimal("2.00000000")
    assert order.pending_size == Decimal("1.00000000")
    assert order.remaining_size == Decimal("7.00000000")
    assert order.status is OrderStatus.PENDING
    assert order.committed_revision == 812
    assert order.accepted_seq == 44
    assert order.funding_source is FundingSource.GLOBAL
    assert order.time_in_force is TimeInForce.GTC
    assert order.created_at == datetime(2026, 9, 29, 12, tzinfo=UTC)


def test_a_closed_reason_wins_status_precedence():
    order = limit_order_from_update(
        live_order(813, "0.00000000", "1.00000000", "expired")
    )
    assert order.status is OrderStatus.CLOSED
    assert order.closed_reason == "expired"


@pytest.mark.parametrize(
    ("closed", "original", "confirmed", "pending", "status"),
    [
        ("cancelled", "10", "10", "0", OrderStatus.CLOSED),
        (None, "10", "10.000", "0", OrderStatus.FILLED),
        (None, "10", "4", "2", OrderStatus.PENDING),
        (None, "10", "4", "0", OrderStatus.OPEN),
    ],
)
def test_status_precedence(closed, original, confirmed, pending, status):
    assert (
        OrderStatus.derive(
            closed, Decimal(original), Decimal(confirmed), Decimal(pending)
        )
        is status
    )


def test_rejects_a_live_order_fact_without_a_limit_price():
    update = replace(
        live_order(814, "7.00000000", "0.00000000"), taker_amount=Decimal(0)
    )
    with pytest.raises(ValidationError, match="limit price"):
        limit_order_from_update(update)
    open_orders = UserOpenLimitOrders()
    with pytest.raises(ValidationError):
        open_orders.apply(update)
    assert open_orders.is_empty()


def test_keeps_only_live_orders_from_a_snapshot():
    open_orders = convert_snapshot_orders(
        [
            snapshot_order("resting", 4, 0),
            snapshot_order("claiming", 0, 2),
            snapshot_order("done", 0, 0),
        ]
    )
    orders = open_orders.get("mkt1", "ob1")
    assert [order.order_hash for order in orders] == ["resting", "claiming"]
    assert orders[0].status is OrderStatus.OPEN
    assert orders[1].status is OrderStatus.PENDING


# ── UserOpenLimitOrders ──────────────────────────────────────────────────────


def tracked(container: UserOpenLimitOrders):
    return container.get(MARKET, BOOK) or []


def closure(scope_kind: int, scope_key: str, accepted_seq: int) -> ClosureUpdate:
    return ClosureUpdate.from_dict(
        {
            "effect_id": "900:0",
            "committed_revision": 900,
            "projection_generation": 1,
            "actionable": True,
            "scope_kind": scope_kind,
            "scope_key": scope_key,
            "accepted_seq": accepted_seq,
            "reason": "cancel_all",
        }
    )


def test_inserts_updates_and_removes_by_liveness():
    container = UserOpenLimitOrders()
    assert (
        container.apply(live_order(10, "8.00000000", "0.00000000"))
        is ApplyOutcome.INSERTED
    )
    assert (
        container.apply(live_order(11, "5.00000000", "3.00000000"))
        is ApplyOutcome.UPDATED
    )
    orders = tracked(container)
    assert len(orders) == 1
    assert orders[0].remaining_size == Decimal("5.00000000")
    assert orders[0].pending_size == Decimal("3.00000000")
    assert orders[0].side is Side.BID

    # Pending fills keep a fully matched order live until confirmation.
    assert (
        container.apply(live_order(12, "0.00000000", "3.00000000"))
        is ApplyOutcome.UPDATED
    )
    assert (
        container.apply(live_order(13, "0.00000000", "0.00000000"))
        is ApplyOutcome.REMOVED
    )
    assert container.is_empty()
    assert (
        container.apply(live_order(14, "0.00000000", "0.00000000"))
        is ApplyOutcome.IGNORED
    )


def test_ignores_older_revisions():
    container = UserOpenLimitOrders()
    container.apply(live_order(20, "5.00000000", "0.00000000"))
    assert (
        container.apply(live_order(19, "8.00000000", "0.00000000"))
        is ApplyOutcome.STALE
    )
    assert tracked(container)[0].remaining_size == Decimal("5.00000000")


def test_closes_orders_in_scope_up_to_the_cutoff():
    container = UserOpenLimitOrders()
    container.apply(live_order(10, "8.00000000", "0.00000000"))
    # Cutoff below the order's accepted_seq (44) leaves it resting.
    assert container.apply_closure(closure(2, BOOK, 43)) == 0
    assert len(tracked(container)) == 1
    # Other books and markets are out of scope.
    assert container.apply_closure(closure(2, "OtherBook", 44)) == 0
    assert container.apply_closure(closure(1, "OtherMarket", 44)) == 0
    # A wallet-book key without a separator matches nothing.
    assert container.apply_closure(closure(5, BOOK, 44)) == 0
    assert container.apply_closure(closure(5, f"wallet:{BOOK}", 44)) == 1
    assert container.is_empty()


def test_keeps_pending_claims_visible_after_a_closure():
    container = UserOpenLimitOrders()
    container.apply(live_order(10, "5.00000000", "3.00000000"))
    assert container.apply_closure(closure(4, "wallet", 50)) == 1
    order = tracked(container)[0]
    assert order.remaining_size == 0
    assert order.cancelled_size == Decimal("5.00000000")
    assert order.status is OrderStatus.CLOSED
    assert order.closed_reason == "cancel_all"
    assert order.committed_revision == 900
    # A fact older than the closure cannot reopen the order.
    assert (
        container.apply(live_order(11, "5.00000000", "3.00000000"))
        is ApplyOutcome.STALE
    )


def test_closes_exactly_independent_of_the_decimal_context():
    with localcontext() as context:
        context.prec = 6
        assert getcontext().prec == 6
        container = UserOpenLimitOrders()
        fact = live_order(10, "12344.678912345", "1.00000000")
        container.apply(replace(fact, cancelled_base=Decimal("0.000000001")))
        assert container.apply_closure(closure(4, "wallet", 50)) == 1
        cancelled = tracked(container)[0].cancelled_size
    assert str(cancelled) == "12344.678912346"


def test_replaces_closed_orders_instead_of_changing_held_references():
    container = UserOpenLimitOrders()
    container.apply(live_order(10, "5.00000000", "3.00000000"))
    held_list = tracked(container)
    held_order = held_list[0]
    assert container.apply_closure(closure(4, "wallet", 50)) == 1
    assert held_order.remaining_size == Decimal("5.00000000")
    assert held_order.status is OrderStatus.PENDING
    assert held_list[0] is held_order
    assert tracked(container)[0] is not held_order
    assert tracked(container)[0].status is OrderStatus.CLOSED

    everything = container.all()
    container.remove(HASH)
    assert len(everything) == 1
    assert held_list == [held_order]


def test_does_not_let_older_state_reopen_an_order_a_closure_removed():
    container = UserOpenLimitOrders()
    container.apply(live_order(10, "8.00000000", "0.00000000"))
    assert container.apply_closure(closure(4, "wallet", 50)) == 1
    assert container.is_empty()
    # A REST page or snapshot captured before the closure arrives late.
    assert (
        container.apply(live_order(11, "8.00000000", "0.00000000"))
        is ApplyOutcome.STALE
    )
    assert container.is_empty()


def test_does_not_let_older_state_reopen_a_removed_order():
    container = UserOpenLimitOrders()
    container.apply(live_order(10, "8.00000000", "0.00000000"))
    assert (
        container.apply(live_order(12, "0.00000000", "0.00000000", "cancelled"))
        is ApplyOutcome.REMOVED
    )
    assert (
        container.apply(live_order(11, "8.00000000", "0.00000000"))
        is ApplyOutcome.STALE
    )
    assert (
        container.apply(live_order(12, "8.00000000", "0.00000000"))
        is ApplyOutcome.STALE
    )
    assert container.is_empty()
    # Newer committed state stays authoritative.
    assert (
        container.apply(live_order(13, "8.00000000", "0.00000000"))
        is ApplyOutcome.INSERTED
    )


def test_applies_a_closure_to_older_state_seeded_after_it():
    container = UserOpenLimitOrders()
    assert container.apply_closure(closure(4, "wallet", 50)) == 0
    assert (
        container.apply(live_order(11, "8.00000000", "0.00000000"))
        is ApplyOutcome.IGNORED
    )
    assert container.is_empty()

    # Pending fills keep the closed order visible, as with a tracked order.
    container.clear()
    container.apply_closure(closure(4, "wallet", 50))
    assert (
        container.apply(live_order(11, "5.00000000", "3.00000000"))
        is ApplyOutcome.INSERTED
    )
    order = tracked(container)[0]
    assert order.remaining_size == 0
    assert order.cancelled_size == Decimal("5.00000000")
    assert order.status is OrderStatus.CLOSED
    assert order.committed_revision == 900

    # Orders accepted after the cutoff, and newer state, are unaffected.
    container.clear()
    container.apply_closure(closure(4, "wallet", 43))
    assert (
        container.apply(live_order(11, "8.00000000", "0.00000000"))
        is ApplyOutcome.INSERTED
    )
    container.clear()
    container.apply_closure(closure(4, "wallet", 50))
    assert (
        container.apply(live_order(901, "8.00000000", "0.00000000"))
        is ApplyOutcome.INSERTED
    )


def test_forgets_retired_orders_and_closures_on_clear():
    container = UserOpenLimitOrders()
    container.apply(live_order(10, "8.00000000", "0.00000000"))
    container.apply_closure(closure(4, "wallet", 50))
    container.clear()
    assert (
        container.apply(live_order(11, "8.00000000", "0.00000000"))
        is ApplyOutcome.INSERTED
    )


@pytest.mark.parametrize("scope_kind", [3, 6, 7, 99, -1])
def test_requests_a_refresh_for_closure_scopes_without_order_data(scope_kind):
    container = UserOpenLimitOrders()
    container.apply(live_order(10, "5.00000000", "0.00000000"))
    assert container.apply_closure(closure(scope_kind, "x", 50)) is None
    assert len(tracked(container)) == 1


def test_removes_and_clears():
    container = UserOpenLimitOrders()
    container.apply(live_order(10, "5.00000000", "0.00000000"))
    assert container.get_by_hash(HASH) is not None
    removed = container.remove(HASH)
    assert removed is not None and removed.order_hash == HASH
    assert container.is_empty()
    assert container.remove(HASH) is None
    container.apply(live_order(11, "5.00000000", "0.00000000"))
    container.clear()
    assert container.is_empty()


def test_never_mutates_the_callers_order():
    container = UserOpenLimitOrders()
    container.apply_closure(closure(4, "wallet", 50))
    order = limit_order_from_update(live_order(11, "5.00000000", "3.00000000"))
    container.apply_order(order)
    assert order.remaining_size == Decimal("5.00000000")
    assert order.status is OrderStatus.PENDING
    assert tracked(container)[0] is not order
