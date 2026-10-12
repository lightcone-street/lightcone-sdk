"""Committed funding accounts (positions routes) and positions paging."""

import asyncio
from decimal import Decimal
from types import SimpleNamespace

import pytest

from lightcone_sdk.domain.position.client import Positions
from lightcone_sdk.domain.position.wire import (
    FundingAccount,
    ObservationState,
    PositionsResponse,
)
from lightcone_sdk.error import DeserializationError, ValidationError
from lightcone_sdk.shared.types import FundingSource

WALLET = "Wallet1111111111111111111111111111111111111"
MARKET = "Market11111111111111111111111111111111111111"


def conditional_account(**overrides):
    """Shape emitted by ``trading_wire::funding_account``, including the
    backend's u128 maximum aggregate fixture."""
    account = {
        "account": "Acct1111111111111111111111111111111111111111",
        "mint": "Cond1111111111111111111111111111111111111111",
        "source": "conditional",
        "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
        "deposit_mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
        "raw_observed": "0.000100",
        "order_reserved": "340282366920938463463374607431768.211455",
        "execution_reserved": "0.000040",
        "signed_remaining_atoms": "-184467440737095516100",
        "accepted_boundary": "boundary",
        "observed_slot": "7",
        "observed_blockhash": "hash",
        "observation_state": "present",
    }
    account.update(overrides)
    return account


def test_decodes_the_backend_positions_page():
    response = PositionsResponse.from_dict(
        {
            "owner": WALLET,
            "market_pubkey": None,
            "funding_accounts": [
                conditional_account(),
                {
                    "account": "Glob1111111111111111111111111111111111111111",
                    "mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
                    "source": "global",
                    "market_pubkey": "",
                    "deposit_mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
                    "raw_observed": None,
                    "order_reserved": "0",
                    "execution_reserved": "0",
                    "signed_remaining_atoms": None,
                    "accepted_boundary": "",
                    "observed_slot": None,
                    "observed_blockhash": "",
                    "observation_state": "unobserved",
                },
            ],
            "committed_revision": "275",
            "projection_generation": "1",
            "has_more": True,
            "next_cursor": "Glob1111111111111111111111111111111111111111",
        }
    )
    assert response.committed_revision == 275
    assert response.projection_generation == 1
    assert response.market_pubkey is None
    assert response.has_more is True
    conditional, global_account = response.funding_accounts
    assert conditional.source is FundingSource.CONDITIONAL
    assert conditional.raw_observed == Decimal("0.000100")
    assert str(conditional.raw_observed) == "0.000100"
    assert conditional.order_reserved == "340282366920938463463374607431768.211455"
    assert conditional.execution_reserved.to_decimal() == Decimal("0.00004")
    assert conditional.signed_remaining_atoms == -184_467_440_737_095_516_100
    assert conditional.observed_slot == 7
    assert conditional.observation_state is ObservationState.PRESENT

    assert global_account.source is FundingSource.GLOBAL
    assert global_account.market_pubkey is None
    assert global_account.raw_observed is None
    assert global_account.signed_remaining_atoms is None
    assert global_account.observed_slot is None
    assert global_account.observation_state is ObservationState.UNOBSERVED


def test_keeps_the_market_filter_of_a_market_scoped_page():
    response = PositionsResponse.from_dict(
        {
            "owner": WALLET,
            "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
            "funding_accounts": [],
            "committed_revision": "9",
            "projection_generation": "2",
            "has_more": True,
            "next_cursor": None,
        }
    )
    assert response.market_pubkey == "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9"
    assert response.funding_accounts == []
    assert response.has_more is True
    assert response.next_cursor is None


def test_unknown_observation_state_and_source_do_not_fail_the_page():
    account = FundingAccount.from_dict(
        conditional_account(observation_state="quarantined", source="vault")
    )
    assert account.observation_state is ObservationState.UNKNOWN
    assert account.source is FundingSource.UNKNOWN


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("order_reserved", 12),
        ("order_reserved", "-0"),
        ("order_reserved", "1e3"),
        ("raw_observed", 0.0001),
        ("signed_remaining_atoms", True),
        ("observed_slot", "-1"),
        ("accepted_boundary", None),
    ],
)
def test_names_the_field_of_a_malformed_account(field, value):
    with pytest.raises(DeserializationError, match=rf"funding_account\.{field}"):
        FundingAccount.from_dict(conditional_account(**{field: value}))


class RecordingHttp:
    def __init__(self):
        self.calls = []

    async def get(self, path, retry_policy=None, *, params=None, **_kwargs):
        self.calls.append(("get", path, params))
        return self._page()

    async def get_with_cookies(
        self, path, retry_policy=None, *, cookie_header, params=None
    ):
        self.calls.append(("get_with_cookies", path, params, cookie_header))
        return self._page()

    @staticmethod
    def _page():
        return {
            "owner": WALLET,
            "funding_accounts": [],
            "committed_revision": "1",
            "projection_generation": "1",
            "has_more": False,
        }


def test_pages_the_authenticated_routes_and_keeps_the_public_routes_query_free():
    http = RecordingHttp()
    positions = Positions(SimpleNamespace(_http=http))

    async def run():
        await positions.positions_page(cursor="next", limit=50)
        await positions.positions_page(MARKET)
        await positions.positions_for_market(MARKET)
        await positions.positions()
        await positions.get(WALLET)
        await positions.get_for_market(WALLET, MARKET)
        await positions.positions_page_with_cookies(MARKET, "c", None, "privy-token=t")
        await positions.positions_with_cookies("privy-token=t")

    asyncio.run(run())
    assert http.calls == [
        ("get", "/api/users/positions", {"cursor": "next", "limit": "50"}),
        ("get", f"/api/users/markets/{MARKET}/positions", None),
        ("get", f"/api/users/markets/{MARKET}/positions", None),
        ("get", "/api/users/positions", None),
        ("get", f"/api/users/{WALLET}/positions", None),
        ("get", f"/api/users/{WALLET}/markets/{MARKET}/positions", None),
        (
            "get_with_cookies",
            f"/api/users/markets/{MARKET}/positions",
            {"cursor": "c"},
            "privy-token=t",
        ),
        ("get_with_cookies", "/api/users/positions", None, "privy-token=t"),
    ]


def test_positions_page_rejects_a_bool_limit_before_sending():
    http = RecordingHttp()
    positions = Positions(SimpleNamespace(_http=http))
    with pytest.raises(ValidationError, match="limit"):
        asyncio.run(positions.positions_page(limit=True))
    assert http.calls == []
