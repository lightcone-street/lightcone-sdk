"""Tests for payout-vector market resolution API payloads."""

from decimal import Decimal

import pytest

from lightcone_sdk import (
    MarketResolutionKind,
    MarketResolutionPayout,
    MarketResolutionResponse,
)
from lightcone_sdk.domain.market import MarketValidationError
from lightcone_sdk.domain.market.convert import (
    market_from_wire,
    validation_errors_from_wire,
)
from lightcone_sdk.domain.market.wire import MarketSearchResult, MarketWire
from lightcone_sdk.domain.notification import Notification
from lightcone_sdk.domain.notification.client import _parse_notification

NOW = "2026-05-06T13:00:00Z"


def scalar_resolution_dict() -> dict:
    return {
        "kind": "scalar",
        "payout_denominator": 10,
        "payouts": [
            {"outcome_index": 0, "payout_numerator": 7},
            {"outcome_index": 1, "payout_numerator": 3},
        ],
        "single_winning_outcome": None,
    }


def single_winner_resolution_dict() -> dict:
    return {
        "kind": "single_winner",
        "payout_denominator": 1,
        "payouts": [
            {"outcome_index": 0, "payout_numerator": 0},
            {"outcome_index": 1, "payout_numerator": 1},
        ],
        "single_winning_outcome": 1,
    }


def market_payload(resolution: dict | None = None) -> dict:
    payload = {
        "market_name": "Test Market",
        "slug": "test-market",
        "description": "Description",
        "definition": "Definition",
        "outcomes": [
            {
                "index": 0,
                "name": "Yes",
                "icon_url_low": "https://example.com/yes-low.png",
            },
            {
                "index": 1,
                "name": "No",
                "icon_url_low": "https://example.com/no-low.png",
            },
        ],
        "banner_image_url_low": "https://example.com/banner-low.png",
        "icon_url_low": "https://example.com/icon-low.png",
        "market_pubkey": "market_1",
        "market_id": 1,
        "num_outcomes": 2,
        "oracle": "oracle",
        "question_id": "question",
        "condition_id": "condition",
        "market_status": "Resolved",
        "created_at": NOW,
        "settled_at": NOW,
        "deposit_assets": [
            {
                "display_name": "USD Coin",
                "symbol": "USDC",
                "deposit_asset": "USDC",
                "id": 1,
                "market_pubkey": "market_1",
                "vault": "vault",
                "num_outcomes": 2,
                "icon_url_low": "https://example.com/usdc-low.png",
                "decimals": 6,
                "min_order_size": "1.000000",
                "conditional_mints": [
                    {
                        "id": 10,
                        "outcome_index": 0,
                        "token_address": "yes_mint",
                        "outcome": "Yes",
                        "short_symbol": "YES",
                        "decimals": 6,
                        "created_at": NOW,
                    },
                    {
                        "id": 11,
                        "outcome_index": 1,
                        "token_address": "no_mint",
                        "outcome": "No",
                        "short_symbol": "NO",
                        "decimals": 6,
                        "created_at": NOW,
                    },
                ],
                "created_at": NOW,
            },
        ],
        "orderbooks": [
            {
                "id": 1,
                "market_pubkey": "market_1",
                "orderbook_id": "ob_yes_no",
                "base_token": "yes_mint",
                "quote_token": "no_mint",
                "outcome_index": 0,
                "tick_size": 1,
                "total_bids": 0,
                "total_asks": 0,
                "active": True,
                "created_at": NOW,
                "updated_at": NOW,
            },
        ],
    }
    if resolution is not None:
        payload["resolution"] = resolution
    return payload


def test_optional_metadata_fields_do_not_fail_validation() -> None:
    payload = market_payload()
    del payload["description"]
    del payload["banner_image_url_low"]

    wire = MarketWire.from_dict(payload)
    assert validation_errors_from_wire(wire) == []

    market = market_from_wire(wire)
    assert market.description is None
    assert market.definition == "Definition"
    assert market.banner_image_url_low is None
    assert market.banner_image_url_medium is None
    assert market.banner_image_url_high is None
    assert market.subcategory is None
    assert market.tags == []


def test_outcome_artwork_is_optional_and_cross_fills_non_blank_quality() -> None:
    payload = market_payload()
    payload["outcomes"][0] = {"index": 0, "name": "Yes"}
    payload["outcomes"][1] = {
        "index": 1,
        "name": "No",
        "icon_url_low": " ",
        "icon_url_medium": "https://example.com/no.png",
        "icon_url_high": "",
    }

    market = market_from_wire(MarketWire.from_dict(payload))
    assert market.outcomes[0].icon_url_low is None
    assert market.outcomes[0].icon_url_medium is None
    assert market.outcomes[0].icon_url_high is None
    assert market.outcomes[1].icon_url_low == "https://example.com/no.png"
    assert market.outcomes[1].icon_url_medium == "https://example.com/no.png"
    assert market.outcomes[1].icon_url_high == "https://example.com/no.png"

    preserved = market_from_wire(MarketWire.from_dict(market_payload())).outcomes[0]
    assert preserved.icon_url_low == "https://example.com/yes-low.png"
    assert preserved.icon_url_medium == "https://example.com/yes-low.png"
    assert preserved.icon_url_high == "https://example.com/yes-low.png"


def test_optional_metadata_fields_pass_through_when_present() -> None:
    payload = market_payload()
    payload["subcategory"] = "Bitcoin"
    payload["tags"] = ["btc"]
    payload["resolution_by"] = 1_760_000_000_000

    market = market_from_wire(MarketWire.from_dict(payload))
    assert market.description == "Description"
    assert market.definition == "Definition"
    assert market.subcategory == "Bitcoin"
    assert market.tags == ["btc"]
    assert market.resolution_by == 1_760_000_000_000


def test_resolution_by_is_nullable_on_full_and_search_markets() -> None:
    market = market_from_wire(MarketWire.from_dict(market_payload()))
    search_result = MarketSearchResult.from_dict(
        {
            "slug": "test-market",
            "market_name": "Test Market",
            "resolution_by": None,
        }
    )

    assert market.resolution_by is None
    assert search_result.resolution_by is None


def test_market_outcome_count_comes_from_market_response() -> None:
    payload = market_payload()
    payload["outcomes"] = []

    market = market_from_wire(MarketWire.from_dict(payload))

    assert market.num_outcomes == 2
    assert market.outcomes == []


def test_market_outcome_count_falls_back_to_deposit_asset() -> None:
    payload = market_payload()
    del payload["num_outcomes"]

    market = market_from_wire(MarketWire.from_dict(payload))

    assert market.num_outcomes == 2


def test_null_market_outcome_count_falls_back_to_deposit_asset() -> None:
    payload = market_payload()
    payload["num_outcomes"] = None

    market = market_from_wire(MarketWire.from_dict(payload))

    assert market.num_outcomes == 2


def test_market_rejects_inconsistent_outcome_counts() -> None:
    payload = market_payload()
    payload["deposit_assets"][0]["num_outcomes"] = 3
    wire = MarketWire.from_dict(payload)

    assert "do not match market" in validation_errors_from_wire(wire)[0]
    with pytest.raises(MarketValidationError, match="do not match market"):
        market_from_wire(wire)


@pytest.mark.parametrize("num_outcomes", [True, 2.0, "2"])
def test_market_rejects_non_integer_outcome_counts(num_outcomes: object) -> None:
    payload = market_payload()
    payload["num_outcomes"] = num_outcomes
    wire = MarketWire.from_dict(payload)

    assert "Invalid outcome count" in validation_errors_from_wire(wire)[0]
    with pytest.raises(MarketValidationError, match="Invalid outcome count"):
        market_from_wire(wire)


def test_fee_bps_null_or_missing_reads_as_zero() -> None:
    # Older backends omit the fee fields entirely, and a backend could also
    # serialize them as JSON null; both must read as zero so domain Market
    # keeps its plain-int fee fields. Explicit values (including a negative
    # maker rebate) must pass through untouched.
    null_fees = market_payload()
    null_fees["maker_fee_bps"] = None
    null_fees["taker_fee_bps"] = None
    signed_fees = market_payload()
    signed_fees["maker_fee_bps"] = -5
    signed_fees["taker_fee_bps"] = 40

    null_market = market_from_wire(MarketWire.from_dict(null_fees))
    missing_market = market_from_wire(MarketWire.from_dict(market_payload()))
    signed_market = market_from_wire(MarketWire.from_dict(signed_fees))

    assert null_market.maker_fee_bps == 0
    assert null_market.taker_fee_bps == 0
    assert missing_market.maker_fee_bps == 0
    assert missing_market.taker_fee_bps == 0
    assert signed_market.maker_fee_bps == -5
    assert signed_market.taker_fee_bps == 40


@pytest.mark.parametrize("definition", [None, "", 1])
def test_definition_is_required(definition: object) -> None:
    payload = market_payload()
    payload["definition"] = definition
    wire = MarketWire.from_dict(payload)

    assert "Missing definition" in validation_errors_from_wire(wire)[0]
    with pytest.raises(MarketValidationError, match="Missing definition"):
        market_from_wire(wire)


def test_banner_urls_cross_fallback_when_partially_set() -> None:
    payload = market_payload()
    del payload["banner_image_url_low"]
    payload["banner_image_url_high"] = "https://example.com/banner-high.png"

    market = market_from_wire(MarketWire.from_dict(payload))
    assert market.banner_image_url_low == "https://example.com/banner-high.png"
    assert market.banner_image_url_medium == "https://example.com/banner-high.png"
    assert market.banner_image_url_high == "https://example.com/banner-high.png"


def test_resolution_response_from_dict_parses_scalar() -> None:
    resolution = MarketResolutionResponse.from_dict(scalar_resolution_dict())

    assert resolution.kind == MarketResolutionKind.SCALAR
    assert resolution.payout_denominator == 10
    assert resolution.single_winning_outcome is None
    assert resolution.payouts == [
        MarketResolutionPayout(outcome_index=0, payout_numerator=7),
        MarketResolutionPayout(outcome_index=1, payout_numerator=3),
    ]


def test_market_wire_helpers_distinguish_scalar_from_unresolved() -> None:
    unresolved = MarketWire.from_dict(market_payload())
    assert unresolved.is_resolved() is False
    assert unresolved.single_winning_outcome() is None
    assert unresolved.has_single_winning_outcome() is False

    scalar = MarketWire.from_dict(market_payload(scalar_resolution_dict()))
    assert scalar.is_resolved() is True
    assert scalar.single_winning_outcome() is None
    assert scalar.has_single_winning_outcome() is False

    single_winner = MarketWire.from_dict(
        market_payload(single_winner_resolution_dict())
    )
    assert single_winner.is_resolved() is True
    assert single_winner.single_winning_outcome() == 1
    assert single_winner.has_single_winning_outcome() is True


def test_market_conversion_preserves_scalar_resolution() -> None:
    wire = MarketWire.from_dict(market_payload(scalar_resolution_dict()))
    market = market_from_wire(wire)

    assert market.is_resolved() is True
    assert market.single_winning_outcome() is None
    assert market.has_single_winning_outcome() is False
    assert market.resolution is not None
    assert market.resolution.kind == MarketResolutionKind.SCALAR
    assert [p.payout_numerator for p in market.resolution.payouts] == [7, 3]
    assert market.deposit_assets[0].min_order_size == Decimal("1")


def test_market_conversion_preserves_single_winner_resolution() -> None:
    wire = MarketWire.from_dict(market_payload(single_winner_resolution_dict()))
    market = market_from_wire(wire)

    assert market.is_resolved() is True
    assert market.single_winning_outcome() == 1
    assert market.has_single_winning_outcome() is True
    assert market.resolution is not None
    assert market.resolution.kind == MarketResolutionKind.SINGLE_WINNER


def test_notification_from_dict_deserializes_market_resolution() -> None:
    notification = Notification.from_dict(
        {
            "id": "notif_1",
            "notification_type": "market_resolved",
            "data": {
                "market_pubkey": "market_1",
                "market_slug": "test-market",
                "market_name": "Test Market",
                "resolution": scalar_resolution_dict(),
            },
            "title": "Market resolved",
            "message": "The market has resolved.",
            "created_at": NOW,
        }
    )

    assert notification.market_resolved_data is not None
    resolution = notification.market_resolved_data.resolution
    assert resolution is not None
    assert resolution.kind == MarketResolutionKind.SCALAR
    assert resolution.single_winning_outcome is None


def test_notification_client_parser_deserializes_market_resolution() -> None:
    notification = _parse_notification(
        {
            "id": "notif_1",
            "notification_type": "market_resolved",
            "data": {
                "market_pubkey": "market_1",
                "market_slug": "test-market",
                "market_name": "Test Market",
                "resolution": scalar_resolution_dict(),
            },
            "title": "Market resolved",
            "message": "The market has resolved.",
            "created_at": NOW,
        }
    )

    assert notification.market_resolved_data is not None
    resolution = notification.market_resolved_data.resolution
    assert resolution is not None
    assert resolution.kind == MarketResolutionKind.SCALAR
    assert resolution.payout_denominator == 10
