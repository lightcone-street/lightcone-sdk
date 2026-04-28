"""HTTP client for the Lightcone SDK.

``get()`` and ``post()`` return the unwrapped API body directly. They handle:
- ``x-request-id`` generation and header injection
- auth/admin cookie injection
- deserialization of the ``ApiResponse`` wrapper
- conversion of backend rejections into ``ApiRejected``

``raw_post()`` bypasses ApiResponse handling for non-API calls such as Solana
JSON-RPC.
"""

from __future__ import annotations

import asyncio
import json
import logging
import uuid
from enum import Enum
from typing import Any, Optional

import aiohttp

from ..error import ApiRejected, HttpError, HttpErrorKind
from ..shared.api_response import ApiResponse, ApiRejectedDetails
from .retry import RetryPolicy, delay_for_attempt

logger = logging.getLogger(__name__)

DEFAULT_TIMEOUT_SECS = 30


class _AuthMode(str, Enum):
    COOKIE = "cookie"
    COOKIE_OVERRIDE = "cookie_override"
    ADMIN_COOKIE = "admin_cookie"


class LightconeHttp:
    """HTTP client with retry, auth, and ApiResponse unwrapping."""

    def __init__(
        self,
        base_url: str,
        timeout: int = DEFAULT_TIMEOUT_SECS,
    ):
        self._base_url = base_url.rstrip("/")
        self._auth_token: Optional[str] = None
        self._admin_token: Optional[str] = None
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

    @property
    def admin_token(self) -> Optional[str]:
        """Public accessor for the admin token."""
        return self._admin_token

    def set_admin_token(self, token: Optional[str]) -> None:
        """Set or clear the admin token."""
        self._admin_token = token

    def clear_admin_token(self) -> None:
        """Clear the admin token."""
        self._admin_token = None

    def has_admin_token(self) -> bool:
        return self._admin_token is not None

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

    async def raw_post(self, url: str, body: Any) -> Any:
        """POST an arbitrary JSON body without ApiResponse parsing."""
        session = await self._ensure_session()
        async with session.post(url, json=body) as response:
            if 200 <= response.status < 300:
                try:
                    return await response.json()
                except (
                    ValueError,
                    json.JSONDecodeError,
                    aiohttp.ContentTypeError,
                ) as error:
                    raise HttpError.request(
                        f"Failed to parse response: {error}"
                    ) from error

            body_text = await response.text()
            raise self._map_status_error(
                response.status, body_text or "", response.headers
            )

    async def get(
        self,
        path: str,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
    ) -> Any:
        """Make a GET request with user auth cookie injection."""
        return await self._request_with_retry(
            "GET",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE,
        )

    async def get_with_auth(
        self,
        path: str,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
        *,
        auth_token: str,
    ) -> Any:
        """Make a GET request with an explicit per-call ``auth_token`` cookie.

        Intended for server-side cookie forwarding (SSR / server functions)
        where the per-request browser cookie can't propagate to the SDK's
        process-wide cookie store. Bypasses both the stored ``auth_token``
        and the response-side ``Set-Cookie`` capture so per-call overrides
        never mutate shared state.
        """
        return await self._request_with_retry(
            "GET",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE_OVERRIDE,
            auth_token_override=auth_token,
        )

    async def post(
        self,
        path: str,
        body: Any,
        retry_policy: RetryPolicy = RetryPolicy.NONE,
    ) -> Any:
        """Make a POST request with user auth cookie injection."""
        return await self._request_with_retry(
            "POST",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE,
            json=body,
        )

    async def admin_post(
        self,
        path: str,
        body: Any,
        retry_policy: RetryPolicy = RetryPolicy.NONE,
    ) -> Any:
        """Make a POST request with admin cookie injection."""
        return await self._request_with_retry(
            "POST",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.ADMIN_COOKIE,
            json=body,
        )

    async def admin_get(
        self,
        path: str,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
    ) -> Any:
        """Make a GET request with admin cookie injection."""
        return await self._request_with_retry(
            "GET",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.ADMIN_COOKIE,
        )

    async def _request_with_retry(
        self,
        method: str,
        path: str,
        *,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
        auth_mode: _AuthMode,
        auth_token_override: Optional[str] = None,
        **kwargs: Any,
    ) -> Any:
        """Make an HTTP request with retry logic and ApiResponse unwrapping."""
        config = retry_policy.resolve_config()

        if config is None:
            return await self._send_and_parse(
                method,
                path,
                auth_mode=auth_mode,
                auth_token_override=auth_token_override,
                **kwargs,
            )

        last_error: Optional[Exception] = None

        for attempt in range(config.max_retries + 1):
            try:
                return await self._send_and_parse(
                    method,
                    path,
                    auth_mode=auth_mode,
                    auth_token_override=auth_token_override,
                    **kwargs,
                )
            except ApiRejected:
                raise
            except HttpError as error:
                should_retry = False

                if error.kind == HttpErrorKind.SERVER_ERROR:
                    should_retry = (
                        error.status is not None
                        and error.status in config.retryable_statuses
                    )
                elif error.kind == HttpErrorKind.RATE_LIMITED:
                    if error.retry_after_ms:
                        await asyncio.sleep(error.retry_after_ms / 1000.0)
                    should_retry = True
                elif error.kind == HttpErrorKind.TIMEOUT:
                    should_retry = True

                if should_retry and attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    logger.debug(
                        "Retrying request to %s (attempt %d/%d, delay %.1fs)",
                        self._resolve_url(path),
                        attempt + 1,
                        config.max_retries,
                        delay,
                    )
                    await asyncio.sleep(delay)
                    last_error = error
                    continue
                raise
            except asyncio.TimeoutError:
                last_error = HttpError.timeout()
                if attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    await asyncio.sleep(delay)
                    continue
                raise last_error
            except aiohttp.ClientError as error:
                retryable = isinstance(
                    error, aiohttp.ClientConnectorError
                ) and not isinstance(error, aiohttp.ClientSSLError)
                if retryable and attempt < config.max_retries:
                    last_error = HttpError.request(str(error))
                    delay = delay_for_attempt(attempt, config)
                    await asyncio.sleep(delay)
                    continue
                raise HttpError.request(str(error)) from error

        raise HttpError.max_retries_exceeded(
            config.max_retries + 1,
            str(last_error) if last_error else "unknown",
        )

    async def _send_and_parse(
        self,
        method: str,
        path: str,
        *,
        auth_mode: _AuthMode,
        auth_token_override: Optional[str] = None,
        **kwargs: Any,
    ) -> Any:
        payload, request_id = await self._send_request(
            method,
            path,
            auth_mode=auth_mode,
            auth_token_override=auth_token_override,
            **kwargs,
        )
        return self._parse_api_response(payload, request_id)

    @staticmethod
    def _parse_api_response(payload: Any, request_id: str) -> Any:
        """Unwrap an API response or raise ApiRejected with the request id."""
        if not isinstance(payload, dict) or payload.get("status") not in {
            "success",
            "error",
        }:
            return payload

        parsed = ApiResponse.from_dict(payload)
        if parsed.status == "success":
            return parsed.body

        details = parsed.details or ApiRejectedDetails(reason="Unknown API rejection")
        raise ApiRejected(details.with_request_id(request_id))

    async def _send_request(
        self,
        method: str,
        path: str,
        *,
        auth_mode: _AuthMode,
        auth_token_override: Optional[str] = None,
        **kwargs: Any,
    ) -> tuple[Any, str]:
        """Send one request and return the raw decoded JSON payload plus request id."""
        session = await self._ensure_session()
        request_id = str(uuid.uuid4())
        headers = dict(kwargs.pop("headers", {}))
        headers["x-request-id"] = request_id
        headers.update(self._auth_headers(auth_mode, auth_token_override))

        async with session.request(
            method,
            self._resolve_url(path),
            headers=headers,
            **kwargs,
        ) as response:
            if 200 <= response.status < 300:
                # Per-call overrides must not mutate the shared cookie store —
                # response Set-Cookie headers from a forwarded auth_token would
                # otherwise leak into the SDK's process-wide token slot.
                if auth_mode is not _AuthMode.COOKIE_OVERRIDE:
                    self._capture_cookies(response.headers)
                try:
                    return await response.json(), request_id
                except (
                    ValueError,
                    json.JSONDecodeError,
                    aiohttp.ContentTypeError,
                ) as error:
                    raise HttpError.request(
                        f"Failed to parse response: {error}"
                    ) from error

            body_text = await response.text()
            raise self._map_status_error(
                response.status, body_text or "", response.headers
            )

    def _resolve_url(self, path: str) -> str:
        if path.startswith("http://") or path.startswith("https://"):
            return path
        return f"{self._base_url}{path}"

    def _auth_headers(
        self,
        auth_mode: _AuthMode,
        auth_token_override: Optional[str] = None,
    ) -> dict[str, str]:
        headers: dict[str, str] = {}
        if auth_mode == _AuthMode.COOKIE_OVERRIDE:
            if auth_token_override:
                headers["Cookie"] = f"auth_token={auth_token_override}"
        elif auth_mode == _AuthMode.COOKIE and self._auth_token:
            headers["Cookie"] = f"auth_token={self._auth_token}"
        elif auth_mode == _AuthMode.ADMIN_COOKIE and self._admin_token:
            headers["Cookie"] = f"admin_token={self._admin_token}"
        return headers

    def _capture_cookies(self, headers: aiohttp.typedefs.LooseHeaders) -> None:
        set_cookie_headers = []
        if hasattr(headers, "getall"):
            set_cookie_headers = list(headers.getall("set-cookie", []))
        for cookie_header in set_cookie_headers:
            if cookie_header.startswith("auth_token="):
                token = cookie_header.split("auth_token=", 1)[1].split(";", 1)[0]
                if token:
                    self._auth_token = token
            elif cookie_header.startswith("admin_token="):
                token = cookie_header.split("admin_token=", 1)[1].split(";", 1)[0]
                if token:
                    self._admin_token = token

    def _map_status_error(
        self,
        status: int,
        message: str,
        headers: Optional[aiohttp.typedefs.LooseHeaders] = None,
    ) -> HttpError:
        """Map HTTP status to HttpError."""
        if status == 401:
            return HttpError.unauthorized(message)
        if status == 404:
            return HttpError.not_found(message)
        if status == 429:
            return HttpError.rate_limited(
                message or "Rate limited",
                retry_after_ms=_retry_after_ms(headers),
            )
        if 400 <= status <= 499:
            return HttpError.bad_request(message)
        return HttpError.server_error(message, status)


def _retry_after_ms(headers: Optional[aiohttp.typedefs.LooseHeaders]) -> Optional[int]:
    if headers is None:
        return None

    def _header(name: str) -> Optional[str]:
        if hasattr(headers, "get"):
            value = headers.get(name)
            return str(value) if value is not None else None
        return None

    retry_after_ms = _header("retry-after-ms")
    if retry_after_ms:
        try:
            return int(retry_after_ms)
        except ValueError:
            return None

    retry_after = _header("retry-after")
    if retry_after:
        try:
            return int(float(retry_after) * 1000)
        except ValueError:
            return None

    return None


__all__ = ["LightconeHttp"]
