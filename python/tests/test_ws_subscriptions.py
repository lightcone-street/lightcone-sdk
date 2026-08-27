"""Tests for book subscription identity and wire mapping with aggregation."""

import json

import pytest

from lightcone_sdk.domain.orderbook.aggregation import FULL_PRECISION, BookAggregation
from lightcone_sdk.domain.orderbook.wire import WsOrderBook
from lightcone_sdk.error import DeserializationError
from lightcone_sdk.ws import (
    WsErrorData,
    parse_message_in,
    subscribe_books,
    subscribe_wallet_deposit_balances,
    unsubscribe_books,
    unsubscribe_wallet_deposit_balances,
)
from lightcone_sdk.ws.client import (
    WsClient,
    _subscribe_params_to_message,
    _unsubscribe_params_to_message,
)
from lightcone_sdk.ws.subscriptions import (
    BookUpdateParams,
    TickerParams,
    UserParams,
    WalletDepositBalancesParams,
    subscription_key,
    unsubscribe_matches,
)


class TestBookAggregation:
    def test_validate_matches_backend_contract(self):
        assert BookAggregation.validate(None, None) == FULL_PRECISION
        assert BookAggregation.validate(3) == BookAggregation(n_sig_figs=3)
        # (5, None) normalizes to (5, 1).
        assert BookAggregation.validate(5) == BookAggregation(n_sig_figs=5, mantissa=1)
        assert BookAggregation.validate(5, 5) == BookAggregation(
            n_sig_figs=5, mantissa=5
        )

        for invalid in [(1, None), (6, None), (4, 2), (None, 2), (5, 3), (5, 0)]:
            with pytest.raises(ValueError):
                BookAggregation.validate(*invalid)

    def test_from_frame_untagged_is_full_precision(self):
        assert BookAggregation.from_frame(None, None).is_full()
        assert not BookAggregation.from_frame(4, None).is_full()

    def test_key_suffix_vocabulary(self):
        assert FULL_PRECISION.key_suffix() == "full"
        assert BookAggregation(n_sig_figs=2).key_suffix() == "sig2"
        assert BookAggregation(n_sig_figs=5).key_suffix() == "sig5m1"
        assert BookAggregation(n_sig_figs=5, mantissa=2).key_suffix() == "sig5m2"


class TestBookSubscriptionIdentity:
    def test_full_precision_keeps_pre_aggregation_key(self):
        params = BookUpdateParams(orderbook_ids=["b", "a"])
        assert subscription_key(params) == "book:a,b"

    def test_aggregated_keys_are_distinct_and_normalized(self):
        grouped = BookUpdateParams(orderbook_ids=["a"], n_sig_figs=5, mantissa=2)
        assert subscription_key(grouped) == "book:a:sig5m2"

        implicit = BookUpdateParams(orderbook_ids=["a"], n_sig_figs=5)
        explicit = BookUpdateParams(orderbook_ids=["a"], n_sig_figs=5, mantissa=1)
        assert subscription_key(implicit) == subscription_key(explicit)

    def test_unsubscribe_matches_normalized_aggregation(self):
        subscribe = BookUpdateParams(orderbook_ids=["a"], n_sig_figs=5)
        normalized = BookUpdateParams(orderbook_ids=["a"], n_sig_figs=5, mantissa=1)
        full_precision = BookUpdateParams(orderbook_ids=["a"])
        other_grouping = BookUpdateParams(orderbook_ids=["a"], n_sig_figs=5, mantissa=2)

        # (5, None) and (5, 1) are the same subscription.
        assert unsubscribe_matches(subscribe, normalized)
        # A grouped subscription is never matched by a full-precision or
        # differently grouped unsubscribe.
        assert not unsubscribe_matches(subscribe, full_precision)
        assert not unsubscribe_matches(subscribe, other_grouping)


class TestWireMapping:
    def test_default_params_emit_no_aggregation_keys(self):
        """A full-precision subscribe must contain neither snake_case keys nor
        nulls — the backend's strict parsing rejects both."""
        message = _subscribe_params_to_message(BookUpdateParams(orderbook_ids=["a"]))
        assert message["params"] == {"type": "book_update", "orderbook_ids": ["a"]}
        assert "n_sig_figs" not in message["params"]
        assert "nSigFigs" not in message["params"]
        assert None not in message["params"].values()

    def test_aggregated_params_rename_to_camel_case(self):
        message = _subscribe_params_to_message(
            BookUpdateParams(orderbook_ids=["a"], n_sig_figs=5, mantissa=2)
        )
        assert message["params"]["nSigFigs"] == 5
        assert message["params"]["mantissa"] == 2
        assert "n_sig_figs" not in message["params"]

        unsubscribe = _unsubscribe_params_to_message(
            BookUpdateParams(orderbook_ids=["a"], n_sig_figs=4)
        )
        assert unsubscribe["params"]["nSigFigs"] == 4
        assert "mantissa" not in unsubscribe["params"]

    def test_builders_normalize_and_omit(self):
        full = subscribe_books(["a"])
        assert full["params"] == {"type": "book_update", "orderbook_ids": ["a"]}

        # (5, None) is sent in its normalized form (5, 1).
        grouped = subscribe_books(["a"], n_sig_figs=5)
        assert grouped["params"]["nSigFigs"] == 5
        assert grouped["params"]["mantissa"] == 1

        ungrouped = unsubscribe_books(["a"], n_sig_figs=3)
        assert ungrouped["params"]["nSigFigs"] == 3
        assert "mantissa" not in ungrouped["params"]

    def test_wallet_balance_subscription_wire_and_identity(self):
        params = WalletDepositBalancesParams(wallet_address="wallet-a")
        assert subscription_key(params) == "wallet_deposit_balances:wallet-a"
        assert _subscribe_params_to_message(params) == (
            subscribe_wallet_deposit_balances("wallet-a")
        )
        assert _unsubscribe_params_to_message(params) == (
            unsubscribe_wallet_deposit_balances("wallet-a")
        )

    @pytest.mark.asyncio
    async def test_auth_cleanup_removes_active_and_queued_wallet_channels(self):
        client = WsClient()
        await client.subscribe(UserParams(wallet_address="wallet-a"))
        await client.subscribe(WalletDepositBalancesParams(wallet_address="wallet-a"))
        await client.subscribe(TickerParams(orderbook_ids=["book-a"]))

        client.clear_authed_subscriptions()

        assert client._active_subscriptions == [  # noqa: SLF001
            TickerParams(orderbook_ids=["book-a"])
        ]
        assert client._pending_messages == [  # noqa: SLF001
            {
                "method": "subscribe",
                "params": {"type": "ticker", "orderbook_ids": ["book-a"]},
            }
        ]


class TestFrameTags:
    def test_frame_aggregation_from_tags(self):
        tagged = WsOrderBook.from_dict(
            {
                "orderbook_id": "ob1",
                "is_snapshot": True,
                "seq": 0,
                "bids": [],
                "asks": [],
                "n_sig_figs": 5,
                "mantissa": 2,
            }
        )
        assert tagged.aggregation() == BookAggregation(n_sig_figs=5, mantissa=2)

        # Untagged frames (old backends / full precision) are full precision.
        untagged = WsOrderBook.from_dict(
            {
                "orderbook_id": "ob1",
                "is_snapshot": True,
                "seq": 0,
                "bids": [],
                "asks": [],
            }
        )
        assert untagged.aggregation().is_full()

    def test_ws_error_aggregation_tags(self):
        error = WsErrorData.from_dict(
            {
                "error": "Engine unreachable, cannot subscribe",
                "code": "ENGINE_UNAVAILABLE",
                "orderbook_id": "ob1",
                "n_sig_figs": 4,
            }
        )
        assert error.orderbook_id == "ob1"
        assert error.aggregation() == BookAggregation(n_sig_figs=4)


class TestBookQuoteNotional:
    def test_decodes_exact_full_and_grouped_bid_ask_levels(self):
        full = parse_message_in(
            json.dumps(
                {
                    "type": "book_update",
                    "version": 0.1,
                    "data": {
                        "orderbook_id": "ob1",
                        "is_snapshot": True,
                        "seq": 10,
                        "bids": [
                            {
                                "side": "bid",
                                "price": "65000",
                                "size": "0.03",
                                "quote_notional": "1948.01",
                            }
                        ],
                        "asks": [
                            {
                                "side": "ask",
                                "price": "65001",
                                "size": "0.02",
                                "quote_notional": "1300.02",
                            }
                        ],
                    },
                }
            )
        )
        assert isinstance(full.data, WsOrderBook)
        assert full.data.bids[0].quote_notional == "1948.01"
        assert full.data.asks[0].quote_notional == "1300.02"

        grouped = parse_message_in(
            json.dumps(
                {
                    "type": "book_update",
                    "version": 0.1,
                    "data": {
                        "orderbook_id": "ob1",
                        "is_snapshot": False,
                        "seq": 11,
                        "n_sig_figs": 5,
                        "mantissa": 2,
                        "bids": [
                            {
                                "side": "bid",
                                "price": "100",
                                "size": "2",
                                "quote_notional": "199",
                            }
                        ],
                        "asks": [
                            {
                                "side": "ask",
                                "price": "101",
                                "size": "3",
                                "quote_notional": "304",
                            }
                        ],
                    },
                }
            )
        )
        assert isinstance(grouped.data, WsOrderBook)
        assert grouped.data.bids[0].quote_notional == "199"
        assert grouped.data.bids[0].quote_notional != "200"
        assert grouped.data.asks[0].quote_notional == "304"
        assert grouped.data.asks[0].quote_notional != "303"

    @pytest.mark.parametrize("quote_notional", [None, 2])
    def test_rejects_missing_or_non_string_quote_notional(self, quote_notional):
        level = {"side": "bid", "price": "1", "size": "2"}
        if quote_notional is not None:
            level["quote_notional"] = quote_notional
        with pytest.raises(DeserializationError):
            parse_message_in(
                json.dumps(
                    {
                        "type": "book_update",
                        "version": 0.1,
                        "data": {
                            "orderbook_id": "ob1",
                            "seq": 1,
                            "bids": [level],
                            "asks": [],
                        },
                    }
                )
            )
