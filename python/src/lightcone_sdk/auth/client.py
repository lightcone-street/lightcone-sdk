"""Authentication client for the Lightcone SDK."""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING

import base58
from nacl.signing import SigningKey
from solders.keypair import Keypair

from . import (
    AuthCredentials,
    GoogleAccountData,
    GoogleIdentity,
    PrivyEmbeddedWallet,
    SessionResponse,
    User,
    UserIdentity,
    UserPrivyData,
    WalletIdentity,
    XAccountData,
    XIdentity,
    generate_signin_message,
)
from ..error import DeserializationError, _require, is_unauthorized
from ..http.retry import RetryPolicy

if TYPE_CHECKING:
    from ..client import LightconeClient


class Auth:
    """Authentication operations.

    Nonce-based auth flow:
    1. Get nonce from server
    2. Sign message with nonce
    3. Submit signed message to login
    """

    def __init__(
        self,
        client: "LightconeClient",
        credentials: Optional[AuthCredentials] = None,
    ):
        self._client = client
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
        data = await self._client._http.get("/api/auth/nonce", retry_policy=RetryPolicy.NONE)
        return data.get("nonce", "")

    async def login_with_message(
        self,
        message: str,
        signature_bs58: str,
        pubkey_bytes: list[int],
        use_embedded_wallet: Optional[bool] = None,
    ) -> SessionResponse:
        """Login with a pre-signed message and return the session envelope.

        Args:
            message: The signed message
            signature_bs58: Base58-encoded signature
            pubkey_bytes: The public key as a list of bytes
            use_embedded_wallet: If True, provision a Privy embedded wallet

        Returns:
            The session envelope with the full user profile
        """
        body: dict = {
            "message": message,
            "signature_bs58": signature_bs58,
            "pubkey_bytes": pubkey_bytes,
        }
        if use_embedded_wallet is not None:
            body["use_embedded_wallet"] = use_embedded_wallet

        # Credential-management endpoint: opts out of the transport's 401
        # restore-and-replay. The backend consumes the login nonce before
        # verifying the signature, so a replayed login deterministically
        # fails — and restoring credentials in order to log in is circular.
        data = await self._client._http.post(
            "/api/auth/login_or_register_with_message",
            body,
            retry_policy=RetryPolicy.NONE,
            allow_credential_restore=False,
        )

        session = _session_from_dict(data)
        # Store credentials (token is extracted from set-cookie by the HTTP layer)
        self._credentials = _credentials_from_session(session)

        return session

    async def login(self, keypair: Keypair) -> SessionResponse:
        """Full login flow: get nonce, sign, submit.

        Args:
            keypair: Solana keypair for signing

        Returns:
            The session envelope with the full user profile
        """
        # Step 1: Get nonce
        nonce = await self.get_nonce()

        # Step 2: Sign message
        message, signature_b58, pubkey_bytes = sign_login_message(keypair, nonce)

        # Step 3: Login
        return await self.login_with_message(
            message, signature_b58, pubkey_bytes
        )

    async def check_session(self) -> SessionResponse:
        """Validate the current session and return the session envelope.

        On success, updates internal credentials. On failure, clears
        credentials and re-raises the error.

        Returns:
            The session envelope with the full user profile

        Raises:
            SdkError: If session is invalid or expired
        """
        try:
            data = await self._client._http.get(
                "/api/auth/me",
                retry_policy=RetryPolicy.IDEMPOTENT,
            )
            session = _session_from_dict(data)
            credentials = _credentials_from_session(session)
        except Exception:
            self._credentials = None
            raise

        self._credentials = credentials

        return session

    async def logout(self) -> None:
        """Logout — clears server-side cookie, internal token, and credentials.

        Local state is cleared even when the server call fails — the caller
        asked to be signed out locally regardless — but the failure is then
        re-raised: callers gating security decisions on teardown (e.g. whether
        an app may restart an authenticated transport) must be able to see
        that the server-side cookie may still be valid. A 401 counts as
        success: it means "already logged out".
        """
        logout_error: Exception | None = None
        try:
            # Credential-management endpoint: opts out of the transport's 401
            # restore-and-replay — a 401 here means "already logged out".
            await self._client._http.post(
                "/api/auth/logout", {},
                retry_policy=RetryPolicy.NONE,
                allow_credential_restore=False,
            )
        except Exception as error:
            if not is_unauthorized(error):
                logout_error = error

        self._client._http.clear_auth_token()
        self._credentials = None

        if logout_error is not None:
            raise logout_error

    async def disconnect_x(self) -> None:
        """Disconnect the user's linked X (Twitter) account."""
        await self._client._http.post(
            "/api/auth/disconnect_x", {},
            retry_policy=RetryPolicy.NONE,
        )

    async def update_max_slippage_preference(self, max_slippage_preference: str) -> str:
        """Persist the authenticated user's account-wide max-slippage preference."""
        data = await self._client._http.post(
            "/api/auth/max_slippage_preference",
            {"max_slippage_preference": max_slippage_preference},
            retry_policy=RetryPolicy.IDEMPOTENT,
        )
        persisted = _require(data, "max_slippage_preference", "max slippage response")
        if not isinstance(persisted, str):
            raise DeserializationError(
                "max slippage response has malformed max_slippage_preference"
            )
        return persisted

    def connect_x_url(self) -> str:
        """Get the URL for linking an X (Twitter) account via OAuth."""
        return f"{self._client._http.base_url}/api/auth/oauth/link/x"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _privy_from_dict(d: dict) -> UserPrivyData:
    wallet_dict = _require(d, "wallet", "privy data")
    if not isinstance(wallet_dict, dict):
        raise DeserializationError("privy data has malformed wallet")
    return UserPrivyData(
        id=str(_require(d, "id", "privy data")),
        wallet=PrivyEmbeddedWallet(
            privy_id=str(_require(wallet_dict, "privy_id", "privy wallet")),
            chain=_require(wallet_dict, "chain", "privy wallet"),  # type: ignore[arg-type]
            address=str(_require(wallet_dict, "address", "privy wallet")),
        ),
    )


def _x_account_from_dict(d: dict) -> XAccountData:
    return XAccountData(
        username=str(_require(d, "username", "x account")),
        user_id=d.get("user_id"),
        display_name=d.get("display_name"),
        avatar_url=d.get("avatar_url"),
    )


def _google_account_from_dict(d: dict) -> GoogleAccountData:
    return GoogleAccountData(
        email=str(_require(d, "email", "google account")),
        name=d.get("name"),
        given_name=d.get("given_name"),
        family_name=d.get("family_name"),
        avatar_url=d.get("avatar_url"),
    )


def _identity_from_dict(d: dict) -> UserIdentity:
    identity_type = _require(d, "type", "identity")
    if identity_type == "google":
        account = _require(d, "account", "google identity")
        privy = _require(d, "privy", "google identity")
        if not isinstance(account, dict) or not isinstance(privy, dict):
            raise DeserializationError("google identity has malformed account/privy")
        return GoogleIdentity(
            account=_google_account_from_dict(account),
            privy=_privy_from_dict(privy),
        )
    if identity_type == "x":
        account = _require(d, "account", "x identity")
        privy = _require(d, "privy", "x identity")
        if not isinstance(account, dict) or not isinstance(privy, dict):
            raise DeserializationError("x identity has malformed account/privy")
        return XIdentity(
            account=_x_account_from_dict(account),
            privy=_privy_from_dict(privy),
        )
    if identity_type == "wallet":
        privy_dict = d.get("privy")
        privy = _privy_from_dict(privy_dict) if isinstance(privy_dict, dict) else None
        return WalletIdentity(
            address=str(_require(d, "address", "wallet identity")),
            chain=_require(d, "chain", "wallet identity"),  # type: ignore[arg-type]
            privy=privy,
        )
    raise DeserializationError(f"unknown identity type: {identity_type!r}")


def _user_from_dict(d: dict) -> User:
    """Parse a User from the session envelope's `user` object."""
    identity_dict = _require(d, "identity", "user")
    if not isinstance(identity_dict, dict):
        raise DeserializationError("user has malformed identity")

    connected_x = None
    connected_x_dict = d.get("connected_x")
    if isinstance(connected_x_dict, dict):
        connected_x = _x_account_from_dict(connected_x_dict)

    # Older backend versions omit this newly added field. Normalize that rollout
    # state to unset while retaining strict validation when the key is present.
    max_slippage_preference = d.get("max_slippage_preference")
    if max_slippage_preference is not None and not isinstance(
        max_slippage_preference, str
    ):
        raise DeserializationError("user has malformed max_slippage_preference")

    return User(
        user_id=str(_require(d, "user_id", "user")),
        identity=_identity_from_dict(identity_dict),
        max_slippage_preference=max_slippage_preference,
        connected_x=connected_x,
    )


def _session_from_dict(d: dict) -> SessionResponse:
    """Parse the session envelope from an auth response."""
    user_dict = _require(d, "user", "session response")
    if not isinstance(user_dict, dict):
        raise DeserializationError("session response has malformed user")
    return SessionResponse(
        user=_user_from_dict(user_dict),
        expires_at=int(_require(d, "expires_at", "session response")),  # type: ignore[call-overload]
        auth_method=_require(d, "auth_method", "session response"),  # type: ignore[arg-type]
        is_beta=bool(d.get("is_beta", False)),
    )


def _credentials_from_session(session: SessionResponse) -> AuthCredentials:
    """Derive session credentials from the envelope. The trading wallet comes
    from the identity + auth method."""
    return AuthCredentials(
        user_id=session.user.user_id,
        wallet_address=session.user.trading_wallet(session.auth_method),
        expires_at=session.expires_at,
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
