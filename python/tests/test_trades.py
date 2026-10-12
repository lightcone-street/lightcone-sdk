"""Committed trade wire types, conversion, and TradeHistory."""

import json
from dataclasses import replace
from datetime import UTC, datetime, timedelta
from decimal import Decimal

import pytest

from lightcone_sdk.domain.trade import (
    Trade,
    TradeHistory,
    TradeResponseWire,
    TradesResponseWire,
    WsTrade,
    trade_from_wire,
    trade_from_ws,
)
from lightcone_sdk.error import DeserializationError, ValidationError
from lightcone_sdk.shared.types import Side
from lightcone_sdk.ws import parse_message_in


def fill_fact(**overrides):
    """Fill fact as ``redis_listener::handle_committed`` relays it on ``trades``."""
    fact = {
        "effect_id": "812:1",
        "committed_revision": 812,
        "projection_generation": 3,
        "actionable": True,
        "event_type": "fill",
        "fill_id": "0d6a6f3e-5b9b-4b43-9f0f-6c7c1f3f9a10:2:0",
        "execution_id": "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11",
        "evidence_id": "0d6a6f3e-5b9b-4b43-9f0f-6c7c1f3f9a10",
        "event_index": 2,
        "leg_index": 0,
        "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
        "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
        "base_mint": "BaseMint111111111111111111111111111111111111",
        "quote_mint": "QuoteMint11111111111111111111111111111111111",
        "base_amount": "4.00000000",
        "quote_amount": "2.200000",
        "maker_order_hash": "aa" * 32,
        "taker_order_hash": "bb" * 32,
        "maker_wallet": "Maker111111111111111111111111111111111111111",
        "taker_wallet": "Taker111111111111111111111111111111111111111",
        "taker_side": 1,
        "maker_fee_estimate": "-0.000110",
        "taker_fee_estimate": "0.006600",
        "fee_mint": "QuoteMint11111111111111111111111111111111111",
        "maker_fee_bps": -5,
        "taker_fee_bps": 30,
        "executed_at": "2026-09-29T12:00:00+00:00",
    }
    fact.update(overrides)
    return fact


def trade_response(**overrides):
    response = {
        "id": 456,
        "trade_id": "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3",
        "orderbook_id": "ob_123",
        "taker_pubkey": "taker123",
        "maker_pubkey": "maker456",
        "side": "bid",
        "size": "10.000000",
        "price": "5.000000",
        "taker_fee_estimate": "0.003250",
        "maker_fee_estimate": "0",
        "executed_at": 1_740_076_800_000,
    }
    response.update(overrides)
    return response


# ── Wire ─────────────────────────────────────────────────────────────────────


def test_decodes_a_fill_fact_and_derives_its_price_and_rest_id():
    trade = WsTrade.from_dict(fill_fact())
    assert trade.commit.committed_revision == 812
    assert trade.commit.projection_generation == 3
    assert trade.commit.actionable is True
    assert trade.taker_side is Side.ASK
    assert trade.base_amount == Decimal("4.00000000")
    assert trade.price() == Decimal("0.55")
    assert trade.maker_fee_estimate == Decimal("-0.000110")
    assert str(trade.maker_fee_estimate) == "-0.000110"
    assert trade.trade_id() == "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3"
    assert trade.fill_id != trade.trade_id()


def test_decodes_the_trades_channel():
    message = parse_message_in(
        json.dumps({"type": "trades", "version": 0.1, "data": fill_fact()})
    )
    assert message.type == "trades"
    trade = message.data
    assert isinstance(trade, WsTrade)
    assert trade.leg_index == 0
    assert trade.fill_id == "0d6a6f3e-5b9b-4b43-9f0f-6c7c1f3f9a10:2:0"
    assert trade.commit.committed_revision == 812
    assert trade.price() == Decimal("0.55")
    assert trade.executed_at == datetime(2026, 9, 29, 12, tzinfo=UTC)


def test_trades_channel_rejects_a_non_object_payload_and_names_bad_fields():
    with pytest.raises(DeserializationError, match="trades"):
        parse_message_in(json.dumps({"type": "trades", "data": [fill_fact()]}))
    with pytest.raises(DeserializationError, match=r"trades\.taker_side"):
        parse_message_in(
            json.dumps({"type": "trades", "data": fill_fact(taker_side=True)})
        )
    with pytest.raises(DeserializationError, match=r"trades\.committed_revision"):
        parse_message_in(
            json.dumps({"type": "trades", "data": fill_fact(committed_revision=812.0)})
        )


def test_decodes_a_user_channel_fill_as_the_trades_payload():
    message = parse_message_in(
        json.dumps({"type": "user", "version": 0.1, "data": fill_fact()})
    )
    assert message.type == "user"
    assert message.data.event_type == "fill"
    trade = message.data.data
    assert isinstance(trade, WsTrade)
    assert trade.trade_id() == "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3"
    assert trade.taker_side is Side.ASK


@pytest.mark.parametrize(
    ("executed_at", "expected"),
    [
        (
            "2026-09-29T12:00:00.123456+00:00",
            datetime(2026, 9, 29, 12, 0, 0, 123456, UTC),
        ),
        (
            "2026-09-29T12:00:00.987654321+00:00",
            datetime(2026, 9, 29, 12, 0, 0, 987654, UTC),
        ),
        ("2026-09-29T14:00:00+02:00", datetime(2026, 9, 29, 12, tzinfo=UTC)),
        ("2026-09-29t12:00:00z", datetime(2026, 9, 29, 12, tzinfo=UTC)),
        ("2026-09-29 12:00:00Z", datetime(2026, 9, 29, 12, tzinfo=UTC)),
    ],
)
def test_decodes_rfc3339_times(executed_at, expected):
    trade = WsTrade.from_dict(fill_fact(executed_at=executed_at))
    assert trade.executed_at == expected
    assert trade.executed_at.tzinfo is UTC


@pytest.mark.parametrize(
    "executed_at",
    [
        "2026-09-29T12:00:00",
        "2026-09-29",
        "20260929T120000Z",
        "2026-09-29T12:00:00+0000",
        1790685600000,
    ],
)
def test_rejects_times_without_an_offset_or_in_other_formats(executed_at):
    with pytest.raises(DeserializationError, match=r"trade\.executed_at"):
        WsTrade.from_dict(fill_fact(executed_at=executed_at))


def test_rest_trade_uses_the_fee_estimate_names_and_keeps_a_large_id_exact():
    trade = TradeResponseWire.from_dict(
        json.loads(
            '{"id":9007199254740993,"trade_id":"5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3",'
            '"orderbook_id":"j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",'
            '"taker_pubkey":"Taker111111111111111111111111111111111111111",'
            '"maker_pubkey":"Maker111111111111111111111111111111111111111","side":"ask",'
            '"size":"4.00000000","price":"0.5500","taker_fee_estimate":"0.006600",'
            '"maker_fee_estimate":"-0.000110","executed_at":1790685600000}'
        )
    )
    assert trade.id == 9_007_199_254_740_993
    assert trade.side is Side.ASK
    assert trade.taker_fee_estimate == Decimal("0.006600")
    assert trade.maker_fee_estimate == Decimal("-0.000110")


@pytest.mark.parametrize(
    ("field", "value"),
    [("id", "456"), ("id", 456.0), ("executed_at", "1740076800000"), ("side", 0)],
)
def test_rest_trade_rejects_text_ids_floats_and_numeric_sides(field, value):
    with pytest.raises(DeserializationError, match=rf"trade\.{field}"):
        TradeResponseWire.from_dict(trade_response(**{field: value}))


def test_trades_page_decodes_cursor_and_decimals():
    page = TradesResponseWire.from_dict(
        {
            "orderbook_id": "ob_123",
            "trades": [trade_response()],
            "next_cursor": 455,
            "has_more": True,
            "decimals": {"price": 6, "size": 6, "fee": 6},
        }
    )
    assert page.next_cursor == 455
    assert page.has_more is True
    assert page.decimals is not None and page.decimals.price == 6
    assert (
        TradesResponseWire.from_dict(
            {"orderbook_id": "ob_123", "trades": [], "decimals": None}
        ).decimals
        is None
    )
    with pytest.raises(DeserializationError, match=r"trades\.decimals\.price"):
        TradesResponseWire.from_dict(
            {"orderbook_id": "ob_123", "trades": [], "decimals": {"price": 256}}
        )


# ── Conversion ───────────────────────────────────────────────────────────────


def test_converts_a_rest_trade():
    trade = trade_from_wire(TradeResponseWire.from_dict(trade_response()))
    assert trade.orderbook_id == "ob_123"
    assert trade.trade_id == "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3"
    assert trade.cursor_id == 456
    assert trade.price == Decimal("5.000000")
    assert trade.size == Decimal("10.000000")
    assert trade.side is Side.BID
    assert trade.timestamp == datetime(2025, 2, 20, 18, 40, tzinfo=UTC)


def test_converts_a_ws_fill_fact_to_the_rest_trade_identity():
    trade = trade_from_ws(WsTrade.from_dict(fill_fact()))
    assert trade.orderbook_id == "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a"
    assert trade.trade_id == "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3"
    assert trade.cursor_id is None
    assert trade.price == Decimal("0.55")
    assert trade.size == Decimal("4.00000000")
    assert trade.side is Side.ASK


def test_rejects_invalid_values_instead_of_defaulting_them():
    for overrides in (
        {"price": "not-a-price"},
        {"size": ""},
        {"price": "1e3"},
        {"executed_at": (1 << 63) - 1},
    ):
        with pytest.raises(ValidationError):
            trade_from_wire(TradeResponseWire.from_dict(trade_response(**overrides)))
    with pytest.raises(ValidationError):
        trade_from_ws(WsTrade.from_dict(fill_fact(base_amount="0")))


# ── TradeHistory ─────────────────────────────────────────────────────────────


def make_trade(trade_id: str, price: int, size: int, seconds: int) -> Trade:
    return Trade(
        orderbook_id="ob1",
        trade_id=trade_id,
        cursor_id=None,
        timestamp=datetime.fromtimestamp(1_790_000_000, UTC)
        + timedelta(seconds=seconds),
        price=Decimal(price),
        size=Decimal(size),
        side=Side.BID,
    )


def ids(history: TradeHistory) -> list[str]:
    return [trade.trade_id for trade in history.trades()]


def test_history_adds_trades_newest_first():
    history = TradeHistory("ob1", 10)
    history.push(make_trade("t1", 50, 5, 1))
    history.push(make_trade("t2", 51, 3, 2))
    assert len(history) == 2
    assert history.latest().trade_id == "t2"


def test_history_evicts_the_oldest_trade_when_full():
    history = TradeHistory("ob1", 3)
    for trade_id, seconds in (("t1", 1), ("t2", 2), ("t3", 3), ("t4", 4)):
        history.push(make_trade(trade_id, 50, 1, seconds))
    assert ids(history) == ["t4", "t3", "t2"]


def test_history_orders_by_execution_time():
    history = TradeHistory("ob1", 10)
    history.push(make_trade("t3", 52, 3, 3))
    history.push(make_trade("t1", 50, 1, 1))
    history.push(make_trade("t2", 51, 2, 2))
    assert ids(history) == ["t3", "t2", "t1"]


def test_history_keeps_the_newest_arrival_first_for_equal_timestamps():
    history = TradeHistory("ob1", 10)
    history.push(make_trade("a", 50, 1, 5))
    history.push(make_trade("b", 50, 1, 5))
    assert ids(history) == ["b", "a"]


def test_history_drops_an_older_trade_when_full():
    history = TradeHistory("ob1", 3)
    for trade_id, seconds in (("t5", 5), ("t4", 4), ("t3", 3), ("t2", 2)):
        history.push(make_trade(trade_id, 50, 1, seconds))
    assert ids(history) == ["t5", "t4", "t3"]


def test_history_deduplicates_rest_and_ws_trades():
    history = TradeHistory("ob1", 10)
    history.push(make_trade("exec:0:1", 50, 1, 1))
    history.push(replace(make_trade("exec:0:1", 50, 1, 1), cursor_id=77))
    assert len(history) == 1
    assert history.latest().cursor_id == 77


def test_history_never_changes_a_pushed_trade():
    shared = make_trade("exec:0:1", 50, 1, 1)
    first = TradeHistory("ob1", 10)
    second = TradeHistory("ob1", 10)
    first.push(shared)
    second.push(shared)
    first.push(replace(shared, cursor_id=77))
    assert first.latest().cursor_id == 77
    assert second.latest().cursor_id is None
    assert shared.cursor_id is None
    held = first.trades()
    first.clear()
    assert len(held) == 1


def test_history_replaces_all_trades():
    history = TradeHistory("ob1", 10)
    history.push(make_trade("t1", 50, 1, 1))
    history.replace([make_trade("a", 49, 1, 2), make_trade("b", 50, 2, 1)])
    assert len(history) == 2
    assert history.latest().trade_id == "a"


def test_history_treats_zero_capacity_as_disabled():
    history = TradeHistory("ob1", 0)
    history.push(make_trade("t1", 50, 1, 1))
    assert history.is_empty()
    history.replace([make_trade("t1", 50, 1, 1)])
    assert history.is_empty()
