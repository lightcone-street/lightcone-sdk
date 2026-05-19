"""High-level Lightcone SDK client with builder pattern.

Mirrors rust/src/client.rs — unified entry point with sub-client accessors.
"""

from __future__ import annotations

from typing import Optional

from solders.pubkey import Pubkey

from .auth import AuthCredentials
from .auth.client import Auth
from .domain.admin.client import Admin
from .domain.faucet import FaucetRequest, FaucetResponse
from .domain.market.client import Markets
from .domain.metrics.client import Metrics
from .domain.notification.client import Notifications
from .domain.order.client import Orders
from .domain.orderbook.client import Orderbooks
from .domain.position.client import Positions
from .domain.price_history.client import PriceHistoryClient
from .domain.referral.client import Referrals
from .domain.trade.client import Trades
from .http.client import LightconeHttp
from .env import LightconeEnv
from .privy.client import Privy
from .rpc import Rpc
from .error import SdkError
from .rpc_failover import ActiveRpc, RpcFailoverState, is_infrastructure_error, FAST_RETRY_DELAY_SECS
from .shared.signing import ExternalSigner, SigningStrategy, SigningStrategyKind, classify_signer_error
from .shared.types import DepositSource
from .ws import WsConfig, WS_DEFAULT_CONFIG
from .ws.client import WsClient


class LightconeClient:
    """High-level client providing access to all Lightcone SDK sub-clients.

    Use LightconeClientBuilder to construct instances.

    Caching philosophy: The SDK is stateless for HTTP data. Caching is the
    consumer's responsibility.
    """

    def __init__(
        self,
        http: LightconeHttp,
        ws_config: Optional[WsConfig] = None,
        auth_credentials: Optional[AuthCredentials] = None,
        program_id: Optional[Pubkey] = None,
        connection: Optional[object] = None,
        backup_connection: Optional[object] = None,
        deposit_source: DepositSource = DepositSource.GLOBAL,
        signing_strategy: Optional[SigningStrategy] = None,
        primary_rpc_url: Optional[str] = None,
        backup_rpc_url: Optional[str] = None,
    ):
        self._http = http
        self._ws_config = ws_config or WS_DEFAULT_CONFIG
        self._program_id: Pubkey = program_id or LightconeEnv.PROD.program_id
        self._primary_connection = connection  # Optional[AsyncClient]
        self._backup_connection = backup_connection  # Optional[AsyncClient]
        self._rpc_failover_state = RpcFailoverState()
        self._deposit_source: DepositSource = deposit_source
        self._signing_strategy: Optional[SigningStrategy] = signing_strategy
        self._primary_rpc_url: Optional[str] = primary_rpc_url
        self._backup_rpc_url: Optional[str] = backup_rpc_url
        self._order_nonce: Optional[int] = None

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
        self._metrics = Metrics(self)
        self._rpc = Rpc(self)

    # ── Properties ───────────────────────────────────────────────────────

    @property
    def program_id(self) -> Pubkey:
        """On-chain program ID."""
        return self._program_id

    @property
    def connection(self) -> Optional[object]:
        """Currently-active Solana RPC connection (AsyncClient), resolved
        through failover state."""
        self._rpc_failover_state.maybe_recover_to_primary()
        if self._rpc_failover_state.active == ActiveRpc.PRIMARY:
            return self._primary_connection
        return self._backup_connection or self._primary_connection

    @property
    def rpc_failover_state(self) -> RpcFailoverState:
        return self._rpc_failover_state

    # ── Deposit source ───────────────────────────────────────────────────

    @property
    def deposit_source(self) -> DepositSource:
        """Get the current deposit source setting."""
        return self._deposit_source

    @deposit_source.setter
    def deposit_source(self, source: DepositSource) -> None:
        """Update the deposit source at runtime."""
        self._deposit_source = source

    def resolve_deposit_source(
        self, override_source: Optional[DepositSource] = None
    ) -> DepositSource:
        """Resolve deposit source: per-call override > client setting."""
        return override_source if override_source is not None else self._deposit_source

    # ── Signing strategy ───────────────────────────────────────────────

    @property
    def signing_strategy(self) -> Optional[SigningStrategy]:
        """Get the current signing strategy, if set."""
        return self._signing_strategy

    @signing_strategy.setter
    def signing_strategy(self, strategy: Optional[SigningStrategy]) -> None:
        """Set the signing strategy at runtime."""
        self._signing_strategy = strategy

    def set_signing_strategy(self, strategy: SigningStrategy) -> None:
        """Set the signing strategy at runtime."""
        self._signing_strategy = strategy

    def clear_signing_strategy(self) -> None:
        """Clear the signing strategy (e.g. on logout)."""
        self._signing_strategy = None

    # ── Nonce cache ───────────────────────────────────────────────────

    @property
    def order_nonce(self) -> Optional[int]:
        """Get the cached order nonce, if one has been set."""
        return self._order_nonce

    def set_order_nonce(self, nonce: int) -> None:
        """Cache an order nonce. This value will be used as the default nonce
        for subsequent orders that don't explicitly call ``.nonce()``."""
        self._order_nonce = nonce

    def clear_order_nonce(self) -> None:
        """Clear the cached nonce (e.g. on logout)."""
        self._order_nonce = None

    def _active_rpc_url(self) -> str:
        """Resolve the currently-active RPC URL, recovering to primary if cooldown elapsed."""
        self._rpc_failover_state.maybe_recover_to_primary()
        if self._rpc_failover_state.active == ActiveRpc.PRIMARY:
            url = self._primary_rpc_url
        else:
            url = self._backup_rpc_url or self._primary_rpc_url
        if url is None:
            raise SdkError("rpc_url is not configured on the client")
        return url

    async def _rpc_call_with_failover(self, body: dict) -> dict:
        """Execute a JSON-RPC call with fast retry + failover."""
        import aiohttp

        active_url = self._active_rpc_url()
        original_active = self._rpc_failover_state.active

        async def _post(url: str) -> dict:
            async with aiohttp.ClientSession() as session:
                async with session.post(url, json=body) as resp:
                    if resp.status in (502, 503, 504):
                        raise aiohttp.ClientResponseError(
                            resp.request_info, resp.history, status=resp.status
                        )
                    return await resp.json()

        # First attempt.
        try:
            return await _post(active_url)
        except Exception as first_error:
            if not is_infrastructure_error(first_error):
                raise SdkError(f"RPC failed: {first_error}") from first_error

        # Fast retry on same URL.
        await asyncio.sleep(FAST_RETRY_DELAY_SECS)
        try:
            return await _post(active_url)
        except Exception as retry_error:
            if not is_infrastructure_error(retry_error):
                raise SdkError(f"RPC failed: {retry_error}") from retry_error

        # Flip and try the other URL.
        other_url = (
            self._backup_rpc_url
            if original_active == ActiveRpc.PRIMARY
            else self._primary_rpc_url
        )
        if other_url is not None:
            try:
                result = await _post(other_url)
                if original_active == ActiveRpc.PRIMARY:
                    self._rpc_failover_state.flip_to_backup()
                else:
                    self._rpc_failover_state.flip_to_primary()
                return result
            except Exception as both_error:
                raise SdkError(
                    f"RPC failed on both endpoints: {both_error}"
                ) from both_error

        raise SdkError(f"RPC failed: {retry_error}") from retry_error  # noqa: F821

    def _require_signing_strategy(self) -> SigningStrategy:
        """Get the signing strategy or raise if not set."""
        if self._signing_strategy is None:
            raise SdkError("signing strategy is not set on the client")
        return self._signing_strategy

    async def sign_and_submit_tx(self, tx: object) -> str:
        """Sign and submit a transaction using the client's signing strategy.

        - **Native**: signs locally with keypair, submits via RPC
        - **WalletAdapter**: signs via external signer, submits via RPC
        - **Privy**: serializes unsigned tx to base64, sends to backend

        Args:
            tx: A ``solders.transaction.Transaction`` instance.

        Returns:
            Transaction signature string.
        """
        strategy = self._require_signing_strategy()

        if strategy.kind == SigningStrategyKind.NATIVE:
            from solders.keypair import Keypair as _Keypair
            keypair: _Keypair = strategy.keypair  # type: ignore[assignment]
            blockhash = await self.rpc().get_latest_blockhash()
            tx.sign([keypair], blockhash)  # type: ignore[attr-defined]
            response = await self._connection.send_raw_transaction(bytes(tx))  # type: ignore[union-attr]
            return str(response.value)

        elif strategy.kind == SigningStrategyKind.WALLET_ADAPTER:
            signer: ExternalSigner = strategy.signer  # type: ignore[assignment]
            import base64 as _b64
            tx_bytes = bytes(tx)  # type: ignore[arg-type]
            signed_bytes = await signer.sign_transaction(tx_bytes)
            base64_tx = _b64.b64encode(signed_bytes).decode("ascii")
            # Submit via RPC with failover
            data = await self._rpc_call_with_failover({
                "jsonrpc": "2.0", "id": 1,
                "method": "sendTransaction",
                "params": [base64_tx, {"encoding": "base64", "preflightCommitment": "confirmed"}],
            })
            if "error" in data:
                raise SdkError(f"RPC error: {data['error']}")
            return data["result"]

        elif strategy.kind == SigningStrategyKind.PRIVY:
            import base64 as _b64
            tx_bytes = bytes(tx)  # type: ignore[arg-type]
            base64_tx = _b64.b64encode(tx_bytes).decode("ascii")
            result = await self.privy().sign_and_send_tx(
                strategy.wallet_id, base64_tx,  # type: ignore[arg-type]
            )
            return result.hash

        raise SdkError(f"Unsupported signing strategy: {strategy.kind}")

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

    def metrics(self) -> Metrics:
        """Metrics sub-client — platform / market / orderbook / category /
        deposit-token volume metrics, market leaderboard, and time-series history."""
        return self._metrics

    async def claim(self, wallet_address: str) -> FaucetResponse:
        """Request testnet SOL + whitelisted deposit tokens for a wallet.

        Only active on environments whose backend has the faucet enabled
        (typically local and staging).

        POST /api/claim
        """
        request = FaucetRequest(wallet_address=wallet_address)
        data = await self._http.post("/api/claim", request.to_dict())
        return FaucetResponse.from_dict(data)

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

    # ── Auth token (cookie) ─────────────────────────────────────────────

    @property
    def auth_token(self) -> Optional[str]:
        """Current ``auth_token`` cookie value, if any.

        Populated by the SDK after a successful login, then attached on
        every authed request. Useful for forwarding the token through
        ``*_with_auth`` methods or persisting the session across
        processes.
        """
        return self._http.auth_token

    def clear_auth_token(self) -> None:
        """Clear the cached ``auth_token``.

        Subsequent authed calls will go out without a ``Cookie`` header
        (and 401) unless they use a ``*_with_auth`` variant.
        """
        self._http.clear_auth_token()

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
        environment = LightconeEnv.PROD
        self._base_url: str = environment.api_url
        self._ws_url: str = environment.ws_url
        self._auth_credentials: Optional[AuthCredentials] = None
        self._ws_config: Optional[WsConfig] = None
        self._timeout: int = 30
        self._program_id: Optional[Pubkey] = environment.program_id
        self._deposit_source: DepositSource = DepositSource.GLOBAL
        self._signing_strategy: Optional[SigningStrategy] = None
        self._primary_rpc_url: Optional[str] = environment.rpc_url
        self._backup_rpc_url: Optional[str] = None
        self._connection: Optional[object] = None

    def env(self, environment: LightconeEnv) -> "LightconeClientBuilder":
        """Set the deployment environment. Configures the API URL, WebSocket URL,
        RPC URL, and program ID for the given environment.

        Individual URL overrides (e.g. ``.base_url()``) take precedence when
        called **after** ``.env()``.
        """
        self._base_url = environment.api_url
        self._ws_url = environment.ws_url
        self._program_id = environment.program_id
        self._primary_rpc_url = environment.rpc_url
        return self

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

    def deposit_source(self, source: DepositSource) -> "LightconeClientBuilder":
        """Set the default deposit source for orders, deposits, and withdrawals.

        Defaults to ``DepositSource.GLOBAL``. Can be overridden per-call.
        """
        self._deposit_source = source
        return self

    def native_signer(self, keypair: object) -> "LightconeClientBuilder":
        """Set a native keypair for signing orders, cancels, and transactions."""
        self._signing_strategy = SigningStrategy.native(keypair)
        return self

    def external_signer(self, signer: ExternalSigner) -> "LightconeClientBuilder":
        """Set an external signer for browser wallet adapters."""
        self._signing_strategy = SigningStrategy.wallet_adapter(signer)
        return self

    def privy_wallet_id(self, wallet_id: str) -> "LightconeClientBuilder":
        """Set a Privy embedded wallet ID for signing."""
        self._signing_strategy = SigningStrategy.privy(wallet_id)
        return self

    def rpc_url(self, url: str) -> "LightconeClientBuilder":
        """Set the primary Solana RPC URL for blockhash fetching, transaction submission, and on-chain reads."""
        self._primary_rpc_url = url
        return self

    def backup_rpc_url(self, url: str) -> "LightconeClientBuilder":
        """Set a backup Solana RPC URL for automatic failover."""
        self._backup_rpc_url = url
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
            pong_timeout_ms=10_000,
        )

        # Resolve connection: explicit connection takes priority over rpc_url
        connection = self._connection
        if connection is None and self._primary_rpc_url is not None:
            from solana.rpc.async_api import AsyncClient
            connection = AsyncClient(self._primary_rpc_url)

        backup_connection = None
        if self._backup_rpc_url is not None:
            from solana.rpc.async_api import AsyncClient
            backup_connection = AsyncClient(self._backup_rpc_url)

        return LightconeClient(
            http=http,
            ws_config=ws_config,
            auth_credentials=self._auth_credentials,
            program_id=self._program_id,
            connection=connection,
            backup_connection=backup_connection,
            deposit_source=self._deposit_source,
            signing_strategy=self._signing_strategy,
            primary_rpc_url=self._primary_rpc_url,
            backup_rpc_url=self._backup_rpc_url,
        )


__all__ = [
    "LightconeClient",
    "LightconeClientBuilder",
]
