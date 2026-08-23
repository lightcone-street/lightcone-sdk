"""Authentication types and utilities for the Lightcone SDK."""

from dataclasses import dataclass, field
from typing import Literal, Optional, Union, cast

from ..shared.api_response import LinkedIdentityType
from ..shared.fmt.str import shorten

# ---------------------------------------------------------------------------
# Type aliases
# ---------------------------------------------------------------------------

ChainType = Literal["solana", "ethereum"]
# How a session authenticated, as reported by the backend (derived from which
# token verified the request). "privy" => embedded wallet is the trading
# identity; "lightcone" => self-custody session, the identity wallet is.
AuthMethod = Literal["privy", "lightcone"]


# ---------------------------------------------------------------------------
# Dataclasses
# ---------------------------------------------------------------------------


@dataclass
class PrivyEmbeddedWallet:
    """A Privy-managed embedded wallet."""

    privy_id: str
    chain: ChainType
    address: str


@dataclass
class UserPrivyData:
    """Privy account data attached to an identity."""

    id: str
    """The Privy DID (`did:privy:...`)."""
    wallet: PrivyEmbeddedWallet
    """Always present: Privy registration provisions the embedded wallet in
    the same transaction that creates the user."""


@dataclass
class XAccountData:
    """X account data — the same shape whether X is the login identity or a
    connected account on a Google/wallet identity."""

    username: str
    user_id: Optional[str] = None
    """X numeric user id (Privy `subject`); absent on legacy rows."""
    display_name: Optional[str] = None
    avatar_url: Optional[str] = None


@dataclass
class GoogleAccountData:
    """Google account data for a Google login identity."""

    email: str
    name: Optional[str] = None
    given_name: Optional[str] = None
    family_name: Optional[str] = None
    avatar_url: Optional[str] = None


@dataclass
class EmailAccountData:
    """Canonical address for a passwordless Email login identity."""

    email: str


# The login identity — how the user authenticates, discriminated by `type`.
# Privy data lives on the variant because its presence is determined by the
# identity type: Google/X login only exists via Privy OAuth (guaranteed DID +
# embedded wallet), while wallet users opt into Privy (SIWS) or stay
# self-custody.


@dataclass
class GoogleIdentity:
    account: GoogleAccountData
    privy: UserPrivyData
    type: Literal["google"] = "google"


@dataclass
class EmailIdentity:
    """Primary Email Identity and the Privy wallet used by its session."""

    account: EmailAccountData
    privy: UserPrivyData
    type: Literal["email"] = "email"


@dataclass
class XIdentity:
    account: XAccountData
    privy: UserPrivyData
    type: Literal["x"] = "x"


@dataclass
class WalletIdentity:
    address: str
    chain: ChainType
    privy: Optional[UserPrivyData] = None
    type: Literal["wallet"] = "wallet"


# Stable Primary Login Identity variants returned in every Account profile.
UserIdentity = Union[EmailIdentity, GoogleIdentity, XIdentity, WalletIdentity]


@dataclass
class EmailLinkedIdentity:
    """Connected Email Identity without repeated Privy wallet data."""

    account: EmailAccountData
    type: Literal["email"] = "email"


@dataclass
class GoogleLinkedIdentity:
    """Connected Google Identity without repeated Privy wallet data."""

    account: GoogleAccountData
    type: Literal["google"] = "google"


@dataclass
class XLinkedIdentity:
    """Connected X Identity without repeated Privy wallet data."""

    account: XAccountData
    type: Literal["x"] = "x"


@dataclass
class WalletLinkedIdentity:
    """Connected Wallet Identity identified by canonical address and chain."""

    address: str
    chain: ChainType
    type: Literal["wallet"] = "wallet"


# Connected Login Identity without repeated Account-level Privy wallet data.
LinkedIdentity = Union[
    EmailLinkedIdentity,
    GoogleLinkedIdentity,
    XLinkedIdentity,
    WalletLinkedIdentity,
]


@dataclass
class LinkedIdentitySelector:
    """Verified method that initiated one interactive Privy authentication.

    Each variant carries only its canonical identifier fields. Invalid mixed or
    incomplete shapes fail during construction rather than at the backend.
    """

    type: LinkedIdentityType
    email: Optional[str] = None
    username: Optional[str] = None
    address: Optional[str] = None
    chain: Optional[ChainType] = None

    def __post_init__(self) -> None:
        """Reject fields that do not belong to the selected login method."""
        fields = {
            "email": self.email,
            "username": self.username,
            "address": self.address,
            "chain": self.chain,
        }
        required = {
            "email": {"email"},
            "google": {"email"},
            "x": {"username"},
            "wallet": {"address", "chain"},
        }[self.type]
        present = {name for name, value in fields.items() if value is not None}
        if present != required or any(not fields[name] for name in required):
            expected = " and ".join(sorted(required))
            raise ValueError(f"{self.type} selector requires exactly {expected}")

    def to_dict(self) -> dict:
        """Serialize the validated tagged selector for register-or-sync."""
        return {key: value for key, value in vars(self).items() if value is not None}


@dataclass
class RegisterPrivyRequest:
    """Register-or-sync request naming the verified attempted identity."""

    attempted_identity: LinkedIdentitySelector

    def to_dict(self) -> dict:
        """Serialize the request using the backend's attempted-identity field."""
        return {"attempted_identity": self.attempted_identity.to_dict()}


# Stable ownership rejection codes emitted by register-or-sync.
RegisterPrivyConflictCode = Literal[
    "IDENTITY_OWNED_BY_ANOTHER_ACCOUNT",
    "IDENTITIES_OWNED_BY_MULTIPLE_ACCOUNTS",
    "WALLET_OWNED_BY_ANOTHER_ACCOUNT",
]


@dataclass(frozen=True)
class RegisterPrivyConflict:
    """Bounded ownership conflict safe for client recovery guidance."""

    code: RegisterPrivyConflictCode
    existing_method: Optional[LinkedIdentityType] = None


def classify_register_privy_conflict(
    error: BaseException,
) -> Optional[RegisterPrivyConflict]:
    """Classify only stable register-or-sync ownership rejection codes."""
    from ..error import ApiRejected

    if not isinstance(error, ApiRejected):
        return None
    code = error.details.error_code
    if code not in (
        "IDENTITY_OWNED_BY_ANOTHER_ACCOUNT",
        "IDENTITIES_OWNED_BY_MULTIPLE_ACCOUNTS",
        "WALLET_OWNED_BY_ANOTHER_ACCOUNT",
    ):
        return None
    return RegisterPrivyConflict(
        code=cast(RegisterPrivyConflictCode, code),
        existing_method=error.details.existing_method,
    )


def identity_text(identity: UserIdentity) -> str:
    """Human-readable login-method label ("Google" / "X" / "Solana")."""
    if isinstance(identity, GoogleIdentity):
        return "Google"
    if isinstance(identity, XIdentity):
        return "X"
    if isinstance(identity, EmailIdentity):
        return "Email"
    return "Solana"


def _email_display_name(email: str) -> str:
    """Keep Email labels compact while preserving recognizable address ends."""
    max_chars = 20
    if len(email) <= max_chars:
        return email
    visible_chars = max_chars - 3
    prefix_chars = visible_chars // 2
    return f"{email[:prefix_chars]}...{email[-(visible_chars - prefix_chars) :]}"


@dataclass
class User:
    """Full user profile — the `user` object of `SessionResponse`."""

    user_id: str
    identity: UserIdentity
    max_slippage_preference: Optional[str]
    """Remembered percentage below 10%; None until one is stored."""
    linked_identities: list[LinkedIdentity] = field(default_factory=list)
    """Every connected login identity, including the primary identity."""
    connected_x: Optional[XAccountData] = None
    """X account connected by a non-X-identity user; None when identity is X."""

    def privy(self) -> Optional[UserPrivyData]:
        """Privy account data, regardless of identity type."""
        return self.identity.privy

    def x_account(self) -> Optional[XAccountData]:
        """The X account, whether it is the login identity or a connected account."""
        if isinstance(self.identity, XIdentity):
            return self.identity.account
        return self.connected_x

    def trading_wallet(self, auth_method: AuthMethod) -> str:
        """The wallet this session operates as.

        Google/X identities only exist via Privy registration, which always
        provisions an embedded wallet — that wallet is the answer regardless
        of auth method. Wallet identities depend on the session: a Privy
        (SIWS) session trades via the embedded wallet, a Lightcone session
        trades via the wallet that signed in.
        """
        if isinstance(self.identity, (EmailIdentity, GoogleIdentity, XIdentity)):
            return self.identity.privy.wallet.address
        if auth_method == "privy" and self.identity.privy is not None:
            return self.identity.privy.wallet.address
        return self.identity.address

    def wallet_display_name(self, auth_method: AuthMethod) -> str:
        """Short display label for the wallet this session operates as."""
        return shorten(self.trading_wallet(auth_method), 8)

    def display_name(self) -> str:
        """Best display name. Email labels are limited to 20 characters;
        Google uses name -> email fallback; X uses display_name -> username;
        wallet identities show the shortened address ("FRGk...WcPR")."""
        if isinstance(self.identity, GoogleIdentity):
            return self.identity.account.name or self.identity.account.email
        if isinstance(self.identity, XIdentity):
            return self.identity.account.display_name or self.identity.account.username
        if isinstance(self.identity, EmailIdentity):
            return _email_display_name(self.identity.account.email)
        return shorten(self.identity.address, 8)

    def avatar_url(self) -> Optional[str]:
        """Avatar URL from the login identity's OAuth provider, if any."""
        if isinstance(self.identity, (GoogleIdentity, XIdentity)):
            return self.identity.account.avatar_url
        return None


@dataclass
class SessionResponse:
    """Session envelope returned by login, register-privy, and GET /api/auth/me.

    There is no `wallet_address` field — derive the session's trading wallet
    with `session.user.trading_wallet(session.auth_method)`.
    """

    user: User
    expires_at: int
    auth_method: AuthMethod
    is_beta: bool


@dataclass
class AuthCredentials:
    """Internal auth session state. Token is NOT exposed."""

    user_id: str
    wallet_address: str
    expires_at: int

    def is_authenticated(self) -> bool:
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
    "ChainType",
    "AuthMethod",
    "LinkedIdentityType",
    "PrivyEmbeddedWallet",
    "UserPrivyData",
    "XAccountData",
    "GoogleAccountData",
    "EmailAccountData",
    "EmailIdentity",
    "GoogleIdentity",
    "XIdentity",
    "WalletIdentity",
    "UserIdentity",
    "EmailLinkedIdentity",
    "GoogleLinkedIdentity",
    "XLinkedIdentity",
    "WalletLinkedIdentity",
    "LinkedIdentity",
    "LinkedIdentitySelector",
    "RegisterPrivyRequest",
    "RegisterPrivyConflictCode",
    "RegisterPrivyConflict",
    "classify_register_privy_conflict",
    "identity_text",
    "User",
    "SessionResponse",
    "AuthCredentials",
    "LoginRequest",
    "NonceResponse",
    "generate_signin_message",
]
