"""Committed-trading order REST responses and order client queries."""

import asyncio
from datetime import UTC, datetime
from decimal import Decimal
from types import SimpleNamespace

import pytest

from lightcone_sdk.domain.order import (
    CancelAllSuccess,
    CancelStatus,
    CancelSuccess,
    InitialCohortState,
    OrderStatus,
    Role,
    SubmitOrderResponse,
    SubmitOrderStatus,
    UserOrder,
    UserOrderFillsResponse,
    UserOrdersResponse,
    limit_order_from_user_order,
)
from lightcone_sdk.domain.order.client import Orders
from lightcone_sdk.domain.position.wire import ObservationState
from lightcone_sdk.error import DeserializationError, ValidationError
from lightcone_sdk.shared.types import FundingSource, Side, TimeInForce

HASH = "4f1a4b1ab1c0c0ffee0000000000000000000000000000000000000000000001"
U64_MAX = (1 << 64) - 1
I128_MIN = -(1 << 127)
WALLET = "Wallet1111111111111111111111111111111111111"


def rest_user_order(**overrides):
    order = {
        "order_hash": HASH,
        "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
        "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
        "side": "bid",
        "amount_in": "5.500000",
        "amount_out": "10.00000000",
        "price": "0.5500",
        "created_at": 1790685521784,
        "expiration": 0,
        "base_mint": "BaseMint111111111111111111111111111111111111",
        "quote_mint": "QuoteMint11111111111111111111111111111111111",
        "outcome_index": -1,
        "state": {
            "original_base": "100",
            "confirmed_base": "40",
            "pending_base": "20",
            "open_base": "40",
            "cancelled_base": "0",
            "closed_reason": None,
            "committed_revision": "7",
            "accepted_seq": "0",
        },
        "tif": "FOK",
        "source": "global",
        "order_type": "limit",
    }
    order.update(overrides)
    return order


def funding_account(**overrides):
    account = {
        "account": "Acct1111111111111111111111111111111111111111",
        "mint": "Cond1111111111111111111111111111111111111111",
        "source": "conditional",
        "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
        "deposit_mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
        "raw_observed": "4.00000000",
        "order_reserved": "340282366920938463463374607431768.211455",
        "execution_reserved": "0.00000000",
        "signed_remaining_atoms": "0",
        "accepted_boundary": "boundary",
        "observed_slot": "123",
        "observed_blockhash": "hash",
        "observation_state": "present",
    }
    account.update(overrides)
    return account


# ── Submission ───────────────────────────────────────────────────────────────


def test_submit_is_accepted_pending_without_a_committed_view():
    response = SubmitOrderResponse.from_dict(
        {
            "order_hash": HASH,
            "status": "accepted_pending",
            "state": None,
            "initial_cohort": None,
            "fills": [],
            "fills_complete": False,
            "fills_next_cursor": None,
        }
    )
    assert response.status is SubmitOrderStatus.ACCEPTED_PENDING
    assert response.state is None
    assert response.initial_cohort is None
    assert response.filled_base() is None
    assert response.open_base() is None


@pytest.mark.parametrize(
    ("wire", "status"),
    [
        ("filled", SubmitOrderStatus.FILLED),
        ("accepted", SubmitOrderStatus.ACCEPTED),
        ("accepted_pending", SubmitOrderStatus.ACCEPTED_PENDING),
        ("partial_fill", SubmitOrderStatus.UNKNOWN),
        ("accepted_partial", SubmitOrderStatus.UNKNOWN),
        ("unknown", SubmitOrderStatus.UNKNOWN),
    ],
)
def test_submit_statuses_keep_the_hash_of_an_unknown_one(wire, status):
    response = SubmitOrderResponse.from_dict(
        {"order_hash": HASH, "status": wire, "fills": [], "fills_complete": True}
    )
    assert response.status is status
    assert response.order_hash == HASH


def test_submit_absent_fills_default_and_null_fills_fail():
    response = SubmitOrderResponse.from_dict({"order_hash": HASH, "status": "accepted"})
    assert response.fills == []
    assert response.fills_complete is False
    with pytest.raises(DeserializationError, match=r"submit\.fills"):
        SubmitOrderResponse.from_dict(
            {"order_hash": HASH, "status": "accepted", "fills": None}
        )


def test_submit_decodes_committed_state_initial_cohort_and_fills():
    response = SubmitOrderResponse.from_dict(
        {
            "order_hash": HASH,
            "status": "accepted_pending",
            "state": {
                "original_base": "10",
                "confirmed_base": "4",
                "pending_base": "2",
                "open_base": "4",
                "executable_base": "4",
                "cancelled_base": "0",
                "ready": True,
                "closed_reason": "",
                "committed_revision": "812",
                "accepted_seq": "44",
            },
            "initial_cohort": {
                "state": "pending",
                "selected_base": "6",
                "known_confirmed_base": "4",
                "known_failed_unfilled_base": "0",
                "unresolved_base": "2",
            },
            "fills": [
                {
                    "fill_id": "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:1",
                    "execution_id": "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11",
                    "counterparty": "Maker111111111111111111111111111111111111111",
                    "counterparty_order_hash": HASH,
                    "base_amount": "4",
                    "quote_amount": "2.2",
                    "is_maker": False,
                    "fee_estimate_atoms": "-12",
                    "fee_bps": -5,
                    "fee_mint": "QuoteMint11111111111111111111111111111111111",
                }
            ],
            "fills_complete": False,
            "fills_next_cursor": "cursor",
        }
    )
    assert response.state is not None
    assert response.state.committed_revision == 812
    assert response.state.closed_reason is None
    assert response.state.ready is True
    assert response.filled_base() == Decimal("4")
    assert response.open_base() == Decimal("4")
    assert response.initial_cohort is not None
    assert response.initial_cohort.state is InitialCohortState.PENDING
    fill = response.fills[0]
    assert fill.fee_estimate_atoms == -12
    assert fill.fee_bps == -5
    assert fill.price() == Decimal("0.55")
    assert response.fills_next_cursor == "cursor"


def test_submit_rejects_float_and_bool_wire_values():
    state = {
        "original_base": "10",
        "confirmed_base": "4",
        "pending_base": "2",
        "open_base": "4",
        "executable_base": "4",
        "cancelled_base": "0",
        "ready": True,
        "committed_revision": "812",
        "accepted_seq": "44",
    }
    for field, value in (
        ("committed_revision", 812.0),
        ("committed_revision", True),
        ("open_base", 4),
        ("ready", 1),
    ):
        with pytest.raises(DeserializationError, match=rf"submit\.state\.{field}"):
            SubmitOrderResponse.from_dict(
                {
                    "order_hash": HASH,
                    "status": "accepted",
                    "state": {**state, field: value},
                }
            )


# ── Cancellation ─────────────────────────────────────────────────────────────


def test_cancel_decodes_raw_base_atoms_and_closed_reasons():
    response = CancelSuccess.from_dict(
        {
            "status": "cancelled",
            "order_hash": HASH,
            "quantities": {
                "newly_cancelled_base": "18446744073709551615",
                "confirmed_base": "0",
                "pending_base": "40",
                "remaining_open_base": "0",
            },
            "quantity_unit": "base_atoms",
            "revision": "812",
            "closed_reason": "",
        }
    )
    assert response.status is CancelStatus.CANCELLED
    assert response.quantities is not None
    assert response.quantities.newly_cancelled_base == U64_MAX
    assert response.quantities.pending_base == 40
    assert response.quantity_unit == "base_atoms"
    assert response.revision == 812
    assert response.closed_reason is None

    already = {
        "status": "already_closed",
        "order_hash": HASH,
        "quantities": None,
        "quantity_unit": "base_atoms",
        "revision": "813",
        "closed_reason": "expired",
    }
    decoded = CancelSuccess.from_dict(already)
    assert decoded.status is CancelStatus.ALREADY_CLOSED
    assert decoded.quantities is None
    assert decoded.closed_reason == "expired"
    assert (
        CancelSuccess.from_dict({**already, "status": "superseded"}).status
        is CancelStatus.UNKNOWN
    )
    with pytest.raises(DeserializationError, match=r"cancel\.quantities\.pending_base"):
        CancelSuccess.from_dict(
            {
                **already,
                "quantities": {
                    "newly_cancelled_base": "0",
                    "confirmed_base": "0",
                    "pending_base": "18446744073709551616",
                    "remaining_open_base": "0",
                },
            }
        )


def test_cancel_all_decodes_the_closure():
    response = CancelAllSuccess.from_dict(
        {
            "status": "success",
            "user_pubkey": WALLET,
            "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
            "closure": {
                "operation_id": "8c1d1c5e-2b8f-4bb1-9f53-3c3f5f0a1b2c",
                "committed_revision": "900",
                "scope": f"WalletBook:{WALLET}:j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                "accepted_seq_cutoff": "44",
                "cleanup_pending": True,
            },
            "message": "Accepted-order cutoff committed",
        }
    )
    assert response.closure is not None
    assert response.closure.committed_revision == 900
    assert response.closure.accepted_seq_cutoff == 44
    assert response.closure.cleanup_pending is True
    assert response.closure.scope.startswith("WalletBook:")

    wallet_wide = CancelAllSuccess.from_dict(
        {
            "status": "success",
            "user_pubkey": WALLET,
            "closure": {
                "operation_id": "8c1d1c5e-2b8f-4bb1-9f53-3c3f5f0a1b2d",
                "committed_revision": "901",
                "scope": f"Wallet:{WALLET}",
                "accepted_seq_cutoff": None,
                "cleanup_pending": False,
            },
            "message": "Accepted-order cutoff committed",
        }
    )
    assert wallet_wide.orderbook_id == ""
    assert wallet_wide.closure is not None
    assert wallet_wide.closure.accepted_seq_cutoff is None


# ── User orders ──────────────────────────────────────────────────────────────


def test_user_orders_decode_orders_and_the_funding_page():
    state = {
        **rest_user_order()["state"],
        "open_base": "4.00000000",
        "accepted_seq": "45",
        "committed_revision": "812",
    }
    response = UserOrdersResponse.from_dict(
        {
            "user_pubkey": WALLET,
            "orders": [rest_user_order(side="ask", state=state, source="conditional")],
            "funding_accounts": [funding_account()],
            "next_cursor": None,
            "has_more": False,
            "next_funding_cursor": None,
            "funding_has_more": False,
            "committed_revision": "812",
            "projection_generation": "1",
        }
    )
    assert response.committed_revision == 812
    order = limit_order_from_user_order(response.orders[0])
    assert order is not None
    assert order.side is Side.ASK
    assert order.remaining_size == Decimal("4.00000000")
    assert str(order.remaining_size) == "4.00000000"
    assert order.accepted_seq == 45
    assert order.funding_source is FundingSource.CONDITIONAL
    assert order.status is OrderStatus.PENDING
    assert order.created_at == datetime(2026, 9, 29, 12, 38, 41, 784000, tzinfo=UTC)
    account = response.funding_accounts[0]
    assert account.order_reserved == "340282366920938463463374607431768.211455"
    assert account.order_reserved.to_decimal() is None
    assert account.observed_slot == 123
    assert account.observation_state is ObservationState.PRESENT


def test_user_orders_can_return_an_empty_page_that_still_has_more():
    response = UserOrdersResponse.from_dict(
        {
            "user_pubkey": WALLET,
            "orders": [],
            "next_cursor": "45:4f1a",
            "has_more": True,
            "committed_revision": "812",
            "projection_generation": "1",
        }
    )
    assert response.orders == []
    assert response.funding_accounts == []
    assert response.has_more is True
    assert response.funding_has_more is False
    assert response.next_cursor == "45:4f1a"


def test_rest_order_recorded_state():
    order = UserOrder.from_dict(rest_user_order())
    assert order.state is not None
    assert order.state.pending_base == Decimal("20")
    assert order.state.committed_revision == 7
    assert order.state.closed_reason is None
    assert order.tif is TimeInForce.FOK
    assert order.source is FundingSource.GLOBAL
    assert order.outcome_index == -1
    assert order.expiration == 0


def test_rest_order_defaults_only_absent_fields():
    order = rest_user_order()
    for key in ("expiration", "outcome_index", "order_type", "tif"):
        del order[key]
    decoded = UserOrder.from_dict(order)
    assert decoded.expiration == 0
    assert decoded.outcome_index == -1
    assert decoded.order_type == ""
    assert decoded.tif is None
    for key in ("expiration", "outcome_index", "order_type"):
        with pytest.raises(DeserializationError, match=rf"order\.{key}"):
            UserOrder.from_dict(rest_user_order(**{key: None}))


def test_rest_order_does_not_fail_on_unknown_wire_values():
    order = UserOrder.from_dict(rest_user_order(tif="unsupported", source="vault"))
    assert order.tif is None
    assert order.source is FundingSource.UNKNOWN
    converted = limit_order_from_user_order(order)
    assert converted is not None
    assert converted.time_in_force is None
    # Wire names are case-sensitive, as in Rust.
    assert UserOrder.from_dict(rest_user_order(tif="fok")).tif is None


def test_rest_order_without_recorded_state_does_not_convert():
    assert (
        limit_order_from_user_order(UserOrder.from_dict(rest_user_order(state=None)))
        is None
    )


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("price", 0.55),
        ("price", "5e-1"),
        ("price", " 0.55"),
        ("side", 0),
        ("side", "BID"),
        ("created_at", "1790685521784"),
        ("created_at", 1790685521784.0),
        ("expiration", "0"),
        ("outcome_index", 40000),
    ],
)
def test_malformed_order_names_the_field(field, value):
    with pytest.raises(DeserializationError, match=rf"order\.{field}"):
        UserOrder.from_dict(rest_user_order(**{field: value}))


# ── Order fills ──────────────────────────────────────────────────────────────


def test_order_fills_decode_the_backend_shape():
    response = UserOrderFillsResponse.from_dict(
        {
            "orders": [
                {
                    "order_hash": HASH,
                    "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                    "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                    "side": "bid",
                    "base_mint": "BaseMint111111111111111111111111111111111111",
                    "quote_mint": "QuoteMint11111111111111111111111111111111111",
                    "original_base": "10.00000000",
                    "confirmed_base": "4.00000000",
                    "pending_base": "0.00000000",
                    "open_base": "0.00000000",
                    "cancelled_base": "6.00000000",
                    "confirmed_quote": "2.200000",
                    "status": "closed",
                    "closed_reason": "expired",
                    "created_at": 1790685521784,
                    "fills": [
                        {
                            "fill_id": "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:1",
                            "counterparty": "Maker111111111111111111111111111111111111111",
                            "counterparty_order_hash": HASH,
                            "role": "taker",
                            "base_amount": "4.00000000",
                            "quote_amount": "2.200000",
                            "fee_estimate_atoms": "-170141183460469231731687303715884105728",
                            "fee_mint": "QuoteMint11111111111111111111111111111111111",
                            "maker_fee_bps": -5,
                            "taker_fee_bps": 30,
                            "tx_signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW",
                            "filled_at": 1790685522000,
                        }
                    ],
                    "fills_has_more": False,
                    "fills_next_cursor": None,
                }
            ],
            "next_cursor": "eyJhdCI6IjIwMjYtMDktMjlUMTI6MDA6MDBaIiwia2V5IjoiYWIifQ",
            "has_more": True,
            "committed_revision": "812",
        }
    )
    assert response.committed_revision == 812
    order = response.orders[0]
    assert order.status is OrderStatus.CLOSED
    assert order.closed_reason == "expired"
    assert order.side is Side.BID
    fill = order.fills[0]
    assert fill.role is Role.TAKER
    assert fill.fee_estimate_atoms == I128_MIN
    assert fill.maker_fee_bps == -5
    assert fill.price() == Decimal("0.55")
    assert str(fill.price()) == "0.55"


def test_order_fills_decode_unknown_roles_and_statuses_as_unknown():
    fill = {
        "fill_id": "f",
        "counterparty": "Maker111111111111111111111111111111111111111",
        "counterparty_order_hash": HASH,
        "role": "arbiter",
        "base_amount": "1",
        "quote_amount": "1",
        "fee_estimate_atoms": "0",
        "fee_mint": "QuoteMint11111111111111111111111111111111111",
        "maker_fee_bps": 0,
        "taker_fee_bps": 0,
        "tx_signature": "sig",
        "filled_at": 1,
    }
    response = UserOrderFillsResponse.from_dict(
        {
            "orders": [
                {
                    "order_hash": HASH,
                    "market_pubkey": "m",
                    "orderbook_id": "ob",
                    "side": "ask",
                    "base_mint": "b",
                    "quote_mint": "q",
                    "original_base": "1",
                    "confirmed_base": "1",
                    "pending_base": "0",
                    "open_base": "0",
                    "cancelled_base": "0",
                    "confirmed_quote": "1",
                    "status": "settling",
                    "created_at": 1,
                    "fills": [fill],
                }
            ],
            "has_more": False,
            "committed_revision": 1,
        }
    )
    order = response.orders[0]
    assert order.status is OrderStatus.UNKNOWN
    assert order.fills[0].role is Role.UNKNOWN
    assert order.fills_has_more is False
    assert response.next_cursor is None


# ── Orders client ────────────────────────────────────────────────────────────


class RecordingHttp:
    def __init__(self, response):
        self.response = response
        self.calls = []

    async def get(self, path, retry_policy=None, *, params=None, **_kwargs):
        self.calls.append(("get", path, params))
        return self.response

    async def get_with_cookies(
        self, path, retry_policy=None, *, cookie_header, params=None
    ):
        self.calls.append(("get_with_cookies", path, params, cookie_header))
        return self.response


def orders_client(response):
    http = RecordingHttp(response)
    return Orders(SimpleNamespace(_http=http)), http


def test_order_fills_send_only_the_given_parameters_in_rust_order():
    orders, http = orders_client(
        {"orders": [], "has_more": False, "committed_revision": "1"}
    )

    async def run():
        await orders.get_user_order_fills(limit=20, cursor="c", fills=0)
        await orders.get_order_fill_page(HASH, "f")
        await orders.get_user_order_fills_by_wallet(WALLET)
        await orders.get_user_order_fills_with_cookies(
            "m", None, None, "privy-token=t", fills=3
        )
        await orders.get_order_fill_page_with_cookies(
            HASH, None, "privy-token=t", fills=16
        )
        await orders.get_order_fill_page_by_wallet(WALLET, HASH, fills=1)

    asyncio.run(run())
    assert http.calls == [
        ("get", "/api/users/order-fills", {"limit": "20", "cursor": "c", "fills": "0"}),
        ("get", "/api/users/order-fills", {"order_hash": HASH, "fill_cursor": "f"}),
        ("get", f"/api/users/{WALLET}/order-fills", None),
        (
            "get_with_cookies",
            "/api/users/order-fills",
            {"market_pubkey": "m", "fills": "3"},
            "privy-token=t",
        ),
        (
            "get_with_cookies",
            "/api/users/order-fills",
            {"order_hash": HASH, "fills": "16"},
            "privy-token=t",
        ),
        ("get", f"/api/users/{WALLET}/order-fills", {"order_hash": HASH, "fills": "1"}),
    ]
    assert list(http.calls[0][2]) == ["limit", "cursor", "fills"]


@pytest.mark.parametrize("value", [True, False, -1, 1 << 32, 1.0, "5"])
def test_paging_parameters_reject_bools_negatives_and_non_integers(value):
    orders, http = orders_client({})

    async def run():
        with pytest.raises(ValidationError, match="limit"):
            await orders.get_user_orders(limit=value)
        with pytest.raises(ValidationError, match="fills"):
            await orders.get_user_order_fills(fills=value)

    asyncio.run(run())
    assert http.calls == []


def test_user_orders_route_decodes_the_page():
    orders, http = orders_client(
        {
            "user_pubkey": WALLET,
            "orders": [rest_user_order()],
            "has_more": False,
            "committed_revision": "9",
            "projection_generation": "1",
        }
    )
    response = asyncio.run(orders.get_user_orders(50, "45:4f1a"))
    assert http.calls == [
        ("get", "/api/users/orders", {"limit": "50", "cursor": "45:4f1a"})
    ]
    assert response.funding_accounts == []
    assert response.orders[0].state is not None
    assert response.orders[0].state.accepted_seq == 0

    asyncio.run(orders.get_user_orders())
    assert http.calls[-1] == ("get", "/api/users/orders", None)


def test_submit_and_cancel_routes_decode_committed_responses():
    class Http:
        def __init__(self):
            self.posts = []

        async def post(self, path, body, *_args, **_kwargs):
            self.posts.append(path)
            if path == "/api/orders/cancel":
                return {
                    "status": "already_filled",
                    "order_hash": HASH,
                    "quantities": None,
                    "quantity_unit": "base_atoms",
                    "revision": "3",
                    "closed_reason": "",
                }
            return {
                "status": "success",
                "user_pubkey": WALLET,
                "orderbook_id": "",
                "closure": None,
                "message": "nothing to close",
            }

    from lightcone_sdk.domain.order import CancelAllBody, CancelBody

    http = Http()
    orders = Orders(SimpleNamespace(_http=http))
    cancelled = asyncio.run(
        orders.cancel(CancelBody(order_hash=HASH, maker=WALLET, signature="00"))
    )
    assert cancelled.status is CancelStatus.ALREADY_FILLED
    assert cancelled.revision == 3
    everything = asyncio.run(
        orders.cancel_all(
            CancelAllBody(
                user_pubkey=WALLET,
                orderbook_id="",
                signature="00",
                timestamp=1,
                salt="s",
            )
        )
    )
    assert everything.closure is None
    assert everything.message == "nothing to close"
    assert http.posts == ["/api/orders/cancel", "/api/orders/cancel-all"]
