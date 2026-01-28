"""Authentication module for Lightcone WebSocket.

Provides functionality for authenticating with the Lightcone API
to access private user streams (orders, balances, fills).

Authentication Flow:
    1. Generate a sign-in message with timestamp
    2. Sign the message with an Ed25519 keypair
    3. POST to the authentication endpoint
    4. Extract token from JSON response
    5. Connect to WebSocket with the auth token
"""

import asyncio
import time
from dataclasses import dataclass

import aiohttp
from nacl.signing import SigningKey
import base58

from .error import WebSocketError


# Authentication API base URL
AUTH_API_URL = "https://tapi.lightcone.xyz/api"

# Authentication request timeout in seconds
AUTH_TIMEOUT_SECS = 10


@dataclass
class AuthCredentials:
    """Authentication credentials returned after successful login."""

    auth_token: str
    """The authentication token to use for WebSocket connection."""

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
        WebSocketError: If system time is before UNIX epoch.
    """
    timestamp_ms = int(time.time() * 1000)
    if timestamp_ms < 0:
        raise WebSocketError("System time before UNIX epoch")
    return f"Sign in to Lightcone\n\nTimestamp: {timestamp_ms}"


def generate_signin_message_with_timestamp(timestamp_ms: int) -> str:
    """Generate the sign-in message with a specific timestamp.

    Args:
        timestamp_ms: Unix timestamp in milliseconds.

    Returns:
        The message to be signed.
    """
    return f"Sign in to Lightcone\n\nTimestamp: {timestamp_ms}"


async def authenticate(signing_key: SigningKey) -> AuthCredentials:
    """Authenticate with Lightcone and obtain credentials.

    Args:
        signing_key: The Ed25519 signing key for authentication.

    Returns:
        AuthCredentials containing the auth token and user public key.

    Raises:
        WebSocketError: If authentication fails or times out.

    Example:
        ```python
        from nacl.signing import SigningKey
        from src.websocket.auth import authenticate

        signing_key = SigningKey(secret_key_bytes)
        credentials = await authenticate(signing_key)
        print(f"Auth token: {credentials.auth_token}")
        ```
    """
    # Generate the message
    message = generate_signin_message()

    # Sign the message
    message_bytes = message.encode("utf-8")
    signed = signing_key.sign(message_bytes)
    signature = signed.signature
    signature_b58 = base58.b58encode(signature).decode("utf-8")

    # Get the public key
    verify_key = signing_key.verify_key
    public_key_b58 = base58.b58encode(bytes(verify_key)).decode("utf-8")

    # Create the request body
    request_data = {
        "pubkey_bytes": list(bytes(verify_key)),
        "message": message,
        "signature_bs58": signature_b58,
    }

    # Send the authentication request with timeout
    url = f"{AUTH_API_URL}/auth/login_or_register_with_message"
    timeout = aiohttp.ClientTimeout(total=AUTH_TIMEOUT_SECS)

    try:
        async with aiohttp.ClientSession(timeout=timeout) as session:
            async with session.post(
                url,
                json=request_data,
                headers={"Content-Type": "application/json"},
            ) as response:
                # Check for HTTP errors
                if not response.ok:
                    raise WebSocketError(
                        f"Authentication failed: HTTP error {response.status}"
                    )

                # Parse the response body
                response_data = await response.json()

                return AuthCredentials(
                    auth_token=response_data["token"],
                    user_pubkey=public_key_b58,
                    user_id=response_data["user_id"],
                    expires_at=response_data["expires_at"],
                )

    except asyncio.TimeoutError:
        raise WebSocketError("Authentication failed: Request timed out")
    except aiohttp.ClientError as e:
        raise WebSocketError(f"Authentication failed: {str(e)}")


async def authenticate_with_secret_key(secret_key: bytes) -> AuthCredentials:
    """Authenticate with Lightcone using a secret key.

    Args:
        secret_key: The Ed25519 secret key (32 bytes).

    Returns:
        AuthCredentials containing the auth token and user public key.

    Example:
        ```python
        from src.websocket.auth import authenticate_with_secret_key

        secret_key = bytes.fromhex("...")  # Your 32-byte secret key
        credentials = await authenticate_with_secret_key(secret_key)
        ```
    """
    signing_key = SigningKey(secret_key)
    return await authenticate(signing_key)


def sign_message(message: str, signing_key: SigningKey) -> str:
    """Sign a message with an Ed25519 signing key.

    Args:
        message: The message to sign.
        signing_key: The Ed25519 signing key.

    Returns:
        The Base58-encoded signature.
    """
    message_bytes = message.encode("utf-8")
    signed = signing_key.sign(message_bytes)
    return base58.b58encode(signed.signature).decode("utf-8")
