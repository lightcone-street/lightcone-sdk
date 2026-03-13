"""Authentication client for the Lightcone SDK."""

import time
from typing import Optional, TYPE_CHECKING

import base58
from nacl.signing import SigningKey
from solders.keypair import Keypair

from . import (
    AuthCredentials,
    User,
    LinkedAccount,
    EmbeddedWallet,
    generate_signin_message,
)
from ..http.retry import RetryPolicy

if TYPE_CHECKING:
    from ..http.client import LightconeHttp


class Auth:
    """Authentication operations.

    Nonce-based auth flow:
    1. Get nonce from server
    2. Sign message with nonce
    3. Submit signed message to login
    """

    def __init__(
        self,
        http: "LightconeHttp",
        credentials: Optional[AuthCredentials] = None,
    ):
        self._http = http
        self._credentials: Optional[AuthCredentials] = credentials

    def credentials(self) -> Optional[AuthCredentials]:
        """Get current credentials."""
        return self._credentials

    def is_authenticated(self) -> bool:
        """Check if authenticated (based on cached credentials + expiry)."""
        if self._credentials is None:
            return False
        return self._credentials.is_authenticated()

    async def get_nonce(self) -> str:
        """Fetch a single-use nonce from the server for the sign-in challenge.

        Returns:
            The nonce string
        """
        data = await self._http.get("/api/auth/nonce", retry_policy=RetryPolicy.NONE)
        return data.get("nonce", "")

    async def login_with_message(
        self,
        message: str,
        signature_bs58: str,
        pubkey_bytes: list[int],
        use_embedded_wallet: Optional[bool] = None,
    ) -> User:
        """Login with a pre-signed message and return the full user profile.

        Args:
            message: The signed message
            signature_bs58: Base58-encoded signature
            pubkey_bytes: The public key as a list of bytes
            use_embedded_wallet: If True, provision a Privy embedded wallet

        Returns:
            Full User profile
        """
        body: dict = {
            "message": message,
            "signature_bs58": signature_bs58,
            "pubkey_bytes": pubkey_bytes,
        }
        if use_embedded_wallet is not None:
            body["use_embedded_wallet"] = use_embedded_wallet

        data = await self._http.post(
            "/api/auth/login_or_register_with_message",
            body,
            retry_policy=RetryPolicy.NONE,
        )

        # Store token in HTTP client (token is never exposed in AuthCredentials)
        token = data.get("token", "")
        self._http.set_auth_token(token)

        # Store credentials (without token)
        self._credentials = AuthCredentials(
            user_id=data.get("user_id", ""),
            wallet_address=data.get("wallet_address", ""),
            expires_at=data.get("expires_at", 0),
        )

        return _user_from_dict(data)

    async def login(self, keypair: Keypair) -> User:
        """Full login flow: get nonce, sign, submit.

        Args:
            keypair: Solana keypair for signing

        Returns:
            Full User profile
        """
        # Step 1: Get nonce
        nonce = await self.get_nonce()

        # Step 2: Sign message
        message, signature_b58, pubkey_bytes = sign_login_message(keypair, nonce)

        # Step 3: Login
        return await self.login_with_message(
            message, signature_b58, pubkey_bytes
        )

    async def check_session(self) -> User:
        """Validate the current session and return the full user profile.

        On success, updates internal credentials. On failure, clears
        credentials and re-raises the error.

        Returns:
            Full User profile

        Raises:
            SdkError: If session is invalid or expired
        """
        try:
            data = await self._http.get(
                "/api/auth/me",
                retry_policy=RetryPolicy.IDEMPOTENT,
            )
        except Exception:
            self._credentials = None
            raise

        self._credentials = AuthCredentials(
            user_id=data.get("user_id", ""),
            wallet_address=data.get("wallet_address", ""),
            expires_at=data.get("expires_at", 0),
        )

        return _user_from_dict(data)

    async def logout(self) -> None:
        """Logout — clears server-side cookie, internal token, and credentials."""
        try:
            await self._http.post(
                "/api/auth/logout", {},
                retry_policy=RetryPolicy.NONE,
            )
        except Exception:
            pass

        self._http.clear_auth_token()
        self._credentials = None

    async def disconnect_x(self) -> None:
        """Disconnect the user's linked X (Twitter) account."""
        await self._http.post(
            "/api/auth/disconnect_x", {},
            retry_policy=RetryPolicy.NONE,
        )

    async def connect_x(
        self,
        x_user_id: str,
        x_username: str,
        x_display_name: Optional[str] = None,
    ) -> None:
        """Link an X (Twitter) account to the user's profile.

        Args:
            x_user_id: X user ID
            x_username: X username
            x_display_name: X display name (optional)
        """
        body: dict = {
            "x_user_id": x_user_id,
            "x_username": x_username,
        }
        if x_display_name is not None:
            body["x_display_name"] = x_display_name

        await self._http.post(
            "/api/auth/connect_x", body,
            retry_policy=RetryPolicy.NONE,
        )


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _user_from_dict(d: dict) -> User:
    """Parse a User from an API response dict."""
    la = d.get("linked_account")
    if la and isinstance(la, dict):
        linked_account = LinkedAccount(
            id=la.get("id", ""),
            type=la.get("type", "wallet"),
            chain=la.get("chain"),
            address=la.get("address", ""),
        )
    else:
        linked_account = LinkedAccount()

    embedded_wallet = None
    ew = d.get("embedded_wallet")
    if ew and isinstance(ew, dict):
        embedded_wallet = EmbeddedWallet(
            privy_id=ew.get("privy_id", ""),
            chain=ew.get("chain", "solana"),
            address=ew.get("address", ""),
        )

    return User(
        id=d.get("user_id", d.get("id", "")),
        wallet_address=d.get("wallet_address", ""),
        linked_account=linked_account,
        privy_id=d.get("privy_id"),
        embedded_wallet=embedded_wallet,
        x_username=d.get("x_username"),
        x_user_id=d.get("x_user_id"),
        x_display_name=d.get("x_display_name"),
        google_email=d.get("google_email"),
    )


def sign_login_message(keypair: Keypair, nonce: str) -> tuple[str, str, list[int]]:
    """Sign a login message with a keypair.

    Args:
        keypair: Solana keypair
        nonce: The nonce from the server

    Returns:
        Tuple of (message, signature_bs58, pubkey_bytes)
    """
    message = generate_signin_message(nonce)
    message_bytes = message.encode("utf-8")

    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]
    signing_key = SigningKey(seed)
    signed = signing_key.sign(message_bytes)
    signature_b58 = base58.b58encode(signed.signature).decode("utf-8")

    pubkey_bytes = list(bytes(keypair.pubkey()))

    return message, signature_b58, pubkey_bytes


__all__ = [
    "Auth",
    "sign_login_message",
]
