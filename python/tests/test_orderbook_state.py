"""Tests for generation-gated snapshot-only orderbook state."""

from lightcone_sdk.domain.orderbook.state import OrderbookState
from lightcone_sdk.domain.orderbook.aggregation import BookAggregation
from lightcone_sdk.domain.orderbook.wire import WsBookLevel, WsOrderBook


def make_book(
    *,
    is_snapshot: bool,
    seq: int,
    bids: list[tuple[str, str]] | None = None,
    asks: list[tuple[str, str]] | None = None,
    resync: bool = False,
) -> WsOrderBook:
    return WsOrderBook(
        orderbook_id="ob1",
        is_snapshot=is_snapshot,
        seq=seq,
        resync=resync,
        bids=[WsBookLevel(side=0, price=price, size=size) for price, size in bids or []],
        asks=[WsBookLevel(side=1, price=price, size=size) for price, size in asks or []],
    )


def test_snapshot_replaces_state():
    snapshot = OrderbookState(orderbook_id="ob1")

    assert snapshot.apply(
        make_book(is_snapshot=True, seq=1, bids=[("0.45", "10")], asks=[("0.55", "12")])
    ).kind == "applied"
    assert snapshot.best_bid() == "0.45"
    assert snapshot.best_ask() == "0.55"

    assert snapshot.apply(
        make_book(is_snapshot=True, seq=2, bids=[("0.44", "20")], asks=[("0.56", "8")])
    ).kind == "applied"
    assert len(snapshot.bids) == 1
    assert len(snapshot.asks) == 1
    assert snapshot.best_bid() == "0.44"
    assert snapshot.best_ask() == "0.56"


def test_lower_and_equal_revisions_are_discarded():
    snapshot = OrderbookState(orderbook_id="ob1")
    snapshot.apply(make_book(is_snapshot=True, seq=42, bids=[("0.45", "10")]))
    assert snapshot.sequence == 42

    assert snapshot.apply(
        make_book(is_snapshot=True, seq=7, bids=[("0.44", "20")])
    ).kind == "discarded_stale"
    assert snapshot.apply(make_book(is_snapshot=True, seq=42)).kind == "discarded_stale"
    assert snapshot.sequence == 42
    assert snapshot.best_bid() == "0.45"


def test_lower_fresh_revision_applies_after_resubscribe():
    snapshot = OrderbookState(orderbook_id="ob1")
    snapshot.apply(
        make_book(is_snapshot=True, seq=42, bids=[("0.45", "10")], asks=[("0.55", "12")])
    )

    result = snapshot.apply(make_book(is_snapshot=False, seq=0, resync=True))
    assert result.kind == "refresh_required"
    assert result.reason is not None
    assert result.reason.kind == "server_resync"
    # Resync leaves the book untouched.
    assert snapshot.sequence == 42
    assert snapshot.best_bid() == "0.45"

    # A lower revision is valid after the generation gate resets.
    snapshot.begin_generation()
    assert snapshot.apply(
        make_book(is_snapshot=True, seq=7, bids=[("0.43", "5")], asks=[("0.57", "2")])
    ).kind == "applied"
    assert snapshot.sequence == 7
    assert snapshot.best_bid() == "0.43"
    assert snapshot.best_ask() == "0.57"


def test_data_frames_replace_regardless_of_snapshot_flag():
    snapshot = OrderbookState(orderbook_id="ob1")
    snapshot.apply(
        make_book(is_snapshot=True, seq=1, bids=[("0.45", "10")], asks=[("0.55", "12")])
    )

    # Every non-resync data frame is a snapshot by contract — the is_snapshot
    # flag is not consulted, so a server omitting it cannot freeze the book.
    result = snapshot.apply(make_book(is_snapshot=False, seq=2, bids=[("0.46", "9")]))
    assert result.kind == "applied"
    assert snapshot.sequence == 2
    assert snapshot.best_bid() == "0.46"
    assert snapshot.best_ask() is None


def test_zero_size_levels_are_skipped():
    snapshot = OrderbookState(orderbook_id="ob1")
    assert snapshot.apply(
        make_book(
            is_snapshot=True,
            seq=1,
            bids=[("0.45", "10"), ("0.44", "0")],
            asks=[("0.55", "12")],
        )
    ).kind == "applied"
    assert len(snapshot.bids) == 1
    assert snapshot.best_bid() == "0.45"


def test_dict_updates_apply_like_typed():
    snapshot = OrderbookState(orderbook_id="ob1")
    assert snapshot.apply(
        {
            "orderbook_id": "ob1",
            "is_snapshot": True,
            "seq": 0,
            "bids": [{"price": "0.45", "size": "10"}],
            "asks": [{"price": "0.55", "size": "12"}],
        }
    ).kind == "applied"
    assert snapshot.best_bid() == "0.45"

    assert snapshot.apply({"orderbook_id": "ob1", "resync": True}).kind == "refresh_required"
    assert snapshot.best_bid() == "0.45"
    snapshot.begin_generation()

    # Data frames replace wholesale regardless of the is_snapshot flag.
    assert snapshot.apply(
        {"orderbook_id": "ob1", "is_snapshot": False, "seq": 5, "bids": [{"price": "0.46", "size": "9"}]}
    ).kind == "applied"
    assert snapshot.sequence == 5
    assert snapshot.best_bid() == "0.46"


def test_mid_price_and_spread():
    snapshot = OrderbookState(orderbook_id="ob1")
    snapshot.apply(
        make_book(is_snapshot=True, seq=1, bids=[("0.50", "10")], asks=[("0.52", "5")])
    )
    assert snapshot.mid_price() == "0.51"
    assert snapshot.spread() == "0.02"


def test_clear_resets_state():
    snapshot = OrderbookState(orderbook_id="ob1")
    snapshot.apply(
        make_book(is_snapshot=True, seq=1, bids=[("0.45", "10")], asks=[("0.55", "12")])
    )
    snapshot.clear()
    assert snapshot.is_empty()
    assert snapshot.sequence == 0


def test_forward_gap_and_truncation_metadata_apply():
    snapshot = OrderbookState(orderbook_id="ob1")
    snapshot.apply(make_book(is_snapshot=True, seq=10))
    update = make_book(is_snapshot=True, seq=15, bids=[("0.45", "1")])
    update.bids_truncated = True
    assert snapshot.apply(update).kind == "applied"
    assert snapshot.sequence == 15
    assert snapshot.bids_truncated is True
    assert snapshot.asks_truncated is False


def test_aggregation_generations_are_separate():
    full = OrderbookState(orderbook_id="ob1")
    grouped = OrderbookState(
        orderbook_id="ob1", aggregation=BookAggregation(n_sig_figs=5, mantissa=2)
    )
    full_frame = make_book(is_snapshot=True, seq=100)
    grouped_frame = make_book(is_snapshot=True, seq=3)
    grouped_frame.n_sig_figs = 5
    grouped_frame.mantissa = 2
    assert full.apply(full_frame).kind == "applied"
    assert grouped.apply(grouped_frame).kind == "applied"
    assert full.apply(grouped_frame).kind == "subscription_mismatch"
    assert full.sequence == 100
    assert grouped.sequence == 3
