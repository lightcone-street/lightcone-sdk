"""Tests for authentication profile helpers."""

from types import SimpleNamespace

import pytest

from lightcone_sdk.auth import (
    EmailAccountData,
    EmailIdentity,
    EmailLinkedIdentity,
    GoogleAccountData,
    GoogleIdentity,
    LinkedIdentitySelector,
    PrivyEmbeddedWallet,
    RegisterPrivyRequest,
    User,
    UserIdentity,
    UserPrivyData,
    WalletIdentity,
    XAccountData,
    XIdentity,
    classify_register_privy_conflict,
)
from lightcone_sdk.auth.client import Auth, _user_from_dict
from lightcone_sdk.error import ApiRejected, DeserializationError
from lightcone_sdk.http.retry import RetryPolicy
from lightcone_sdk.shared import ApiRejectedDetails


def privy(address: str) -> UserPrivyData:
    return UserPrivyData(
        id="did:privy:test",
        wallet=PrivyEmbeddedWallet(
            privy_id="wallet:test",
            chain="solana",
            address=address,
        ),
    )


def user(identity: UserIdentity) -> User:
    return User(
        user_id="user:test",
        identity=identity,
        max_slippage_preference=None,
    )


def test_wallet_display_name_uses_the_session_trading_wallet():
    google_wallet = "FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR"
    x_wallet = "So11111111111111111111111111111111111111112"
    sign_in_wallet = "11111111111111111111111111111111"
    embedded_wallet = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

    google = user(
        GoogleIdentity(
            account=GoogleAccountData(
                email="user@example.com",
                name="Google User",
            ),
            privy=privy(google_wallet),
        )
    )
    x = user(
        XIdentity(
            account=XAccountData(
                user_id="123",
                username="x_user",
                display_name="X User",
            ),
            privy=privy(x_wallet),
        )
    )
    wallet = user(
        WalletIdentity(
            address=sign_in_wallet,
            chain="solana",
            privy=privy(embedded_wallet),
        )
    )
    wallet_no_privy = user(
        WalletIdentity(
            address=sign_in_wallet,
            chain="solana",
        )
    )

    assert google.wallet_display_name("privy") == "FRGk...WcPR"
    assert x.wallet_display_name("privy") == "So11...1112"
    assert wallet.wallet_display_name("lightcone") == "1111...1111"
    assert wallet.wallet_display_name("privy") == "Toke...Q5DA"
    assert wallet_no_privy.wallet_display_name("privy") == "1111...1111"


def test_email_identity_and_linked_method_shape():
    email = user(
        EmailIdentity(
            account=EmailAccountData(email="verified@example.com"),
            privy=privy("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR"),
        )
    )
    email.linked_identities = [
        EmailLinkedIdentity(account=EmailAccountData(email="verified@example.com"))
    ]
    assert email.display_name() == "verified@example.com"
    assert (
        email.trading_wallet("privy") == "FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR"
    )


def test_email_display_name_is_limited_to_twenty_characters():
    email = user(
        EmailIdentity(
            account=EmailAccountData(email="lightconewebtesting@gmail.com"),
            privy=privy("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR"),
        )
    )

    assert email.display_name() == "lightcon...gmail.com"
    assert len(email.display_name()) == 20


@pytest.mark.asyncio
async def test_register_privy_posts_attempted_selector():
    calls: list[tuple[str, dict, RetryPolicy]] = []

    class Http:
        async def post(
            self, path: str, body: dict, *, retry_policy: RetryPolicy
        ) -> dict:
            calls.append((path, body, retry_policy))
            return {}

    request = RegisterPrivyRequest(
        attempted_identity=LinkedIdentitySelector(
            type="email", email="verified@example.com"
        )
    )
    await Auth(SimpleNamespace(_http=Http())).register_privy(request)  # type: ignore[arg-type]
    assert calls == [
        (
            "/api/auth/register-privy",
            {
                "attempted_identity": {
                    "type": "email",
                    "email": "verified@example.com",
                }
            },
            RetryPolicy.NONE,
        )
    ]


@pytest.mark.parametrize(
    "selector",
    [
        {"type": "email"},
        {"type": "email", "email": "verified@example.com", "username": "wrong"},
        {"type": "google", "email": ""},
        {"type": "x", "email": "wrong@example.com"},
        {"type": "wallet", "address": "wallet-only"},
    ],
)
def test_linked_identity_selector_rejects_invalid_variant_fields(selector):
    with pytest.raises(ValueError):
        LinkedIdentitySelector(**selector)


def test_user_parser_accepts_omitted_or_nullable_string_max_slippage_preference():
    base = {
        "user_id": "user:test",
        "identity": {
            "type": "wallet",
            "address": "11111111111111111111111111111111",
            "chain": "solana",
        },
    }
    assert _user_from_dict(base).max_slippage_preference is None
    assert (
        _user_from_dict(
            {**base, "max_slippage_preference": None}
        ).max_slippage_preference
        is None
    )
    assert (
        _user_from_dict(
            {**base, "max_slippage_preference": "5.50"}
        ).max_slippage_preference
        == "5.50"
    )
    with pytest.raises(DeserializationError):
        _user_from_dict({**base, "max_slippage_preference": 10})


@pytest.mark.asyncio
async def test_update_max_slippage_preference_uses_exact_contract():
    calls: list[tuple[str, dict, RetryPolicy]] = []

    class Http:
        async def post(
            self,
            path: str,
            body: dict,
            *,
            retry_policy: RetryPolicy,
        ) -> dict:
            calls.append((path, body, retry_policy))
            return {"max_slippage_preference": "5.50"}

    auth = Auth(SimpleNamespace(_http=Http()))  # type: ignore[arg-type]
    persisted = await auth.update_max_slippage_preference("5.50")

    assert persisted == "5.50"
    assert calls == [
        (
            "/api/auth/max_slippage_preference",
            {"max_slippage_preference": "5.50"},
            RetryPolicy.IDEMPOTENT,
        )
    ]


@pytest.mark.asyncio
async def test_update_max_slippage_preference_rejects_non_string_response():
    class Http:
        async def post(
            self,
            path: str,
            body: dict,
            *,
            retry_policy: RetryPolicy,
        ) -> dict:
            return {"max_slippage_preference": 12.5}

    auth = Auth(SimpleNamespace(_http=Http()))  # type: ignore[arg-type]
    with pytest.raises(DeserializationError):
        await auth.update_max_slippage_preference("5.50")


def test_register_privy_conflicts_require_exact_codes_and_typed_methods():
    conflict = ApiRejected(
        ApiRejectedDetails(
            reason="Identity belongs to another account",
            error_code="IDENTITY_OWNED_BY_ANOTHER_ACCOUNT",
            existing_method="google",
            http_status=409,
        )
    )
    classified = classify_register_privy_conflict(conflict)
    assert classified is not None
    assert classified.code == "IDENTITY_OWNED_BY_ANOTHER_ACCOUNT"
    assert classified.existing_method == "google"

    unrelated = ApiRejected(
        ApiRejectedDetails(
            reason="Conflict",
            error_code="RESOURCE_CONFLICT",
            existing_method="email",
            http_status=409,
        )
    )
    assert classify_register_privy_conflict(unrelated) is None
