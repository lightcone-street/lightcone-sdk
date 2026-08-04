"""Tests for authentication profile helpers."""

from lightcone_sdk.auth import (
    GoogleAccountData,
    GoogleIdentity,
    PrivyEmbeddedWallet,
    User,
    UserIdentity,
    UserPrivyData,
    WalletIdentity,
    XAccountData,
    XIdentity,
)


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
    return User(user_id="user:test", identity=identity)


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

    assert google.wallet_display_name("privy") == "FRGk...WcPR"
    assert x.wallet_display_name("privy") == "So11...1112"
    assert wallet.wallet_display_name("lightcone") == "1111...1111"
    assert wallet.wallet_display_name("privy") == "Toke...Q5DA"
