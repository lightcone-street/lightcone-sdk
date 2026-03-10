"""High-level Lightcone SDK client with builder pattern."""

from typing import Optional

from .auth import AuthCredentials
from .auth.client import Auth
from .domain.market.client import Markets
from .domain.order.client import Orders
from .domain.orderbook.client import Orderbooks
from .domain.orderbook.wire import DecimalsResponse
from .domain.position.client import Positions
from .domain.trade.client import Trades
from .domain.price_history.client import PriceHistoryClient
from .domain.admin.client import Admin
from .domain.referral.client import Referrals
from .http.client import LightconeHttp
from .http.retry import RetryConfig
from .network import DEFAULT_API_URL, DEFAULT_WS_URL
from .ws import WsConfig, WS_DEFAULT_CONFIG
from .ws.client import WsClient


class LightconeClient:
    """High-level client providing access to all Lightcone SDK sub-clients.

    Use LightconeClientBuilder to construct instances.
    """

    def __init__(
        self,
        http: LightconeHttp,
        ws_config: Optional[WsConfig] = None,
        auth_credentials: Optional[AuthCredentials] = None,
    ):
        self._http = http
        self._ws_config = ws_config or WS_DEFAULT_CONFIG
        self._decimals_cache: dict[str, DecimalsResponse] = {}

        # Sub-clients
        self._markets = Markets(http)
        self._orders = Orders(http)
        self._orderbooks = Orderbooks(http, self._decimals_cache)
        self._positions = Positions(http)
        self._trades = Trades(http)
        self._price_history = PriceHistoryClient(http)
        self._admin = Admin(http)
        self._auth = Auth(http)
        self._referrals = Referrals(http)

        # Apply auth if provided
        if auth_credentials:
            self._http.set_auth_token(auth_credentials.token)

    def markets(self) -> Markets:
        return self._markets

    def orders(self) -> Orders:
        return self._orders

    def orderbooks(self) -> Orderbooks:
        return self._orderbooks

    def positions(self) -> Positions:
        return self._positions

    def trades(self) -> Trades:
        return self._trades

    def price_history(self) -> PriceHistoryClient:
        return self._price_history

    def admin(self) -> Admin:
        return self._admin

    def auth(self) -> Auth:
        return self._auth

    def referrals(self) -> Referrals:
        return self._referrals

    def ws(self) -> WsClient:
        """Create a new WebSocket client with the current config."""
        client = WsClient(self._ws_config)
        if self._http.has_auth_token():
            client.set_auth_token(self._http._auth_token)
        return client

    def ws_config(self) -> WsConfig:
        return self._ws_config

    def clear_decimals_cache(self) -> None:
        self._decimals_cache.clear()

    async def close(self) -> None:
        """Close the HTTP session."""
        await self._http.close()

    async def __aenter__(self) -> "LightconeClient":
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        await self.close()


class LightconeClientBuilder:
    """Builder for constructing LightconeClient instances."""

    def __init__(self):
        self._base_url: str = DEFAULT_API_URL
        self._ws_url: str = DEFAULT_WS_URL
        self._auth_credentials: Optional[AuthCredentials] = None
        self._retry_config: Optional[RetryConfig] = None
        self._ws_config: Optional[WsConfig] = None
        self._timeout: int = 30

    def base_url(self, url: str) -> "LightconeClientBuilder":
        self._base_url = url
        return self

    def ws_url(self, url: str) -> "LightconeClientBuilder":
        self._ws_url = url
        return self

    def auth(self, credentials: AuthCredentials) -> "LightconeClientBuilder":
        self._auth_credentials = credentials
        return self

    def retry_config(self, config: RetryConfig) -> "LightconeClientBuilder":
        self._retry_config = config
        return self

    def ws_config(self, config: WsConfig) -> "LightconeClientBuilder":
        self._ws_config = config
        return self

    def timeout(self, timeout: int) -> "LightconeClientBuilder":
        self._timeout = timeout
        return self

    def build(self) -> LightconeClient:
        """Build the LightconeClient."""
        http = LightconeHttp(
            base_url=self._base_url,
            auth_token=self._auth_credentials.token if self._auth_credentials else None,
            retry_config=self._retry_config,
            timeout=self._timeout,
        )

        ws_config = self._ws_config or WsConfig(
            url=self._ws_url,
            reconnect=True,
            max_reconnect_attempts=10,
            base_reconnect_delay_ms=1000,
            ping_interval_ms=30_000,
            pong_timeout_ms=10_000,
        )

        return LightconeClient(
            http=http,
            ws_config=ws_config,
            auth_credentials=self._auth_credentials,
        )


__all__ = [
    "LightconeClient",
    "LightconeClientBuilder",
]
