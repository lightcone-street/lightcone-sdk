"""Engine rejection codes and transport error codes."""

import json

import pytest

from lightcone_sdk.shared import (
    ApiRejectedDetails,
    ApiResponse,
    ErrorCode,
    RejectionCode,
)

REJECTION_CODES = [
    "DUPLICATE_ORDER",
    "INSUFFICIENT_BALANCE",
    "FOK_INSUFFICIENT_LIQUIDITY",
    "SELF_TRADE",
    "INVALID_ORDER",
    "INVALID_REQUEST",
    "ORDER_EXPIRED",
    "TRADING_PAUSED",
    "INVALID_SIGNATURE",
    "CANCEL_ALL_REPLAY",
    "STALE_STATE",
    "REQUEST_EXPIRED",
    "TRADING_CAPACITY_UNAVAILABLE",
    "TRADING_NOT_READY",
    "INTERNAL_ERROR",
    "ORDER_NOT_FOUND",
    "ORDERBOOK_NOT_FOUND",
]

ERROR_CODES = [
    "INVALID_ARGUMENT",
    "FAILED_PRECONDITION",
    "NOT_FOUND",
    "FORBIDDEN",
    "ALREADY_EXISTS",
    "ABORTED",
    "RESOURCE_EXHAUSTED",
    "ENGINE_UNAVAILABLE",
    "ENGINE_INTERNAL_ERROR",
    "TRADING_UNAVAILABLE",
    "AUTH_REQUIRED",
    "AUTH_WALLET_MISMATCH",
    "INVALID_TIF",
    "INVALID_DEPOSIT_SOURCE",
    "INVALID_SIGNATURE",
    "INVALID_PUBKEY",
    "INVALID_CURSOR",
    "INVALID_ORDER_HASH",
    "UNEXPECTED_QUERY",
    "INVALID_LIMIT",
    "RATE_LIMITED",
    "ORDERBOOK_DECIMALS_NOT_FOUND",
    "ORDERBOOK_CONFIGURATION_ERROR",
    "NON_CANONICAL_ENCODING",
]


def rejected_details(body: str) -> ApiRejectedDetails:
    response = ApiResponse.from_dict(json.loads(body))
    assert response.status == "error"
    assert response.details is not None
    return response.details


class TestRejectionCode:
    def test_matches_any_case_format(self) -> None:
        assert RejectionCode.from_wire("INSUFFICIENT_BALANCE").known() == (
            "INSUFFICIENT_BALANCE"
        )
        assert RejectionCode.from_wire("insufficient_balance").known() == (
            "INSUFFICIENT_BALANCE"
        )

    def test_keeps_an_unknown_codes_original_text(self) -> None:
        code = RejectionCode.from_wire("new_thing")
        assert not code.is_known()
        assert code.label() == "new_thing"
        assert code.wire_name() == "new_thing"

    @pytest.mark.parametrize(
        "removed",
        [
            "NONCE_MISMATCH",
            "INVALID_NONCE",
            "FOK_NO_FILL",
            "BELOW_MIN_ORDER_SIZE",
            "EXPIRED",
            "PRICE_NOT_EXACTLY_REPRESENTABLE",
        ],
    )
    def test_decodes_retired_codes_as_unknown(self, removed: str) -> None:
        code = RejectionCode.from_wire(removed)
        assert not code.is_known()
        assert code.label() == removed

    def test_labels_known_codes(self) -> None:
        assert RejectionCode.from_wire("INSUFFICIENT_BALANCE").label() == (
            "Insufficient Balance"
        )
        assert RejectionCode.from_wire("SELF_TRADE").label() == "Self Trade"
        assert RejectionCode.from_wire("FOK_INSUFFICIENT_LIQUIDITY").label() == (
            "FOK Insufficient Liquidity"
        )
        assert str(RejectionCode.from_wire("INSUFFICIENT_BALANCE")) == (
            "Insufficient Balance"
        )

    def test_round_trips_every_engine_code(self) -> None:
        for wire in REJECTION_CODES:
            code = RejectionCode.from_wire(wire)
            assert code.is_known(), wire
            assert code.wire_name() == wire
            assert code.label() != wire

    def test_treats_only_the_retryable_admission_failures_as_transient(self) -> None:
        assert RejectionCode.from_wire("STALE_STATE").is_transient()
        assert RejectionCode.from_wire("TRADING_NOT_READY").is_transient()
        assert not RejectionCode.from_wire("DUPLICATE_ORDER").is_transient()
        assert not RejectionCode.from_wire("INSUFFICIENT_BALANCE").is_transient()
        assert not RejectionCode.from_wire("UNKNOWN_FUTURE").is_transient()


class TestErrorCode:
    def test_round_trips_every_code_and_tolerates_unknown_values(self) -> None:
        for wire in ERROR_CODES:
            code = ErrorCode.from_wire(wire)
            assert code.known() == wire
            assert code.wire_name() == wire
        unknown = ErrorCode.from_wire("WALLET_NOT_AUTHORIZED")
        assert not unknown.is_known()
        assert unknown.wire_name() == "WALLET_NOT_AUTHORIZED"
        # Error codes match exactly; lowercase is not the wire form.
        assert not ErrorCode.from_wire("engine_unavailable").is_known()
        assert ErrorCode.from_wire("ENGINE_UNAVAILABLE").is_retryable()
        assert not ErrorCode.from_wire("AUTH_WALLET_MISMATCH").is_retryable()
        assert ErrorCode.from_wire(None) is None

    @pytest.mark.parametrize(
        "code",
        [
            "ABORTED",
            "RESOURCE_EXHAUSTED",
            "ENGINE_UNAVAILABLE",
            "TRADING_UNAVAILABLE",
            "AUTH_WALLET_MISMATCH",
            "FORBIDDEN",
            "ALREADY_EXISTS",
            "INVALID_ARGUMENT",
        ],
    )
    def test_types_engine_error_codes_from_the_envelope(self, code: str) -> None:
        details = rejected_details(
            '{"status":"error","error_details":{"reason":"Engine temporarily '
            f'unavailable","error_code":"{code}","error_log_id":"LCERR_1"}}}}'
        )
        assert details.error_code == code
        kind = details.error_code_kind()
        assert kind is not None and kind.known() == code

    def test_carries_a_business_rejection_on_a_200_envelope_without_an_error_code(
        self,
    ) -> None:
        details = rejected_details(
            '{"status":"error","error_details":{"reason":"The full original BASE '
            'target is not executable","rejection_code":"FOK_INSUFFICIENT_LIQUIDITY",'
            '"error_log_id":"LCERR_2"}}'
        )
        assert details.rejection_code is not None
        assert details.rejection_code.known() == "FOK_INSUFFICIENT_LIQUIDITY"
        assert details.error_code_kind() is None
