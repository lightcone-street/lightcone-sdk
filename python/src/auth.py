"""Authentication module for Lightcone.

Provides functionality for authenticating with the Lightcone API.
Used by both the REST API client and WebSocket client.

Authentication Flow:
    1. Generate a sign-in message with timestamp
    2. Sign the message with an Ed25519 keypair
    3. POST to the authentication endpoint
    4. Extract token from JSON response
"""

import asyncio
import time
from dataclasses import dataclass
from typing import Optional

import aiohttp
import base58
from nacl.signing import SigningKey
from solders.keypair import Keypair


# Authentication API base URL
AUTH_API_URL = "https://tapi.lightcone.xyz/api"

# Authentication request timeout in seconds
AUTH_TIMEOUT_SECS = 10


class AuthError(Exception):
    """Error during authentication.

    Mirrors Rust's AuthError enum with variants:
    - SystemTime: System time error
    - HttpError: HTTP request failed
    - AuthenticationFailed: Server rejected authentication
    """

    def __init__(self, message: str, variant: str = "AuthenticationFailed"):
        super().__init__(message)
        self.variant = variant

    @staticmethod
    def system_time(message: str) -> "AuthError":
        return AuthError(f"System time error: {message}", "SystemTime")

    @staticmethod
    def http_error(message: str) -> "AuthError":
        return AuthError(f"HTTP error: {message}", "HttpError")

    @staticmethod
    def authentication_failed(message: str) -> "AuthError":
        return AuthError(f"Authentication failed: {message}", "AuthenticationFailed")


@dataclass
class AuthCredentials:
    """Authentication credentials returned after successful login."""

    auth_token: str
    """The authentication token."""

    user_pubkey: str
    """The user's public key (Base58 encoded)."""

    user_id: str
    """The user's ID."""

    expires_at: int
    """Token expiration timestamp (Unix seconds)."""


def generate_signin_message() -> str:
    """Generate the sign-in message with current timestamp.

    Returns:
        The message to be signed.

    Raises:
        AuthError: If system time is before UNIX epoch.
    """
    timestamp_ms = int(time.time() * 1000)
    if timestamp_ms < 0:
        raise AuthError.system_time("System time before UNIX epoch")
    return f"Sign in to Lightcone\n\nTimestamp: {timestamp_ms}"


def generate_signin_message_with_timestamp(timestamp_ms: int) -> str:
    """Generate the sign-in message with a specific timestamp.

    Args:
        timestamp_ms: Unix timestamp in milliseconds.

    Returns:
        The message to be signed.
    """
    return f"Sign in to Lightcone\n\nTimestamp: {timestamp_ms}"


async def authenticate(keypair: Keypair, base_url: str = AUTH_API_URL) -> AuthCredentials:
    """Authenticate with Lightcone and obtain credentials.

    Args:
        keypair: The Solana Keypair for authentication.
        base_url: The base URL for the auth API. Defaults to production.

    Returns:
        AuthCredentials containing the auth token and user public key.

    Raises:
        AuthError: If authentication fails or times out.

    Example:
        ```python
        from solders.keypair import Keypair
        from lightcone_sdk.auth import authenticate

        keypair = Keypair()
        credentials = await authenticate(keypair)
        print(f"Auth token: {credentials.auth_token}")
        ```
    """
    # Generate the message
    message = generate_signin_message()

    # Sign the message using nacl (extract seed from keypair)
    message_bytes = message.encode("utf-8")
    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]
    signing_key = SigningKey(seed)
    signed = signing_key.sign(message_bytes)
    signature_b58 = base58.b58encode(signed.signature).decode("utf-8")

    # Get the public key
    pubkey = str(keypair.pubkey())
    pubkey_bytes = list(bytes(keypair.pubkey()))

    # Create the request body (matches Rust LoginRequest)
    request_data = {
        "pubkey_bytes": pubkey_bytes,
        "message": message,
        "signature_bs58": signature_b58,
    }

    # Send the authentication request with timeout
    url = f"{base_url}/auth/login_or_register_with_message"
    timeout = aiohttp.ClientTimeout(total=AUTH_TIMEOUT_SECS)

    try:
        async with aiohttp.ClientSession(timeout=timeout) as session:
            async with session.post(
                url,
                json=request_data,
                headers={"Content-Type": "application/json"},
            ) as response:
                if not response.ok:
                    raise AuthError.http_error(f"HTTP {response.status}")

                response_data = await response.json()

                return AuthCredentials(
                    auth_token=response_data["token"],
                    user_pubkey=pubkey,
                    user_id=response_data["user_id"],
                    expires_at=response_data["expires_at"],
                )

    except asyncio.TimeoutError:
        raise AuthError.authentication_failed("Request timed out")
    except aiohttp.ClientError as e:
        raise AuthError.http_error(str(e))


def sign_message(message: str, keypair: Keypair) -> str:
    """Sign a message with a Solana Keypair.

    Args:
        message: The message to sign.
        keypair: The Solana Keypair.

    Returns:
        The Base58-encoded signature.
    """
    message_bytes = message.encode("utf-8")
    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]
    signing_key = SigningKey(seed)
    signed = signing_key.sign(message_bytes)
    return base58.b58encode(signed.signature).decode("utf-8")
