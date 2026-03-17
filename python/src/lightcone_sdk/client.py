"""High-level Lightcone SDK client with builder pattern.

Mirrors rust/src/client.rs — unified entry point with sub-client accessors.
"""

from __future__ import annotations

from typing import Optional

from solders.pubkey import Pubkey

from .auth import AuthCredentials
from .auth.client import Auth
from .domain.admin.client import Admin
from .domain.market.client import Markets
from .domain.notification.client import Notifications
from .domain.order.client import Orders
from .domain.orderbook.client import Orderbooks
from .domain.orderbook.wire import DecimalsResponse
from .domain.position.client import Positions
from .domain.price_history.client import PriceHistoryClient
from .domain.referral.client import Referrals
from .domain.trade.client import Trades
from .http.client import LightconeHttp
from .network import DEFAULT_API_URL, DEFAULT_WS_URL
from .privy.client import Privy
from .program.constants import PROGRAM_ID
from .rpc import Rpc
from .ws import WsConfig, WS_DEFAULT_CONFIG
from .ws.client import WsClient


class LightconeClient:
    """High-level client providing access to all Lightcone SDK sub-clients.

    Use LightconeClientBuilder to construct instances.

    Caching philosophy: The SDK is stateless for HTTP data. Caching is the
    consumer's responsibility. The only internal cache is decimals_cache for
    orderbook decimals, which are effectively immutable.
    """

    def __init__(
        self,
        http: LightconeHttp,
        ws_config: Optional[WsConfig] = None,
        auth_credentials: Optional[AuthCredentials] = None,
        program_id: Optional[Pubkey] = None,
        connection: Optional[object] = None,
    ):
        self._http = http
        self._ws_config = ws_config or WS_DEFAULT_CONFIG
        self._decimals_cache: dict[str, DecimalsResponse] = {}
        self._program_id: Pubkey = program_id or PROGRAM_ID
        self._connection = connection  # Optional[AsyncClient]

        # Sub-clients (all take self reference)
        self._markets = Markets(self)
        self._orders = Orders(self)
        self._orderbooks = Orderbooks(self)
        self._positions = Positions(self)
        self._trades = Trades(self)
        self._price_history = PriceHistoryClient(self)
        self._admin = Admin(self)
        self._auth = Auth(self, auth_credentials)
        self._privy = Privy(self)
        self._referrals = Referrals(self)
        self._notifications = Notifications(self)
        self._rpc = Rpc(self)

    # ── Properties ───────────────────────────────────────────────────────

    @property
    def program_id(self) -> Pubkey:
        """On-chain program ID."""
        return self._program_id

    @property
    def connection(self) -> Optional[object]:
        """Optional Solana RPC connection (AsyncClient)."""
        return self._connection

    # ── Sub-client accessors ─────────────────────────────────────────────

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

    def privy(self) -> Privy:
        return self._privy

    def referrals(self) -> Referrals:
        return self._referrals

    def notifications(self) -> Notifications:
        return self._notifications

    def rpc(self) -> Rpc:
        """RPC sub-client — PDA helpers, account fetchers, and blockhash access."""
        return self._rpc

    def ws(self) -> WsClient:
        """Create a new WebSocket client with the current config."""
        client = WsClient(self._ws_config)
        if self._http.has_auth_token():
            client.set_auth_token(self._http.auth_token)
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
        self._ws_config: Optional[WsConfig] = None
        self._timeout: int = 30
        self._program_id: Optional[Pubkey] = None
        self._rpc_url: Optional[str] = None
        self._connection: Optional[object] = None

    def base_url(self, url: str) -> "LightconeClientBuilder":
        self._base_url = url
        return self

    def ws_url(self, url: str) -> "LightconeClientBuilder":
        self._ws_url = url
        return self

    def auth(self, credentials: AuthCredentials) -> "LightconeClientBuilder":
        self._auth_credentials = credentials
        return self

    def ws_config(self, config: WsConfig) -> "LightconeClientBuilder":
        self._ws_config = config
        return self

    def timeout(self, timeout: int) -> "LightconeClientBuilder":
        self._timeout = timeout
        return self

    def program_id(self, pid: Pubkey) -> "LightconeClientBuilder":
        """Set a custom on-chain program ID (defaults to canonical Lightcone program)."""
        self._program_id = pid
        return self

    def rpc_url(self, url: str) -> "LightconeClientBuilder":
        """Set the Solana RPC URL for on-chain reads and transaction building."""
        self._rpc_url = url
        return self

    def rpc_connection(self, connection: object) -> "LightconeClientBuilder":
        """Set a pre-built Solana AsyncClient for on-chain reads."""
        self._connection = connection
        return self

    def build(self) -> LightconeClient:
        """Build the LightconeClient."""
        http = LightconeHttp(
            base_url=self._base_url,
            timeout=self._timeout,
        )

        ws_config = self._ws_config or WsConfig(
            url=self._ws_url,
            reconnect=True,
            max_reconnect_attempts=10,
            base_reconnect_delay_ms=1000,
            ping_interval_ms=30_000,
            pong_timeout_ms=1_000,
        )

        # Resolve connection: explicit connection takes priority over rpc_url
        connection = self._connection
        if connection is None and self._rpc_url is not None:
            from solana.rpc.async_api import AsyncClient
            connection = AsyncClient(self._rpc_url)

        return LightconeClient(
            http=http,
            ws_config=ws_config,
            auth_credentials=self._auth_credentials,
            program_id=self._program_id,
            connection=connection,
        )


__all__ = [
    "LightconeClient",
    "LightconeClientBuilder",
]
