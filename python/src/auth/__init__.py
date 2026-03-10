"""Authentication types and utilities for the Lightcone SDK.

Matches TS auth/index.ts with User, AuthCredentials, LinkedAccount types.
"""

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
    """A linked account (wallet, Twitter, Google, etc.)."""

    type: LinkedAccountType
    address: Optional[str] = None
    chain_type: Optional[ChainType] = None
    wallet_client_type: Optional[str] = None
    connector_type: Optional[str] = None
    subject: Optional[str] = None
    name: Optional[str] = None
    email: Optional[str] = None
    username: Optional[str] = None


@dataclass
class EmbeddedWallet:
    """An embedded (Privy-managed) wallet."""

    address: str
    chain_type: ChainType = "solana"


@dataclass
class User:
    """Authenticated user profile."""

    id: str
    created_at: Optional[str] = None
    linked_accounts: list[LinkedAccount] = field(default_factory=list)
    mfa_methods: list[str] = field(default_factory=list)
    has_accepted_terms: bool = False
    is_guest: bool = False
    wallet_address: Optional[str] = None
    privy_id: Optional[str] = None
    embedded_wallet: Optional[EmbeddedWallet] = None
    x_username: Optional[str] = None


@dataclass
class AuthCredentials:
    """Authentication credentials."""

    token: str
    user: Optional[User] = None
    user_id: Optional[str] = None
    wallet_address: Optional[str] = None
    expires_at: Optional[int] = None


@dataclass
class LoginRequest:
    """Login request body."""

    pubkey_bytes: list[int]
    message: str
    signature_bs58: str


@dataclass
class LoginResponse:
    """Login response body."""

    token: str
    user_id: str
    expires_at: int


@dataclass
class NonceResponse:
    """Nonce response from the auth endpoint."""

    nonce: str


# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------


def generate_signin_message(nonce: str) -> str:
    """Generate the sign-in message with a nonce.

    Format: "Sign in to Lightcone\\n\\nNonce: {nonce}"
    Matches TS auth/index.ts generateSigninMessage.
    """
    return f"Sign in to Lightcone\n\nNonce: {nonce}"


def is_authenticated(credentials: Optional[AuthCredentials]) -> bool:
    """Check if credentials are present and have a token."""
    return credentials is not None and bool(credentials.token)


__all__ = [
    "LinkedAccountType",
    "ChainType",
    "LinkedAccount",
    "EmbeddedWallet",
    "User",
    "AuthCredentials",
    "LoginRequest",
    "LoginResponse",
    "NonceResponse",
    "generate_signin_message",
    "is_authenticated",
]
