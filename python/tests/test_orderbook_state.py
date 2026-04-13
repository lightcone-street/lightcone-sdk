"""Tests for orderbook sequence handling."""

from lightcone_sdk.domain.orderbook.state import OrderbookSnapshot
from lightcone_sdk.domain.orderbook.wire import WsBookLevel, WsOrderBook


def make_book(
    *,
    is_snapshot: bool,
    seq: int,
    bids: list[tuple[str, str]] | None = None,
    asks: list[tuple[str, str]] | None = None,
) -> WsOrderBook:
    return WsOrderBook(
        orderbook_id="ob1",
        is_snapshot=is_snapshot,
        seq=seq,
        bids=[WsBookLevel(side=0, price=price, size=size) for price, size in bids or []],
        asks=[WsBookLevel(side=1, price=price, size=size) for price, size in asks or []],
    )


def test_gap_detection_preserves_existing_book():
    snapshot = OrderbookSnapshot(orderbook_id="ob1")

    assert snapshot.apply(
        make_book(
            is_snapshot=True,
            seq=1,
            bids=[("0.45", "10")],
            asks=[("0.55", "12")],
        )
    ).kind == "applied"

    result = snapshot.apply(
        make_book(
            is_snapshot=False,
            seq=3,
            bids=[("0.46", "9")],
            asks=[("0.54", "8")],
        )
    )

    assert result.kind == "gap_detected"
    assert result.expected == 2
    assert result.got == 3
    assert snapshot.best_bid() == "0.45"
    assert snapshot.best_ask() == "0.55"


def test_stale_delta_is_ignored():
    snapshot = OrderbookSnapshot(orderbook_id="ob1")

    assert snapshot.apply(
        make_book(
            is_snapshot=True,
            seq=1,
            bids=[("0.45", "10")],
            asks=[("0.55", "12")],
        )
    ).kind == "applied"

    assert snapshot.apply(
        make_book(
            is_snapshot=False,
            seq=2,
            bids=[("0.46", "9")],
        )
    ).kind == "applied"

    assert snapshot.sequence == 2

    # Late delta (seq < current) should be ignored
    result = snapshot.apply(
        make_book(
            is_snapshot=False,
            seq=1,
            bids=[("0.44", "5")],
        )
    )
    assert result.kind == "ignored_stale"
    assert snapshot.sequence == 2

    # Duplicate delta (seq == current) should also be ignored
    result = snapshot.apply(
        make_book(
            is_snapshot=False,
            seq=2,
            bids=[("0.44", "5")],
        )
    )
    assert result.kind == "ignored_stale"
    assert snapshot.sequence == 2

    # Book should be unchanged
    assert snapshot.best_bid() == "0.46"
    assert snapshot.best_ask() == "0.55"


def test_snapshot_after_gap_restores_state():
    snapshot = OrderbookSnapshot(orderbook_id="ob1")

    snapshot.apply(
        make_book(
            is_snapshot=True,
            seq=1,
            bids=[("0.45", "10")],
            asks=[("0.55", "12")],
        )
    )
    snapshot.apply(
        make_book(
            is_snapshot=False,
            seq=3,
            bids=[("0.46", "9")],
            asks=[("0.54", "8")],
        )
    )

    result = snapshot.apply(
        make_book(
            is_snapshot=True,
            seq=10,
            bids=[("0.49", "5")],
            asks=[("0.51", "7")],
        )
    )

    assert result.kind == "applied"
    assert snapshot.sequence == 10
    assert snapshot.best_bid() == "0.49"
    assert snapshot.best_ask() == "0.51"
