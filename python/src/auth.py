"""Standalone authentication module for the Lightcone API."""

import time
from dataclasses import dataclass
from typing import Optional

import aiohttp
from nacl.signing import SigningKey
from solders.keypair import Keypair

AUTH_API_URL = "https://tapi.lightcone.xyz/api"


class AuthError(Exception):
    """Error during authentication."""

    pass


@dataclass
class AuthCredentials:
    """Authentication credentials returned from login."""

    auth_token: str
    user_pubkey: str
    user_id: Optional[str]
    expires_at: Optional[int]


def generate_signin_message_with_timestamp(timestamp: int) -> str:
    """Generate a sign-in message with a specific timestamp.

    Args:
        timestamp: Unix timestamp to include in the message

    Returns:
        The sign-in message string
    """
    return f"Sign in to Lightcone: {timestamp}"


def generate_signin_message() -> str:
    """Generate a sign-in message with the current timestamp.

    Returns:
        The sign-in message string
    """
    return generate_signin_message_with_timestamp(int(time.time()))


async def authenticate(
    keypair: Keypair,
    api_url: str = AUTH_API_URL,
) -> AuthCredentials:
    """Authenticate with the Lightcone API using an Ed25519 keypair.

    Signs a timestamped message and exchanges it for an auth token.

    Args:
        keypair: The Solana keypair to authenticate with
        api_url: Base API URL (defaults to AUTH_API_URL)

    Returns:
        AuthCredentials with the auth token and user info

    Raises:
        AuthError: If authentication fails
    """
    timestamp = int(time.time())
    message = generate_signin_message_with_timestamp(timestamp)
    message_bytes = message.encode("utf-8")

    # Sign the message
    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]
    signing_key = SigningKey(seed)
    signed = signing_key.sign(message_bytes)
    signature_hex = signed.signature.hex()

    pubkey = str(keypair.pubkey())

    # POST to auth endpoint
    url = f"{api_url}/auth/login_or_register_with_message"
    payload = {
        "pubkey": pubkey,
        "message": message,
        "signature": signature_hex,
    }

    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(url, json=payload) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise AuthError(
                        f"Authentication failed (HTTP {response.status}): {error_text}"
                    )

                data = await response.json()

                # Extract auth token from response or cookies
                auth_token = data.get("token") or data.get("auth_token", "")
                if not auth_token:
                    # Try to get from set-cookie header
                    cookies = response.cookies
                    if "auth_token" in cookies:
                        auth_token = cookies["auth_token"].value

                if not auth_token:
                    raise AuthError("No auth token in response")

                return AuthCredentials(
                    auth_token=auth_token,
                    user_pubkey=pubkey,
                    user_id=data.get("user_id"),
                    expires_at=data.get("expires_at"),
                )
    except aiohttp.ClientError as e:
        raise AuthError(f"Authentication request failed: {e}")
