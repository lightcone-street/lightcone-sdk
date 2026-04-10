"""Tests for trade history ordering."""

from lightcone_sdk.domain.trade import Trade
from lightcone_sdk.domain.trade.state import TradeHistory


def make_trade(trade_id: str, sequence: int) -> Trade:
    return Trade(
        orderbook_id="ob1",
        trade_id=trade_id,
        timestamp="2025-01-01T00:00:00Z",
        price="1.0",
        size="1.0",
        side=0,
        sequence=sequence,
    )


def test_trade_history_reorders_by_sequence():
    history = TradeHistory(orderbook_id="ob1", max_size=10)

    history.push(make_trade("t3", 3))
    history.push(make_trade("t1", 1))
    history.push(make_trade("t2", 2))

    assert [trade.trade_id for trade in history.trades()] == ["t1", "t2", "t3"]


def test_trade_history_drops_older_sequence_when_full():
    history = TradeHistory(orderbook_id="ob1", max_size=3)

    history.push(make_trade("t3", 3))
    history.push(make_trade("t4", 4))
    history.push(make_trade("t5", 5))
    history.push(make_trade("t2", 2))

    assert [trade.trade_id for trade in history.trades()] == ["t3", "t4", "t5"]
