"""HTTP client for the Lightcone SDK."""

import asyncio
import json
import logging
from typing import Any, Optional

import aiohttp

from ..error import HttpError, HttpErrorKind
from ..network import DEFAULT_API_URL
from .retry import RetryConfig, RetryPolicy, delay_for_attempt

logger = logging.getLogger(__name__)

DEFAULT_TIMEOUT_SECS = 30


class LightconeHttp:
    """HTTP client with retry, auth, and error mapping.

    Auth token is sent via Cookie header.
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
        self._default_retry_config = retry_config
        self._timeout = aiohttp.ClientTimeout(total=timeout)
        self._session: Optional[aiohttp.ClientSession] = None

    @property
    def base_url(self) -> str:
        return self._base_url

    @property
    def auth_token(self) -> Optional[str]:
        """Public accessor for the auth token."""
        return self._auth_token

    def set_auth_token(self, token: Optional[str]) -> None:
        """Set or clear the auth token."""
        self._auth_token = token

    def clear_auth_token(self) -> None:
        """Clear the auth token."""
        self._auth_token = None

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

    def _resolve_retry_config(self, policy: RetryPolicy) -> Optional[RetryConfig]:
        """Resolve a retry policy to a RetryConfig, or None for no retries."""
        if policy.is_none():
            return None
        if policy._kind == "custom":
            return policy._config
        # Idempotent: use client's override if provided, otherwise standard idempotent
        return self._default_retry_config or RetryConfig.idempotent()

    def _map_status_error(self, status: int, message: str) -> HttpError:
        """Map HTTP status to HttpError variant."""
        if status == 401:
            return HttpError.unauthorized(message)
        elif status == 404:
            return HttpError.not_found(message)
        elif status == 429:
            return HttpError.rate_limited(message)
        elif 400 <= status <= 499:
            return HttpError.bad_request(message, status)
        else:
            return HttpError.server_error(message, status)

    async def _do_request(self, method: str, path: str, **kwargs: Any) -> Any:
        """Execute a single HTTP request (no retry)."""
        session = await self._ensure_session()
        url = f"{self._base_url}{path}"
        headers = self._get_headers()
        if "headers" in kwargs:
            headers.update(kwargs.pop("headers"))

        async with session.request(method, url, headers=headers, **kwargs) as response:
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
    ) -> Any:
        """Make an HTTP request with retry logic."""
        config = self._resolve_retry_config(retry_policy)

        if config is None:
            # RetryPolicy.NONE — single attempt, no retry loop
            return await self._do_request(method, path, **kwargs)

        last_error: Optional[Exception] = None

        for attempt in range(config.max_retries + 1):
            try:
                return await self._do_request(method, path, **kwargs)
            except HttpError as e:
                should_retry = False

                if e.kind == HttpErrorKind.SERVER_ERROR:
                    should_retry = (
                        e.status is not None
                        and e.status in config.retryable_statuses
                    )
                elif e.kind == HttpErrorKind.RATE_LIMITED:
                    # Always retry rate-limited, honor retry_after_ms
                    if e.retry_after_ms:
                        await asyncio.sleep(e.retry_after_ms / 1000.0)
                    should_retry = True
                elif e.kind == HttpErrorKind.TIMEOUT:
                    should_retry = True

                if should_retry and attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    logger.debug(
                        "Retrying request to %s%s (attempt %d/%d, delay %.1fs)",
                        self._base_url, path, attempt + 1, config.max_retries, delay,
                    )
                    await asyncio.sleep(delay)
                    last_error = e
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
                # Transport errors (connect, DNS, etc.) — always retryable
                last_error = HttpError.request(str(e))
                if attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    await asyncio.sleep(delay)
                    continue
                raise last_error

        raise HttpError.max_retries_exceeded(
            config.max_retries + 1,
            str(last_error) if last_error else "unknown",
        )

    async def get(
        self,
        path: str,
        params: Optional[dict] = None,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
    ) -> Any:
        """Make a GET request.

        Args:
            path: URL path (appended to base_url)
            params: Optional query parameters
            retry_policy: Retry policy (default: IDEMPOTENT with retries)
        """
        return await self._request_with_retry(
            "GET", path, retry_policy=retry_policy, params=params
        )

    async def post(
        self,
        path: str,
        body: Optional[Any] = None,
        retry_policy: RetryPolicy = RetryPolicy.NONE,
    ) -> Any:
        """Make a POST request.

        Args:
            path: URL path (appended to base_url)
            body: Optional JSON body
            retry_policy: Retry policy (default: NONE for non-idempotent)
        """
        return await self._request_with_retry(
            "POST", path, retry_policy=retry_policy, json=body
        )


__all__ = ["LightconeHttp"]
