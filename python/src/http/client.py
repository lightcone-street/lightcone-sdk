"""HTTP client for the Lightcone SDK.

Matches TS http/client.ts with retry logic and cookie-based auth.
"""

import asyncio
import json
from typing import Any, Optional
from urllib.parse import quote as url_quote

import aiohttp

from ..error import HttpError
from ..network import DEFAULT_API_URL
from .retry import RetryConfig, RetryPolicy, delay_for_attempt, is_retryable_status


DEFAULT_TIMEOUT_SECS = 30


class LightconeHttp:
    """HTTP client with retry, auth, and error mapping.

    Auth token is sent via Cookie header matching the TS implementation.
    """

    def __init__(
        self,
        base_url: str = DEFAULT_API_URL,
        auth_token: Optional[str] = None,
        retry_config: Optional[RetryConfig] = None,
        timeout: int = DEFAULT_TIMEOUT_SECS,
    ):
        self._base_url = base_url.rstrip("/")
        self._auth_token: Optional[str] = auth_token
        self._retry_config = retry_config or RetryConfig.default()
        self._timeout = aiohttp.ClientTimeout(total=timeout)
        self._session: Optional[aiohttp.ClientSession] = None

    @property
    def base_url(self) -> str:
        return self._base_url

    def set_auth_token(self, token: Optional[str]) -> None:
        """Set or clear the auth token."""
        self._auth_token = token

    def has_auth_token(self) -> bool:
        return self._auth_token is not None

    async def _ensure_session(self) -> aiohttp.ClientSession:
        if self._session is None or self._session.closed:
            self._session = aiohttp.ClientSession(
                timeout=self._timeout,
                headers={
                    "Content-Type": "application/json",
                    "Accept": "application/json",
                },
            )
        return self._session

    async def close(self) -> None:
        """Close the HTTP session."""
        if self._session and not self._session.closed:
            await self._session.close()
            self._session = None

    async def __aenter__(self) -> "LightconeHttp":
        await self._ensure_session()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        await self.close()

    def _get_headers(self) -> dict[str, str]:
        """Build headers with auth cookie if present."""
        headers: dict[str, str] = {}
        if self._auth_token:
            headers["Cookie"] = f"auth_token={self._auth_token}"
        return headers

    def _map_status_error(self, status: int, message: str) -> HttpError:
        """Map HTTP status to HttpError variant."""
        if status == 401:
            return HttpError.unauthorized(message)
        elif status == 400:
            return HttpError.bad_request(message)
        elif status == 404:
            return HttpError.not_found(message)
        elif status == 429:
            return HttpError.rate_limited(message)
        elif status >= 500:
            return HttpError.server_error(message, status)
        else:
            return HttpError.request(f"HTTP {status}: {message}")

    async def _handle_response(self, response: aiohttp.ClientResponse) -> dict:
        """Parse response JSON or raise HttpError."""
        if 200 <= response.status < 300:
            try:
                return await response.json()
            except (ValueError, json.JSONDecodeError, aiohttp.ContentTypeError) as e:
                raise HttpError.request(f"Failed to parse response: {e}")
        else:
            error_text = await response.text()
            try:
                error_data = json.loads(error_text)
                error_msg = (
                    error_data.get("error")
                    or error_data.get("message")
                    or error_text
                )
            except (ValueError, json.JSONDecodeError):
                error_msg = error_text or "Unknown error"

            raise self._map_status_error(response.status, error_msg)

    async def _request_with_retry(
        self,
        method: str,
        path: str,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
        **kwargs: Any,
    ) -> dict:
        """Make an HTTP request with retry logic."""
        session = await self._ensure_session()
        url = f"{self._base_url}{path}"
        headers = self._get_headers()
        if "headers" in kwargs:
            headers.update(kwargs.pop("headers"))

        config = self._retry_config if retry_policy != RetryPolicy.NONE else RetryConfig.none()

        last_error: Optional[Exception] = None

        for attempt in range(config.max_retries + 1):
            try:
                async with session.request(
                    method, url, headers=headers, **kwargs
                ) as response:
                    return await self._handle_response(response)
            except HttpError as e:
                last_error = e
                if (
                    e.status is not None
                    and is_retryable_status(e.status, config)
                    and attempt < config.max_retries
                ):
                    delay = delay_for_attempt(attempt, config)
                    await asyncio.sleep(delay)
                    continue
                raise
            except asyncio.TimeoutError:
                last_error = HttpError.timeout()
                if attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    await asyncio.sleep(delay)
                    continue
                raise last_error
            except aiohttp.ClientError as e:
                last_error = HttpError.request(str(e))
                if attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    await asyncio.sleep(delay)
                    continue
                raise last_error

        raise last_error or HttpError.max_retries_exceeded(config.max_retries)

    async def get(
        self,
        path: str,
        params: Optional[dict] = None,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
    ) -> dict:
        """Make a GET request.

        Args:
            path: URL path (appended to base_url)
            params: Optional query parameters
            retry_policy: Retry policy to use

        Returns:
            Parsed JSON response
        """
        return await self._request_with_retry(
            "GET", path, retry_policy=retry_policy, params=params
        )

    async def post(
        self,
        path: str,
        body: Optional[Any] = None,
        retry_policy: RetryPolicy = RetryPolicy.NONE,
    ) -> dict:
        """Make a POST request.

        Args:
            path: URL path (appended to base_url)
            body: Optional JSON body
            retry_policy: Retry policy to use

        Returns:
            Parsed JSON response
        """
        return await self._request_with_retry(
            "POST", path, retry_policy=retry_policy, json=body
        )


__all__ = ["LightconeHttp"]
