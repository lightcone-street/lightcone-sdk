"""Tests for payout-vector market resolution API payloads."""

from lightcone_sdk import (
    MarketResolutionKind,
    MarketResolutionPayout,
    MarketResolutionResponse,
)
from lightcone_sdk.domain.market.convert import market_from_wire
from lightcone_sdk.domain.market.wire import MarketWire
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
