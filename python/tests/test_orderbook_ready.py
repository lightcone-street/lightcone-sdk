"""Orderbook ticker paging, strict depth decoding, and committed-book readiness."""

import asyncio
import json
from types import SimpleNamespace

import pytest

from lightcone_sdk.domain.metrics.client import Metrics, _collect_ticker_pages
from lightcone_sdk.domain.metrics.wire import (
    OrderbookTickerEntry,
    OrderbookTickersResponse,
)
from lightcone_sdk.domain.orderbook.client import Orderbooks
from lightcone_sdk.domain.orderbook.state import OrderbookState
from lightcone_sdk.domain.orderbook.wire import PriceLevel, WsOrderBook
from lightcone_sdk.error import DeserializationError, ValidationError
from lightcone_sdk.ws import parse_message_in

RULES = {
    "base_size_decimals": 0,
    "max_price_decimals": 4,
    "max_price_significant_figures": 5,
    "integer_prices_always_allowed": True,
    "price_quantum": "0.0001",
    "price_quantum_raw": "100",
    "base_size_quantum": "1",
    "base_size_quantum_raw": "1",
}


class StubHttp:
    def __init__(self, body):
        self.body = body
        self.calls = []

    async def get(self, path, retry_policy=None, *, params=None, **_kwargs):
        self.calls.append((path, params))
        return self.body


def stub_client(body):
    return SimpleNamespace(_http=StubHttp(body))


# ── Ticker paging ────────────────────────────────────────────────────────────


def page(tickers: int, next_cursor, has_more: bool) -> OrderbookTickersResponse:
    return OrderbookTickersResponse(
        tickers=[
            OrderbookTickerEntry(
                orderbook_id="ob",
                market_pubkey="market",
                base_deposit_asset="base",
                quote_deposit_asset="quote",
            )
            for _ in range(tickers)
        ],
        next_cursor=next_cursor,
        has_more=has_more,
    )


def collect(max_pages, pages):
    queue = list(pages)
    cursors = []

    async def fetch(cursor):
        cursors.append(cursor)
        if not queue:
            raise AssertionError("unexpected page request")
        return queue.pop(0)

    return asyncio.run(_collect_ticker_pages(max_pages, fetch)), cursors


def test_merges_ticker_pages_until_the_last_one():
    merged, cursors = collect(
        10, [page(8, "a", True), page(0, "b", True), page(3, None, False)]
    )
    assert len(merged.tickers) == 11
    assert merged.has_more is False
    assert merged.next_cursor is None
    assert cursors == [None, "a", "b"]


def test_reports_truncation_at_the_page_cap():
    merged, _ = collect(2, [page(8, "a", True), page(8, "b", True)])
    assert len(merged.tickers) == 16
    assert merged.has_more is True
    assert merged.next_cursor == "b"


@pytest.mark.parametrize(
    "pages",
    [
        [page(8, "a", True), page(8, "a", True)],
        [page(8, None, True)],
    ],
)
def test_fails_when_a_page_reports_more_without_an_advancing_cursor(pages):
    with pytest.raises(ValidationError, match="advancing cursor"):
        collect(10, pages)


def test_decodes_a_ticker_page_cursor_and_omitted_prices():
    client = stub_client(
        {
            "tickers": [
                {
                    "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                    "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                    "outcome_index": 1,
                    "outcome_name": "Republicans Sweep",
                    "base_deposit_asset": "4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy",
                    "quote_deposit_asset": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
                    "best_ask": "0.5500",
                    "computed_at": "2026-09-29T12:38:41.952+00:00",
                }
            ],
            "next_cursor": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
            "has_more": True,
        }
    )
    response = asyncio.run(
        Metrics(client).orderbook_tickers_page(" mint ", "cursor", 8)
    )
    assert client._http.calls == [
        (
            "/api/metrics/orderbooks/tickers",
            {"deposit_asset": "mint", "cursor": "cursor", "limit": "8"},
        )
    ]
    assert response.has_more is True
    assert response.next_cursor == "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a"
    ticker = response.tickers[0]
    assert ticker.best_bid is None
    assert ticker.midpoint is None
    assert ticker.best_ask == "0.5500"
    assert ticker.outcome_index == 1


def test_orderbook_tickers_follows_every_page():
    class PagedHttp:
        def __init__(self):
            self.calls = []

        async def get(self, path, retry_policy=None, *, params=None, **_kwargs):
            self.calls.append(params)
            entry = {
                "orderbook_id": "ob",
                "market_pubkey": "market",
                "base_deposit_asset": "base",
                "quote_deposit_asset": "quote",
            }
            if params is None or "cursor" not in params:
                return {"tickers": [entry], "next_cursor": "ob", "has_more": True}
            return {"tickers": [entry], "has_more": False}

    http = PagedHttp()
    merged = asyncio.run(Metrics(SimpleNamespace(_http=http)).orderbook_tickers("mint"))
    assert len(merged.tickers) == 2
    assert merged.has_more is False
    assert http.calls == [
        {"deposit_asset": "mint"},
        {"deposit_asset": "mint", "cursor": "ob"},
    ]


@pytest.mark.parametrize(
    "body",
    [
        {
            "tickers": [
                {"orderbook_id": "ob", "market_pubkey": "m", "quote_deposit_asset": "q"}
            ]
        },
        {
            "tickers": [
                {
                    "orderbook_id": "ob",
                    "market_pubkey": "m",
                    "base_deposit_asset": "b",
                    "quote_deposit_asset": "q",
                    "best_bid": "abc",
                }
            ]
        },
        {
            "tickers": [
                {
                    "orderbook_id": "ob",
                    "market_pubkey": "m",
                    "base_deposit_asset": "b",
                    "quote_deposit_asset": "q",
                    "outcome_index": "1",
                }
            ]
        },
        {
            "tickers": [
                {
                    "orderbook_id": "ob",
                    "market_pubkey": "m",
                    "base_deposit_asset": "b",
                    "quote_deposit_asset": "q",
                    "computed_at": "yesterday",
                }
            ]
        },
        {"tickers": [], "has_more": None},
        {"tickers": None},
    ],
)
def test_rejects_a_malformed_ticker_page(body):
    with pytest.raises(DeserializationError, match="orderbook_tickers"):
        asyncio.run(Metrics(stub_client(body)).orderbook_tickers_page())


# ── Depth strictness ─────────────────────────────────────────────────────────


def depth_body(**overrides):
    body = {
        "orderbook_id": "ob",
        "bids": [{"price": "0.5000", "size": "10"}],
        "asks": [],
        "price_quantum": "0.0001",
        "trading_rules": RULES,
        "revision": 275,
        "captured_at_ms": 1790685521784,
        "decimals": {"price": 4, "size": 8},
        "ready": True,
    }
    body.update(overrides)
    return {key: value for key, value in body.items() if value is not ...}


def test_decodes_depth_levels_and_defaults_absent_flags():
    depth = asyncio.run(Orderbooks(stub_client(depth_body(ready=...))).get("ob"))
    assert depth.ready is False
    assert depth.bids_truncated is False
    assert depth.revision == 275
    assert depth.bids == [PriceLevel(price="0.5000", size="10")]


@pytest.mark.parametrize(
    ("overrides", "field"),
    [
        ({"ready": "yes"}, "ready"),
        ({"ready": None}, "ready"),
        ({"revision": "275"}, "revision"),
        ({"revision": "abc"}, "revision"),
        ({"revision": -1}, "revision"),
        ({"revision": 275.0}, "revision"),
        ({"captured_at_ms": ...}, "captured_at_ms"),
        ({"captured_at_ms": "1790685521784"}, "captured_at_ms"),
        ({"decimals": {"price": 300, "size": 8}}, r"decimals\.price"),
        ({"bids": [{"price": "x", "size": "1"}]}, r"bids\[0\]\.price"),
        ({"bids": [["0.5", "1"]]}, r"bids\[0\]"),
        ({"bids": ...}, "bids"),
        ({"best_bid": "0.5 "}, "best_bid"),
        ({"bids_truncated": None}, "bids_truncated"),
    ],
)
def test_rejects_a_malformed_depth_body_naming_the_field(overrides, field):
    with pytest.raises(DeserializationError, match=rf"orderbook_depth\.{field}"):
        asyncio.run(Orderbooks(stub_client(depth_body(**overrides))).get("ob"))


def test_rejects_non_canonical_trading_rules():
    with pytest.raises(DeserializationError, match="decimal strings"):
        asyncio.run(
            Orderbooks(
                stub_client(
                    depth_body(trading_rules={**RULES, "price_quantum_raw": "0.1"})
                )
            ).get("ob")
        )


# ── Readiness ────────────────────────────────────────────────────────────────


def test_exposes_ready_on_rest_depth_and_ws_frames():
    depth = asyncio.run(
        Orderbooks(
            stub_client(
                {
                    "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                    "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                    "bids": [],
                    "asks": [],
                    "tick_size": "0.1000",
                    "price_quantum": "0.1000",
                    "trading_rules": RULES,
                    "revision": 275,
                    "captured_at_ms": 1790685521784,
                    "decimals": {"price": 4, "size": 8},
                    "ready": True,
                }
            )
        ).get("j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a")
    )
    assert depth.ready is True
    assert depth.best_bid is None

    frame = parse_message_in(
        json.dumps(
            {
                "type": "book_update",
                "version": 0.1,
                "data": {
                    "orderbook_id": "ob",
                    "timestamp": "2026-09-29T12:00:00+00:00",
                    "seq": 276,
                    "ready": True,
                    "bids": [],
                    "asks": [],
                    "is_snapshot": True,
                },
            }
        )
    )
    assert frame.type == "book_update"
    assert isinstance(frame.data, WsOrderBook)
    assert frame.data.ready is True
    assert frame.data.seq == 276


def test_decodes_a_resync_frame_without_seq():
    # Exact ``orderbook_manager::resync_signal_json`` payload.
    frame = parse_message_in(
        json.dumps(
            {
                "type": "book_update",
                "version": 0.1,
                "data": {
                    "orderbook_id": "ob",
                    "resync": True,
                    "message": "Please re-subscribe to get fresh snapshot",
                    "n_sig_figs": 5,
                    "mantissa": 2,
                },
            }
        )
    )
    book = frame.data
    assert isinstance(book, WsOrderBook)
    assert book.resync is True
    assert book.seq == 0
    assert book.ready is False
    assert book.bids == []
    assert (book.aggregation().n_sig_figs, book.aggregation().mantissa) == (5, 2)

    state = OrderbookState(orderbook_id="ob", aggregation=book.aggregation())
    assert state.apply(book).kind == "refresh_required"


@pytest.mark.parametrize(
    ("data", "field"),
    [
        ({"orderbook_id": "ob", "seq": None}, "seq"),
        ({"orderbook_id": "ob", "seq": "5"}, "seq"),
        ({"orderbook_id": "ob", "seq": True}, "seq"),
        ({"orderbook_id": "ob", "seq": -1}, "seq"),
        ({"orderbook_id": "ob", "ready": 1}, "ready"),
        ({"orderbook_id": "ob", "bids": None}, "bids"),
        ({"orderbook_id": "ob", "bids": [["0.5", "1"]]}, r"bids\[0\]"),
        ({"orderbook_id": "ob", "bids_truncated": "no"}, "bids_truncated"),
    ],
)
def test_rejects_malformed_book_frames(data, field):
    with pytest.raises(DeserializationError, match=rf"book_update\.{field}"):
        parse_message_in(json.dumps({"type": "book_update", "data": data}))
    with pytest.raises(DeserializationError, match="book_update"):
        parse_message_in(json.dumps({"type": "book_update", "data": [data]}))


def test_state_ready_follows_accepted_frames():
    state = OrderbookState(orderbook_id="ob")
    assert state.ready is False
    assert (
        state.apply(WsOrderBook(orderbook_id="ob", seq=5, ready=True)).kind == "applied"
    )
    assert state.ready is True
    # A stale frame leaves readiness unchanged.
    assert (
        state.apply(WsOrderBook(orderbook_id="ob", seq=4, ready=False)).kind
        == "discarded_stale"
    )
    assert state.ready is True
    # A new generation keeps the last accepted readiness until a frame lands.
    state.begin_generation()
    assert state.ready is True
    assert (
        state.apply(WsOrderBook(orderbook_id="ob", seq=1, ready=False)).kind
        == "applied"
    )
    assert state.ready is False
    state.apply(WsOrderBook(orderbook_id="ob", seq=2, ready=True))
    state.clear()
    assert state.ready is False


def test_state_dict_path_reads_ready_and_defaults_a_missing_seq():
    state = OrderbookState(orderbook_id="ob")
    assert state.apply(
        {"orderbook_id": "ob", "ready": True, "bids": [], "asks": []}
    ).kind == ("applied")
    assert state.sequence == 0
    assert state.ready is True
    with pytest.raises(ValueError, match="ready"):
        OrderbookState(orderbook_id="ob").apply(
            {"orderbook_id": "ob", "seq": 1, "ready": "yes"}
        )


LEVEL = {"side": "bid", "price": "0.45", "size": "10", "quote_notional": "4.5"}


@pytest.mark.parametrize(
    ("data", "field"),
    [
        (
            {"orderbook_id": "ob", "bids": [{**LEVEL, "price": None}]},
            r"bids\[0\]\.price",
        ),
        ({"orderbook_id": "ob", "bids": [{**LEVEL, "size": 10}]}, r"bids\[0\]\.size"),
        ({"orderbook_id": "ob", "bids": [{**LEVEL, "side": "xx"}]}, r"bids\[0\]\.side"),
        ({"orderbook_id": "ob", "bids": [{**LEVEL, "side": 0}]}, r"bids\[0\]\.side"),
        (
            {"orderbook_id": "ob", "asks": [{**LEVEL, "quote_notional": "abc"}]},
            r"asks\[0\]\.quote_notional",
        ),
        ({"orderbook_id": "ob", "n_sig_figs": [5]}, "n_sig_figs"),
        ({"orderbook_id": "ob", "n_sig_figs": "5"}, "n_sig_figs"),
        ({"orderbook_id": "ob", "mantissa": -1}, "mantissa"),
        ({"orderbook_id": "ob", "resync": "false"}, "resync"),
        ({"orderbook_id": "ob", "is_snapshot": 1}, "is_snapshot"),
        ({"orderbook_id": 7}, "orderbook_id"),
        ({"orderbook_id": "ob", "timestamp": 5}, "timestamp"),
    ],
)
def test_book_frames_reject_malformed_levels_and_tags(data, field):
    with pytest.raises(DeserializationError, match=rf"book_update\.{field}"):
        parse_message_in(json.dumps({"type": "book_update", "data": data}))


def test_book_levels_decode_text_sides_and_keep_decimal_text():
    frame = parse_message_in(
        json.dumps(
            {
                "type": "book_update",
                "data": {
                    "orderbook_id": "ob",
                    "seq": 3,
                    "bids": [LEVEL],
                    "asks": [{**LEVEL, "side": "sell", "price": "0.55"}],
                    "n_sig_figs": 5,
                    "mantissa": 2,
                },
            }
        )
    ).data
    assert frame.bids[0].side == 0 and frame.asks[0].side == 1
    assert frame.bids[0].price == "0.45"
    assert (frame.n_sig_figs, frame.mantissa) == (5, 2)


def test_ws_order_book_keeps_its_positional_fields():
    book = WsOrderBook("ob", 5, True)
    assert book.is_snapshot is True
    assert book.ready is False
