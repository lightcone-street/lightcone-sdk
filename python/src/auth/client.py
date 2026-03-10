"""Authentication client for the Lightcone SDK.

Matches TS auth/client.ts with nonce-based login flow.
"""

from typing import Optional, TYPE_CHECKING

import base58
from nacl.signing import SigningKey
from solders.keypair import Keypair

from . import (
    AuthCredentials,
    User,
    NonceResponse,
    LoginResponse,
    generate_signin_message,
)
from ..error import AuthError

if TYPE_CHECKING:
    from ..http.client import LightconeHttp


class Auth:
    """Authentication operations.

    Implements nonce-based auth flow matching TS Auth class:
    1. Get nonce from server
    2. Sign message with nonce
    3. Submit signed message to login
    """

    def __init__(self, http: "LightconeHttp"):
        self._http = http
        self._credentials: Optional[AuthCredentials] = None

    def credentials(self) -> Optional[AuthCredentials]:
        """Get current credentials."""
        return self._credentials

    def is_authenticated(self) -> bool:
        """Check if authenticated."""
        return self._credentials is not None and bool(self._credentials.token)

    async def get_nonce(self, pubkey: str) -> str:
        """Get a nonce for the login message.

        Args:
            pubkey: The user's public key (Base58)

        Returns:
            The nonce string
        """
        data = await self._http.post("/auth/nonce", {"pubkey": pubkey})
        return data.get("nonce", "")

    async def login_with_message(
        self,
        pubkey_bytes: list[int],
        message: str,
        signature_bs58: str,
    ) -> AuthCredentials:
        """Login with a pre-signed message.

        Args:
            pubkey_bytes: The public key as a list of bytes
            message: The signed message
            signature_bs58: Base58-encoded signature

        Returns:
            AuthCredentials with token and user info
        """
        data = await self._http.post(
            "/auth/login_or_register_with_message",
            {
                "pubkey_bytes": pubkey_bytes,
                "message": message,
                "signature_bs58": signature_bs58,
            },
        )

        creds = AuthCredentials(
            token=data.get("token", ""),
            user_id=data.get("user_id"),
            expires_at=data.get("expires_at"),
        )
        self._credentials = creds
        self._http.set_auth_token(creds.token)
        return creds

    async def login(self, keypair: Keypair) -> AuthCredentials:
        """Full login flow: get nonce, sign, submit.

        Args:
            keypair: Solana keypair for signing

        Returns:
            AuthCredentials
        """
        pubkey = str(keypair.pubkey())

        # Step 1: Get nonce
        nonce = await self.get_nonce(pubkey)

        # Step 2: Sign message
        message = generate_signin_message(nonce)
        message_bytes = message.encode("utf-8")

        secret_bytes = bytes(keypair)
        seed = secret_bytes[:32]
        signing_key = SigningKey(seed)
        signed = signing_key.sign(message_bytes)
        signature_b58 = base58.b58encode(signed.signature).decode("utf-8")

        pubkey_bytes = list(bytes(keypair.pubkey()))

        # Step 3: Login
        return await self.login_with_message(pubkey_bytes, message, signature_b58)

    async def check_session(self) -> User:
        """Check the current session.

        Returns:
            User profile if session is valid

        Raises:
            AuthError: If not authenticated
        """
        if not self.is_authenticated():
            raise AuthError.not_authenticated()

        data = await self._http.get("/auth/session")
        return User(
            id=data.get("id", ""),
            wallet_address=data.get("wallet_address"),
            x_username=data.get("x_username"),
        )

    async def logout(self) -> None:
        """Logout and clear credentials."""
        if self.is_authenticated():
            try:
                await self._http.post("/auth/logout", {})
            except Exception:
                pass
        self._credentials = None
        self._http.set_auth_token(None)

    async def disconnect_x(self) -> User:
        """Disconnect Twitter/X account.

        Returns:
            Updated User profile
        """
        data = await self._http.post("/auth/disconnect-x", {})
        return User(
            id=data.get("id", ""),
            wallet_address=data.get("wallet_address"),
        )

    async def connect_x(self, redirect_url: str) -> str:
        """Start Twitter/X connection flow.

        Args:
            redirect_url: URL to redirect after OAuth

        Returns:
            OAuth authorization URL
        """
        data = await self._http.post(
            "/auth/connect-x",
            {"redirect_url": redirect_url},
        )
        return data.get("url", "")


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
