"""Lightcone REST API client implementation."""

import asyncio
import json
from typing import Any, Optional
from urllib.parse import quote

import aiohttp
from nacl.signing import SigningKey
from solders.keypair import Keypair

from .error import (
    ApiError,
    HttpError,
    NotFoundError,
    BadRequestError,
    ForbiddenError,
    UnauthorizedError,
    RateLimitedError,
    ConflictError,
    ServerError,
    DeserializeError,
    UnexpectedStatusError,
    ErrorResponse,
)
from .validation import (
    validate_pubkey,
    validate_signature,
    validate_limit,
)
from .retry import RetryConfig, is_retryable, calculate_delay
from .types import (
    MarketsResponse,
    MarketInfoResponse,
    DepositAssetsResponse,
    OrderbookResponse,
    SubmitOrderRequest,
    OrderResponse,
    CancelResponse,
    CancelAllResponse,
    PositionsResponse,
    MarketPositionsResponse,
    UserOrdersResponse,
    PriceHistoryParams,
    PriceHistoryResponse,
    TradesParams,
    TradesResponse,
    AdminResponse,
    CreateOrderbookRequest,
    CreateOrderbookResponse,
    DecimalsResponse,
)

DEFAULT_TIMEOUT_SECS = 30


class LightconeApiClient:
    """Lightcone REST API client.

    Provides methods for all Lightcone API endpoints including markets, orderbooks,
    orders, positions, and price history.

    Example:
        ```python
        async with LightconeApiClient("https://api.lightcone.xyz") as client:
            markets = await client.get_markets()
            print(f"Found {markets.total} markets")
        ```
    """

    def __init__(
        self,
        base_url: str,
        timeout: int = DEFAULT_TIMEOUT_SECS,
        headers: Optional[dict[str, str]] = None,
        retry_config: Optional[RetryConfig] = None,
        auth_token: Optional[str] = None,
    ):
        """Create a new client with the given base URL.

        Args:
            base_url: The base URL of the Lightcone API
            timeout: Request timeout in seconds
            headers: Optional additional headers for all requests
            retry_config: Optional retry configuration for failed requests
            auth_token: Optional authentication token
        """
        self._base_url = base_url.rstrip("/")
        self._timeout = aiohttp.ClientTimeout(total=timeout)
        self._headers = {
            "Content-Type": "application/json",
            "Accept": "application/json",
        }
        if headers:
            self._headers.update(headers)
        self._session: Optional[aiohttp.ClientSession] = None
        self._retry_config = retry_config or RetryConfig.default()
        self._auth_token: Optional[str] = auth_token
        self._decimals_cache: dict[str, DecimalsResponse] = {}

    @property
    def base_url(self) -> str:
        """Get the base URL."""
        return self._base_url

    def set_auth_token(self, token: str) -> None:
        """Set the authentication token."""
        self._auth_token = token

    def clear_auth_token(self) -> None:
        """Clear the authentication token."""
        self._auth_token = None

    def has_auth_token(self) -> bool:
        """Check if an auth token is set."""
        return self._auth_token is not None

    async def login(self, keypair: Keypair) -> str:
        """Authenticate with the API and store the auth token.

        Args:
            keypair: The Solana keypair to authenticate with

        Returns:
            The auth token string

        Raises:
            ApiError: If authentication fails
        """
        import time

        timestamp = int(time.time())
        message = f"Sign in to Lightcone: {timestamp}"
        message_bytes = message.encode("utf-8")

        secret_bytes = bytes(keypair)
        seed = secret_bytes[:32]
        signing_key = SigningKey(seed)
        signed = signing_key.sign(message_bytes)
        signature_hex = signed.signature.hex()

        pubkey = str(keypair.pubkey())

        session = await self._ensure_session()
        url = f"{self._base_url}/api/auth/login_or_register_with_message"
        payload = {
            "pubkey": pubkey,
            "message": message,
            "signature": signature_hex,
        }

        async with session.post(url, json=payload) as response:
            data = await self._handle_response(response)
            token = data.get("token") or data.get("auth_token", "")

            if not token:
                # Try cookies
                cookies = response.cookies
                if "auth_token" in cookies:
                    token = cookies["auth_token"].value

            if not token:
                raise ServerError("No auth token in login response")

            self._auth_token = token
            return token

    async def __aenter__(self) -> "LightconeApiClient":
        """Enter async context manager."""
        await self._ensure_session()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit async context manager."""
        await self.close()

    async def _ensure_session(self) -> aiohttp.ClientSession:
        """Ensure we have an active session."""
        if self._session is None or self._session.closed:
            self._session = aiohttp.ClientSession(
                timeout=self._timeout,
                headers=self._headers,
            )
        return self._session

    async def close(self) -> None:
        """Close the HTTP session."""
        if self._session and not self._session.closed:
            await self._session.close()
            self._session = None

    def _get_auth_headers(self) -> dict[str, str]:
        """Get headers with auth token if available."""
        headers = {}
        if self._auth_token:
            headers["Cookie"] = f"auth_token={self._auth_token}"
        return headers

    def _map_status_error(self, status: int, message: str) -> ApiError:
        """Map HTTP status code to ApiError."""
        if status == 401:
            return UnauthorizedError(message)
        elif status == 400:
            return BadRequestError(message)
        elif status == 403:
            return ForbiddenError(message)
        elif status == 404:
            return NotFoundError(message)
        elif status == 409:
            return ConflictError(message)
        elif status == 429:
            return RateLimitedError(message)
        elif status >= 500:
            return ServerError(message)
        else:
            return UnexpectedStatusError(status, message)

    async def _handle_response(self, response: aiohttp.ClientResponse) -> dict:
        """Handle HTTP response and map errors."""
        if response.status >= 200 and response.status < 300:
            try:
                return await response.json()
            except (ValueError, json.JSONDecodeError, aiohttp.ContentTypeError) as e:
                raise DeserializeError(f"Failed to deserialize response: {e}")
        else:
            error_text = await response.text()
            try:
                error_data = json.loads(error_text)
                error_resp = ErrorResponse.from_dict(error_data)
                error_msg = error_resp.get_message()
            except (ValueError, KeyError, json.JSONDecodeError):
                error_msg = error_text or "Unknown error"

            raise self._map_status_error(response.status, error_msg)

    async def _request_with_retry(
        self,
        method: str,
        url: str,
        **kwargs: Any,
    ) -> dict:
        """Make request with retry logic.

        Args:
            method: HTTP method (GET, POST, etc.)
            url: Request URL
            **kwargs: Additional arguments for the request

        Returns:
            JSON response data

        Raises:
            ApiError: If all retries fail
        """
        session = await self._ensure_session()
        last_error: Optional[Exception] = None

        for attempt in range(self._retry_config.max_retries + 1):
            try:
                async with session.request(method, url, **kwargs) as response:
                    return await self._handle_response(response)
            except Exception as e:
                last_error = e

                # Don't retry if not retryable or last attempt
                if not is_retryable(e) or attempt >= self._retry_config.max_retries:
                    raise

                # Calculate and apply delay
                delay = calculate_delay(attempt, self._retry_config)
                await asyncio.sleep(delay)

        # Should not reach here, but satisfy type checker
        raise last_error or RuntimeError("Unexpected retry loop exit")

    # =========================================================================
    # Health endpoints
    # =========================================================================

    async def health_check(self) -> None:
        """Check API health.

        Raises:
            ServerError: If the health check fails
        """
        session = await self._ensure_session()
        url = f"{self._base_url}/health"
        async with session.get(url) as response:
            if not response.ok:
                raise ServerError("Health check failed")

    # =========================================================================
    # Market endpoints
    # =========================================================================

    async def get_markets(self) -> MarketsResponse:
        """Get all markets.

        Returns:
            MarketsResponse containing list of markets and total count
        """
        session = await self._ensure_session()
        url = f"{self._base_url}/api/markets"
        async with session.get(url) as response:
            data = await self._handle_response(response)
            return MarketsResponse.from_dict(data)

    async def get_market(self, market_pubkey: str) -> MarketInfoResponse:
        """Get market details by pubkey.

        Args:
            market_pubkey: The market's public key

        Returns:
            MarketInfoResponse containing market details and deposit assets

        Raises:
            InvalidParameterError: If market_pubkey is invalid
        """
        validate_pubkey(market_pubkey, "market_pubkey")
        session = await self._ensure_session()
        url = f"{self._base_url}/api/markets/{quote(market_pubkey, safe='')}"
        async with session.get(url) as response:
            data = await self._handle_response(response)
            return MarketInfoResponse.from_dict(data)

    async def get_market_by_slug(self, slug: str) -> MarketInfoResponse:
        """Get market by URL-friendly slug.

        Args:
            slug: The market's URL slug

        Returns:
            MarketInfoResponse containing market details

        Raises:
            ValueError: If slug is empty
        """
        if not slug or not slug.strip():
            raise ValueError("slug cannot be empty")
        session = await self._ensure_session()
        url = f"{self._base_url}/api/markets/by-slug/{quote(slug, safe='')}"
        async with session.get(url) as response:
            data = await self._handle_response(response)
            return MarketInfoResponse.from_dict(data)

    async def get_deposit_assets(self, market_pubkey: str) -> DepositAssetsResponse:
        """Get deposit assets for a market.

        Args:
            market_pubkey: The market's public key

        Returns:
            DepositAssetsResponse containing deposit assets

        Raises:
            InvalidParameterError: If market_pubkey is invalid
        """
        validate_pubkey(market_pubkey, "market_pubkey")
        session = await self._ensure_session()
        url = f"{self._base_url}/api/markets/{quote(market_pubkey, safe='')}/deposit-assets"
        async with session.get(url) as response:
            data = await self._handle_response(response)
            return DepositAssetsResponse.from_dict(data)

    # =========================================================================
    # Orderbook endpoints
    # =========================================================================

    async def get_orderbook(
        self,
        orderbook_id: str,
        depth: Optional[int] = None,
    ) -> OrderbookResponse:
        """Get orderbook depth.

        Args:
            orderbook_id: Orderbook identifier
            depth: Optional max price levels per side (0 or None = all)

        Returns:
            OrderbookResponse containing bids and asks

        Raises:
            ValueError: If orderbook_id is empty
        """
        if not orderbook_id or not orderbook_id.strip():
            raise ValueError("orderbook_id cannot be empty")
        session = await self._ensure_session()
        url = f"{self._base_url}/api/orderbook/{quote(orderbook_id, safe='')}"
        params = {}
        if depth is not None:
            params["depth"] = depth

        async with session.get(url, params=params if params else None) as response:
            data = await self._handle_response(response)
            return OrderbookResponse.from_dict(data)

    async def get_orderbook_decimals(
        self,
        orderbook_id: str,
    ) -> DecimalsResponse:
        """Get decimal configuration for an orderbook, with in-memory caching.

        Args:
            orderbook_id: Orderbook identifier

        Returns:
            DecimalsResponse with base, quote, and price decimals
        """
        if orderbook_id in self._decimals_cache:
            return self._decimals_cache[orderbook_id]

        session = await self._ensure_session()
        url = f"{self._base_url}/api/orderbook/{quote(orderbook_id, safe='')}/decimals"
        async with session.get(url) as response:
            data = await self._handle_response(response)
            result = DecimalsResponse.from_dict(data)
            self._decimals_cache[orderbook_id] = result
            return result

    async def prefetch_decimals(self, orderbook_ids: list[str]) -> None:
        """Prefetch decimal configurations for multiple orderbooks.

        Args:
            orderbook_ids: List of orderbook identifiers to prefetch
        """
        for orderbook_id in orderbook_ids:
            if orderbook_id not in self._decimals_cache:
                await self.get_orderbook_decimals(orderbook_id)

    # =========================================================================
    # Order endpoints
    # =========================================================================

    async def submit_order(self, request: SubmitOrderRequest) -> OrderResponse:
        """Submit a new order.

        The order must be pre-signed with the maker's Ed25519 key.

        Args:
            request: The order submission request

        Returns:
            OrderResponse with order hash, status, and fills

        Raises:
            InvalidParameterError: If any pubkey or signature is invalid
        """
        # Validate inputs
        validate_pubkey(request.maker, "maker")
        validate_pubkey(request.market_pubkey, "market_pubkey")
        validate_pubkey(request.base_token, "base_token")
        validate_pubkey(request.quote_token, "quote_token")
        validate_signature(request.signature)

        session = await self._ensure_session()
        url = f"{self._base_url}/api/orders/submit"
        async with session.post(url, json=request.to_dict()) as response:
            data = await self._handle_response(response)
            return OrderResponse.from_dict(data)

    async def submit_full_order(
        self,
        order: "FullOrder",
        orderbook_id: str,
    ) -> OrderResponse:
        """Submit a signed FullOrder to the API.

        Converts the FullOrder to a SubmitOrderRequest and submits it.

        Args:
            order: A signed FullOrder
            orderbook_id: The orderbook identifier

        Returns:
            OrderResponse with order hash, status, and fills
        """
        from ..program.orders import to_submit_request

        request = to_submit_request(order, orderbook_id)
        return await self.submit_order(request)

    async def cancel_order(self, order_hash: str, maker: str) -> CancelResponse:
        """Cancel a specific order.

        Args:
            order_hash: Hash of order to cancel
            maker: Must match order creator

        Returns:
            CancelResponse with cancellation status

        Raises:
            InvalidParameterError: If maker pubkey is invalid
        """
        validate_pubkey(maker, "maker")
        session = await self._ensure_session()
        url = f"{self._base_url}/api/orders/cancel"
        request = {"order_hash": order_hash, "maker": maker}
        async with session.post(url, json=request) as response:
            data = await self._handle_response(response)
            return CancelResponse.from_dict(data)

    async def cancel_all_orders(
        self,
        user_pubkey: str,
        market_pubkey: Optional[str] = None,
    ) -> CancelAllResponse:
        """Cancel all orders for a user.

        Args:
            user_pubkey: User's public key
            market_pubkey: Optional market filter

        Returns:
            CancelAllResponse with cancelled order hashes

        Raises:
            InvalidParameterError: If any pubkey is invalid
        """
        validate_pubkey(user_pubkey, "user_pubkey")
        if market_pubkey:
            validate_pubkey(market_pubkey, "market_pubkey")

        session = await self._ensure_session()
        url = f"{self._base_url}/api/orders/cancel-all"
        request: dict = {"user_pubkey": user_pubkey}
        if market_pubkey:
            request["market_pubkey"] = market_pubkey

        async with session.post(url, json=request) as response:
            data = await self._handle_response(response)
            return CancelAllResponse.from_dict(data)

    # =========================================================================
    # User endpoints
    # =========================================================================

    async def get_user_positions(self, user_pubkey: str) -> PositionsResponse:
        """Get all positions for a user.

        Args:
            user_pubkey: User's public key

        Returns:
            PositionsResponse containing user positions

        Raises:
            InvalidParameterError: If user_pubkey is invalid
        """
        validate_pubkey(user_pubkey, "user_pubkey")
        session = await self._ensure_session()
        url = f"{self._base_url}/api/users/{quote(user_pubkey, safe='')}/positions"
        async with session.get(url) as response:
            data = await self._handle_response(response)
            return PositionsResponse.from_dict(data)

    async def get_user_market_positions(
        self,
        user_pubkey: str,
        market_pubkey: str,
    ) -> MarketPositionsResponse:
        """Get user positions in a specific market.

        Args:
            user_pubkey: User's public key
            market_pubkey: Market's public key

        Returns:
            MarketPositionsResponse containing positions in the market

        Raises:
            InvalidParameterError: If user_pubkey or market_pubkey is invalid
        """
        validate_pubkey(user_pubkey, "user_pubkey")
        validate_pubkey(market_pubkey, "market_pubkey")
        session = await self._ensure_session()
        url = f"{self._base_url}/api/users/{quote(user_pubkey, safe='')}/markets/{quote(market_pubkey, safe='')}/positions"
        async with session.get(url) as response:
            data = await self._handle_response(response)
            return MarketPositionsResponse.from_dict(data)

    async def get_user_orders(
        self,
        user_pubkey: str,
        cursor: Optional[str] = None,
        limit: Optional[int] = None,
    ) -> UserOrdersResponse:
        """Get all open orders and balances for a user.

        Args:
            user_pubkey: User's public key
            cursor: Optional pagination cursor
            limit: Optional result limit

        Returns:
            UserOrdersResponse containing orders, balances, and next_cursor

        Raises:
            InvalidParameterError: If user_pubkey is invalid
        """
        validate_pubkey(user_pubkey, "user_pubkey")
        session = await self._ensure_session()
        url = f"{self._base_url}/api/users/orders"
        request: dict[str, Any] = {"user_pubkey": user_pubkey}
        if cursor is not None:
            request["cursor"] = cursor
        if limit is not None:
            request["limit"] = limit

        headers = self._get_auth_headers()
        async with session.post(url, json=request, headers=headers) as response:
            data = await self._handle_response(response)
            return UserOrdersResponse.from_dict(data)

    # =========================================================================
    # Price history endpoints
    # =========================================================================

    async def get_price_history(
        self,
        params: PriceHistoryParams,
    ) -> PriceHistoryResponse:
        """Get historical price data (candlesticks).

        Args:
            params: Price history query parameters

        Returns:
            PriceHistoryResponse containing price points

        Raises:
            InvalidParameterError: If limit is out of bounds
        """
        if params.limit is not None:
            validate_limit(params.limit)

        session = await self._ensure_session()
        url = f"{self._base_url}/api/price-history"
        query_params = params.to_query_params()

        async with session.get(url, params=query_params) as response:
            data = await self._handle_response(response)
            return PriceHistoryResponse.from_dict(data)

    # =========================================================================
    # Trade endpoints
    # =========================================================================

    async def get_trades(self, params: TradesParams) -> TradesResponse:
        """Get executed trades.

        Args:
            params: Trades query parameters

        Returns:
            TradesResponse containing trades

        Raises:
            InvalidParameterError: If limit is out of bounds
        """
        if params.limit is not None:
            validate_limit(params.limit)

        session = await self._ensure_session()
        url = f"{self._base_url}/api/trades"
        query_params = params.to_query_params()

        async with session.get(url, params=query_params) as response:
            data = await self._handle_response(response)
            return TradesResponse.from_dict(data)

    # =========================================================================
    # Admin endpoints
    # =========================================================================

    async def admin_health_check(self) -> AdminResponse:
        """Admin health check endpoint.

        Returns:
            AdminResponse with status and message
        """
        session = await self._ensure_session()
        url = f"{self._base_url}/api/admin/test"
        async with session.get(url) as response:
            data = await self._handle_response(response)
            return AdminResponse.from_dict(data)

    async def create_orderbook(
        self,
        request: CreateOrderbookRequest,
    ) -> CreateOrderbookResponse:
        """Create a new orderbook for a market.

        Args:
            request: Orderbook creation request

        Returns:
            CreateOrderbookResponse with created orderbook details
        """
        session = await self._ensure_session()
        url = f"{self._base_url}/api/admin/create-orderbook"
        async with session.post(url, json=request.to_dict()) as response:
            data = await self._handle_response(response)
            return CreateOrderbookResponse.from_dict(data)
