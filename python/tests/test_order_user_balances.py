"""User market balance wire payload tests."""

import json

import pytest

from lightcone_sdk.domain.order import ConditionalBalance, OrderStatus
from lightcone_sdk.domain.order.client import _user_orders_response_from_wire
from lightcone_sdk.domain.order.convert import order_from_ws
from lightcone_sdk.domain.order.wire import (
    OrderUpdate,
    UserBalanceUpdate,
    UserSnapshot,
    UserUpdate,
)
from lightcone_sdk.error import DeserializationError
from lightcone_sdk.shared import OrderUpdateType
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
        message.data.data.market_balance.deposit_assets[0]
        .outcomes[0]
        .conditional_token
        == "trump-usdc-mint"
    )


def test_limit_order_expiration_event_uses_expired_variants():
    update = UserUpdate.from_dict(
        {
            "event_type": "order",
            "order_type": "limit",
            "market_pubkey": "market-1",
            "orderbook_id": "orderbook-1",
            "timestamp": "2026-08-05T12:00:00Z",
            "type": "EXPIRATION",
            "order": {
                "order_hash": "order-1",
                "price": "0.5",
                "is_maker": True,
                "remaining": "0",
                "filled": "1",
                "fill_amount": "0",
                "side": "bid",
                "created_at": 0,
                "base_mint": "base",
                "quote_mint": "quote",
                "outcome_index": 0,
                "status": "EXPIRED",
            },
        }
    )

    assert isinstance(update.data, OrderUpdate)
    assert update.data.order is not None
    assert update.data.update_type is not None
    assert (
        OrderUpdateType.from_wire(update.data.update_type) is OrderUpdateType.EXPIRATION
    )
    order = order_from_ws(
        update.data.order,
        update.data.market_pubkey,
        update.data.orderbook_id,
    )
    assert order.status is OrderStatus.EXPIRED


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
        response.market_balances[0]
        .deposit_assets[0]
        .outcomes[0]
        .balance_on_book
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
