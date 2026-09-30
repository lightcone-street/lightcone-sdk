"""User market balance wire payload tests."""

import json

import pytest

from lightcone_sdk.domain.order import ConditionalBalance
from lightcone_sdk.domain.order.client import _user_orders_response_from_wire
from lightcone_sdk.domain.order.convert import convert_snapshot_orders, order_from_ws
from lightcone_sdk.domain.order.state import UserOpenLimitOrders
from lightcone_sdk.domain.order.wire import (
    OrderUpdate,
    UserBalanceUpdate,
    UserSnapshot,
    UserUpdate,
)
from lightcone_sdk.error import DeserializationError
from lightcone_sdk.ws import parse_message_in


def market_balance() -> dict:
    return {
        "market_pubkey": "market-1",
        "deposit_assets": [
            {
                "deposit_asset": "usdc-mint",
                "outcomes": [
                    {
                        "outcome_index": 0,
                        "conditional_token": "trump-usdc-mint",
                        "balance": "125.000000",
                        "balance_idle": "40.000000",
                        "balance_on_book": "85.000000",
                    },
                    {
                        "outcome_index": 1,
                        "conditional_token": "kamala-usdc-mint",
                        "balance": "100.000000",
                        "balance_idle": "100.000000",
                        "balance_on_book": "0.000000",
                    },
                    {
                        "outcome_index": 2,
                        "conditional_token": "biden-usdc-mint",
                        "balance": "100.000000",
                        "balance_idle": "100.000000",
                        "balance_on_book": "0.000000",
                    },
                ],
            }
        ],
    }


def user_order(orderbook_id: str, base_mint: str) -> dict:
    return {
        "order_type": "limit",
        "order_hash": f"order-{orderbook_id}",
        "market_pubkey": "market-1",
        "orderbook_id": orderbook_id,
        "side": "bid",
        "amount_in": "10.000000",
        "amount_out": "10.000000",
        "remaining": "10.000000",
        "filled": "0.000000",
        "price": "1.000000",
        "created_at": 0,
        "expiration": 0,
        "base_mint": base_mint,
        "quote_mint": "trump-usdc-mint",
        "outcome_index": 0,
        "status": "OPEN",
    }


def test_snapshot_market_balances_group_multiple_outcomes_by_deposit_asset():
    update = UserUpdate.from_dict(
        {
            "event_type": "snapshot",
            "orders": [
                user_order("trump-btc-usdc", "trump-btc-mint"),
                user_order("trump-eth-usdc", "trump-eth-mint"),
            ],
            "market_balances": [market_balance()],
            "global_deposits": [],
            "notifications": [],
            "nonce": 7,
        }
    )

    assert update.event_type == "snapshot"
    assert isinstance(update.data, UserSnapshot)
    assert [order.orderbook_id for order in update.data.orders] == [
        "trump-btc-usdc",
        "trump-eth-usdc",
    ]
    deposit_asset = update.data.market_balances[0].deposit_assets[0]
    assert deposit_asset.deposit_asset == "usdc-mint"
    assert len(deposit_asset.outcomes) == 3
    assert deposit_asset.outcomes[0].conditional_token == "trump-usdc-mint"
    assert deposit_asset.outcomes[0].balance_on_book == "85.000000"
    assert not hasattr(update.data, "balances")


def test_websocket_market_balance_update_parses_new_event():
    message = parse_message_in(
        json.dumps(
            {
                "type": "user",
                "version": 1,
                "data": {
                    "event_type": "market_balance_update",
                    "market_pubkey": "market-1",
                    "market_balance": market_balance(),
                    "timestamp": "2026-06-19T12:00:00Z",
                },
            }
        )
    )

    assert message.type == "user"
    assert isinstance(message.data, UserUpdate)
    assert message.data.event_type == "market_balance_update"
    assert isinstance(message.data.data, UserBalanceUpdate)
    assert message.data.data.market_pubkey == "market-1"
    assert (
        message.data.data.market_balance.deposit_assets[0].outcomes[0].conditional_token
        == "trump-usdc-mint"
    )


def test_user_orders_rest_response_uses_market_balances():
    response = _user_orders_response_from_wire(
        {
            "user_pubkey": "user-1",
            "orders": [],
            "market_balances": [market_balance()],
            "has_more": False,
        },
        "",
    )

    assert response.market_balances[0].market_pubkey == "market-1"
    assert (
        response.market_balances[0].deposit_assets[0].outcomes[0].balance_on_book
        == "85.000000"
    )
    assert not hasattr(response, "balances")


def test_embedded_balances_use_conditional_token():
    balance = ConditionalBalance.from_dict(
        {
            "outcome_index": 0,
            "conditional_token": "trump-usdc-mint",
            "idle": "40.000000",
            "on_book": "85.000000",
        }
    )

    assert balance.conditional_token == "trump-usdc-mint"
    assert not hasattr(balance, "mint")


def test_old_balance_payload_names_are_rejected():
    with pytest.raises(DeserializationError, match="balance_update"):
        UserUpdate.from_dict(
            {
                "event_type": "balance_update",
                "market_pubkey": "market-1",
                "orderbook_id": "old-orderbook",
                "balance": {"outcomes": []},
                "timestamp": "2026-06-19T12:00:00Z",
            }
        )

    with pytest.raises(DeserializationError, match="market_balances"):
        _user_orders_response_from_wire(
            {
                "user_pubkey": "user-1",
                "orders": [],
                "balances": [],
                "has_more": False,
            },
            "",
        )

    with pytest.raises(DeserializationError, match="conditional_token"):
        ConditionalBalance.from_dict(
            {
                "outcome_index": 0,
                "mint": "trump-usdc-mint",
                "idle": "40.000000",
                "on_book": "85.000000",
            }
        )


def rest_payload(orders):
    return {
        "user_pubkey": "user-1",
        "orders": orders,
        "market_balances": [market_balance()],
        "next_cursor": "next-page",
        "has_more": True,
    }


def user_frame(data):
    return json.dumps({"type": "user", "version": 1, "data": data})


def test_limit_rest_orders_keep_account_data_and_pagination():
    supported = user_order("book-1", "base-1")
    response = _user_orders_response_from_wire(rest_payload([supported]), "")
    assert [order.order_hash for order in response.orders] == ["order-book-1"]
    assert (
        response.market_balances[0].deposit_assets[0].outcomes[0].balance
        == "125.000000"
    )
    assert response.next_cursor == "next-page"
    assert response.has_more is True
    assert not convert_snapshot_orders(response.orders).is_empty()

    empty = _user_orders_response_from_wire(rest_payload([]), "")
    assert empty.orders == []
    assert empty.market_balances == response.market_balances
    assert empty.next_cursor == "next-page"
    assert empty.has_more is True


def test_limit_snapshot_keeps_all_account_data():
    message = parse_message_in(
        user_frame(
            {
                "event_type": "snapshot",
                "orders": [
                    user_order("book-1", "base-1"),
                ],
                "market_balances": [market_balance()],
                "global_deposits": [{"mint": "quote", "balance": "9.250000"}],
                "notifications": [
                    {
                        "id": "notice",
                        "notification_type": "global",
                        "title": "Notice",
                        "message": "Fixture",
                        "created_at": "2026-01-01T00:00:00Z",
                    }
                ],
                "nonce": 17,
            }
        )
    )
    assert isinstance(message.data, UserUpdate)
    snapshot = message.data.data
    assert isinstance(snapshot, UserSnapshot)
    assert [order.order_hash for order in snapshot.orders] == ["order-book-1"]
    assert (
        snapshot.market_balances[0].deposit_assets[0].outcomes[0].balance
        == "125.000000"
    )
    assert snapshot.global_deposits[0].balance == "9.250000"
    assert snapshot.notifications[0].id == "notice"
    assert snapshot.nonce == 17


@pytest.mark.parametrize("field", ["amount_in", "amount_out", "order_hash"])
def test_malformed_limit_orders_fail_rest_and_snapshot_decoding(field):
    invalid = user_order("book-2", "base-2")
    del invalid[field]
    payload = rest_payload([user_order("book-1", "base-1"), invalid])
    with pytest.raises(DeserializationError):
        _user_orders_response_from_wire(payload, "")
    with pytest.raises(DeserializationError):
        parse_message_in(user_frame({**payload, "event_type": "snapshot"}))


def test_live_limit_order_updates_state():
    state = UserOpenLimitOrders()
    event = {
        "event_type": "order",
        "order_type": "limit",
        "market_pubkey": "market-1",
        "orderbook_id": "book-1",
        "order": user_order("book-1", "base-1"),
    }
    message = parse_message_in(user_frame(event))
    assert isinstance(message.data, UserUpdate)
    order_event = message.data.data
    assert isinstance(order_event, OrderUpdate)
    assert order_event.order is not None
    state.upsert(
        order_from_ws(
            order_event.order,
            order_event.market_pubkey,
            order_event.orderbook_id,
        )
    )
    assert not state.is_empty()


@pytest.mark.parametrize("orders", [{}, "", None])
def test_malformed_order_collections_fail_rest_and_snapshot_decoding(orders):
    payload = rest_payload(orders)
    with pytest.raises(DeserializationError, match="orders must be an array"):
        _user_orders_response_from_wire(payload, "")
    with pytest.raises(DeserializationError, match="orders must be an array"):
        parse_message_in(user_frame({**payload, "event_type": "snapshot"}))


@pytest.mark.parametrize("order_type", ["market", None])
def test_invalid_wire_kind_is_not_relabelled_as_limit(order_type):
    invalid = {**user_order("book-1", "base-1"), "order_type": order_type}
    payload = rest_payload([invalid])
    with pytest.raises(DeserializationError, match="order_type 'limit'"):
        _user_orders_response_from_wire(payload, "")
    with pytest.raises(DeserializationError, match="order_type 'limit'"):
        parse_message_in(user_frame({**payload, "event_type": "snapshot"}))
    with pytest.raises(DeserializationError, match="order_type 'limit'"):
        parse_message_in(
            user_frame(
                {
                    "event_type": "order",
                    "order_type": order_type,
                    "market_pubkey": "market-1",
                    "orderbook_id": "book-1",
                    "order": user_order("book-1", "base-1"),
                }
            )
        )


def test_malformed_live_limit_order_fails_decoding():
    for fields in [{}, {"order": {}}]:
        malformed = {
            "event_type": "order",
            "order_type": "limit",
            "market_pubkey": "market-1",
            "orderbook_id": "book-1",
            **fields,
        }
        with pytest.raises(DeserializationError):
            parse_message_in(user_frame(malformed))
