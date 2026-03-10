"""Authentication types and utilities for the Lightcone SDK."""

from dataclasses import dataclass, field
from typing import Optional, Literal


# ---------------------------------------------------------------------------
# Type aliases
# ---------------------------------------------------------------------------

LinkedAccountType = Literal["wallet", "twitter_oauth", "google_oauth"]
ChainType = Literal["solana", "ethereum"]


# ---------------------------------------------------------------------------
# Dataclasses
# ---------------------------------------------------------------------------


@dataclass
class LinkedAccount:
    """A linked identity (wallet, Google OAuth, X OAuth) associated with a user."""

    id: str = ""
    type: LinkedAccountType = "wallet"
    chain: Optional[ChainType] = None
    address: str = ""


@dataclass
class EmbeddedWallet:
    """A Privy-managed embedded wallet."""

    privy_id: str = ""
    chain: ChainType = "solana"
    address: str = ""


@dataclass
class User:
    """Full user profile from the Lightcone platform."""

    id: str = ""
    wallet_address: str = ""
    linked_account: Optional[LinkedAccount] = None
    privy_id: Optional[str] = None
    embedded_wallet: Optional[EmbeddedWallet] = None
    x_username: Optional[str] = None
    x_user_id: Optional[str] = None
    x_display_name: Optional[str] = None
    google_email: Optional[str] = None


@dataclass
class AuthCredentials:
    """Internal auth session state. Token is NOT exposed."""

    user_id: str = ""
    wallet_address: str = ""
    expires_at: int = 0

    def is_valid(self) -> bool:
        """Whether the session is still valid (not expired)."""
        import time
        return time.time() < self.expires_at


@dataclass
class LoginRequest:
    """Login request body sent to the backend."""

    message: str = ""
    signature_bs58: str = ""
    pubkey_bytes: list[int] = field(default_factory=list)
    use_embedded_wallet: Optional[bool] = None


@dataclass
class LoginResponse:
    """Login response from the backend."""

    token: str = ""
    user_id: str = ""
    wallet_address: str = ""
    expires_at: int = 0
    linked_account: Optional[LinkedAccount] = None
    privy_id: Optional[str] = None
    embedded_wallet: Optional[EmbeddedWallet] = None
    x_username: Optional[str] = None
    x_user_id: Optional[str] = None
    x_display_name: Optional[str] = None
    google_email: Optional[str] = None


@dataclass
class MeResponse:
    """Response from GET /api/auth/me."""

    user_id: str = ""
    wallet_address: str = ""
    linked_account: Optional[LinkedAccount] = None
    privy_id: Optional[str] = None
    embedded_wallet: Optional[EmbeddedWallet] = None
    x_username: Optional[str] = None
    x_user_id: Optional[str] = None
    x_display_name: Optional[str] = None
    google_email: Optional[str] = None
    expires_at: int = 0


@dataclass
class NonceResponse:
    """Nonce response from the auth endpoint."""

    nonce: str = ""


# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------


def generate_signin_message(nonce: str) -> str:
    """Generate the sign-in message with a nonce.

    Format: "Sign in to Lightcone\\nNonce: {nonce}"
    """
    return f"Sign in to Lightcone\nNonce: {nonce}"


__all__ = [
    "LinkedAccountType",
    "ChainType",
    "LinkedAccount",
    "EmbeddedWallet",
    "User",
    "AuthCredentials",
    "LoginRequest",
    "LoginResponse",
    "MeResponse",
    "NonceResponse",
    "generate_signin_message",
]
