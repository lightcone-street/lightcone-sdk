"""High-level Lightcone SDK client with builder pattern.

Mirrors rust/src/client.rs — unified entry point with sub-client accessors.
"""

from __future__ import annotations

import asyncio
from collections.abc import Sequence
from copy import copy
from dataclasses import dataclass

from solders.instruction import Instruction
from solders.pubkey import Pubkey

from .auth import AuthCredentials
from .auth.client import Auth
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
from .env import LightconeEnv
from .error import InsufficientSolForTransactionFees, SdkError
from .http.client import DEFAULT_TIMEOUT_SECS, LightconeHttp
from .http.credential_restorer import CredentialRestorer
from .privy.client import Privy
from .program.transaction import V1ResourceConfig, V1Transaction, V1TransactionContext
from .rpc import Rpc
from .rpc_failover import (
    FAST_RETRY_DELAY_SECS,
    ActiveRpc,
    RpcFailoverState,
    is_infrastructure_error,
)
from .shared.signing import (
    ExternalSigner,
    SigningStrategy,
    SigningStrategyKind,
    classify_signer_error,
)
from .shared.types import DepositSource
from .ws import WS_DEFAULT_CONFIG, WsConfig
from .ws.client import WsClient


@dataclass(frozen=True)
class ConfirmedTransaction:
    """A confirmed transaction signature and its processing slot."""

    #: Base58 transaction signature accepted by the cluster.
    signature: str
    #: Slot whose confirmed status authorizes a freshness-bounded state refresh.
    slot: int


class LightconeClient:
    """High-level client providing access to all Lightcone SDK sub-clients.

    Use LightconeClientBuilder to construct instances.

    Caching philosophy: The SDK is stateless for HTTP data. Caching is the
    consumer's responsibility.
    """

    def __init__(
        self,
        http: LightconeHttp,
        ws_config: WsConfig | None = None,
        auth_credentials: AuthCredentials | None = None,
        program_id: Pubkey | None = None,
        connection: object | None = None,
        backup_connection: object | None = None,
        deposit_source: DepositSource = DepositSource.GLOBAL,
        signing_strategy: SigningStrategy | None = None,
        primary_rpc_url: str | None = None,
        backup_rpc_url: str | None = None,
        transaction_sponsorship_enabled: bool = False,
        transaction_resources: V1ResourceConfig | None = None,
    ):
        if transaction_resources is not None and not isinstance(
            transaction_resources, V1ResourceConfig
        ):
            raise SdkError("explicit V1ResourceConfig is required")
        self._transaction_resources = transaction_resources
        self._http = http
        self._ws_config = ws_config or WS_DEFAULT_CONFIG
        self._program_id: Pubkey = program_id or LightconeEnv.PROD.program_id
        self._primary_connection = connection  # Optional[AsyncClient]
        self._backup_connection = backup_connection  # Optional[AsyncClient]
        self._rpc_failover_state = RpcFailoverState()
        self._deposit_source: DepositSource = deposit_source
        self._signing_strategy: SigningStrategy | None = signing_strategy
        self._transaction_sponsorship_enabled = transaction_sponsorship_enabled
        self._primary_rpc_url: str | None = primary_rpc_url
        self._backup_rpc_url: str | None = backup_rpc_url
        self._order_nonce: int | None = None

        # Sub-clients (all take self reference)
        self._markets = Markets(self)
        self._orders = Orders(self)
        self._orderbooks = Orderbooks(self)
        self._positions = Positions(self)
        self._trades = Trades(self)
        self._price_history = PriceHistoryClient(self)
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
    def connection(self) -> object | None:
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
        self, override_source: DepositSource | None = None
    ) -> DepositSource:
        """Resolve deposit source: per-call override > client setting."""
        return override_source if override_source is not None else self._deposit_source

    # ── Signing strategy ───────────────────────────────────────────────

    @property
    def signing_strategy(self) -> SigningStrategy | None:
        """Get the current signing strategy, if set."""
        return self._signing_strategy

    @signing_strategy.setter
    def signing_strategy(self, strategy: SigningStrategy | None) -> None:
        """Set the signing strategy at runtime."""
        self._signing_strategy = strategy

    def set_signing_strategy(self, strategy: SigningStrategy) -> None:
        """Set the signing strategy at runtime."""
        self._signing_strategy = strategy

    def clear_signing_strategy(self) -> None:
        """Clear the signing strategy (e.g. on logout)."""
        self._signing_strategy = None

    @property
    def transaction_sponsorship_enabled(self) -> bool:
        """Return the trusted assertion that an external sponsor pays fees."""
        return self._transaction_sponsorship_enabled

    @transaction_sponsorship_enabled.setter
    def transaction_sponsorship_enabled(self, enabled: bool) -> None:
        """Replace the client-wide Transaction Sponsorship Capability."""
        self._transaction_sponsorship_enabled = enabled

    def set_transaction_sponsorship_enabled(self, enabled: bool) -> None:
        """Replace the capability used by subsequent shared submissions."""
        self._transaction_sponsorship_enabled = enabled

    # ── Nonce cache ───────────────────────────────────────────────────

    @property
    def order_nonce(self) -> int | None:
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

    def _require_transaction_signing_context(self) -> tuple[SigningStrategy, bool]:
        """Capture one signer and sponsorship assertion before async transaction work."""
        return (
            copy(self._require_signing_strategy()),
            self._transaction_sponsorship_enabled,
        )

    def _validate_transaction_fee_funding_context(
        self,
        fee_payer: Pubkey,
        strategy: SigningStrategy,
        sponsorship_enabled: bool,
    ) -> None:
        """Reject invalid payer and sponsorship combinations before async work.

        Unsponsored known signers must control the payer being classified. Sponsored
        external flows may use a different payer, while native sponsorship is rejected
        before blockhash RPC or caller-transaction mutation.
        """
        if sponsorship_enabled:
            if strategy.kind == SigningStrategyKind.NATIVE:
                raise SdkError(
                    "transaction sponsorship is not supported with local-keypair signing"
                )
            return
        signing_address = strategy.controlled_wallet_address()
        if signing_address is not None and signing_address != str(fee_payer):
            raise SdkError("signing strategy does not control transaction fee payer")

    def _validate_transaction_signing_context(
        self,
        fee_payer: Pubkey,
        strategy: SigningStrategy,
        sponsorship_enabled: bool,
    ) -> None:
        """Validate local signer authority before acquiring transaction context."""
        if strategy.kind == SigningStrategyKind.PRIVY:
            raise SdkError(
                "Privy transaction signing cannot return verifiable v1 signed bytes; use a v1-capable external signer"
            )
        if not sponsorship_enabled and strategy.controlled_wallet_address() is None:
            raise SdkError("signing strategy wallet identity is required")
        self._validate_transaction_fee_funding_context(
            fee_payer, strategy, sponsorship_enabled
        )

    async def _sign_and_submit_instructions(
        self, instructions: Sequence[Instruction], payer: Pubkey
    ) -> str:
        """Capture validated builder inputs and signing authority before RPC work."""
        instructions = tuple(instructions)
        strategy, sponsorship_enabled = self._require_transaction_signing_context()
        self._validate_transaction_signing_context(payer, strategy, sponsorship_enabled)
        context = await self.transaction_context()
        transaction = V1Transaction.compile(instructions, payer, context)
        signature, _height = await self._sign_and_submit_tx_inner(
            transaction, strategy, sponsorship_enabled
        )
        return signature

    async def _preflight_transaction_fee_funding(
        self,
        tx: V1Transaction,
        strategy: SigningStrategy,
        sponsorship_enabled: bool,
    ) -> None:
        """Reject proven fee shortfalls before signing and continue on unknown evidence.

        The transaction's prepared message supplies the exact fee and declared fee
        payer. Fee or balance lookup failure is deliberately best-effort; planner-owned
        SOL admission remains fail-closed before reaching this shared boundary. The
        signer and sponsorship value were captured together before RPC work.
        """
        self._validate_transaction_fee_funding_context(
            tx.message.account_keys[0], strategy, sponsorship_enabled
        )
        if sponsorship_enabled:
            return
        fee_payer = tx.message.account_keys[0]

        try:
            required_lamports = await self.rpc().estimate_prepared_transaction_fee(tx)
        except Exception:
            return
        try:
            available_lamports = await self.rpc().balance_lamports(fee_payer)
        except Exception:
            return
        if available_lamports < required_lamports:
            raise InsufficientSolForTransactionFees(
                available_lamports, required_lamports
            )

    async def transaction_context_with_resources(
        self, resources: V1ResourceConfig
    ) -> V1TransactionContext:
        """Fetch one blockhash/expiry pair for explicit validated resources."""
        if not isinstance(resources, V1ResourceConfig):
            raise SdkError("explicit V1ResourceConfig is required")
        blockhash, height = await self.rpc().get_latest_blockhash_with_height()
        return V1TransactionContext(blockhash, height, resources)

    async def transaction_context(self) -> V1TransactionContext:
        """Fetch context using caller-configured resources; no implicit budgets."""
        if self._transaction_resources is None:
            raise SdkError(
                "transaction resources are required; configure them on the client builder"
            )
        return await self.transaction_context_with_resources(
            self._transaction_resources
        )

    async def sign_and_submit_tx(self, tx: V1Transaction) -> str:
        """Sign the exact v1 message, simulate, and send once with preflight."""
        signature, _height = await self._sign_and_submit_tx_inner(tx)
        return signature

    async def sign_and_submit_tx_confirmed(self, tx: V1Transaction) -> str:
        """Submit v1 and confirm using its original blockhash expiry."""
        return (await self.sign_and_submit_tx_confirmed_with_slot(tx)).signature

    async def sign_and_submit_tx_confirmed_with_slot(
        self, tx: V1Transaction
    ) -> ConfirmedTransaction:
        """Submit the exact message and return its confirmed processing slot."""
        signature, height = await self._sign_and_submit_tx_inner(tx)
        status = await self.rpc().confirm_signature_status(signature, height)
        return ConfirmedTransaction(signature, status.slot)

    async def sign_and_submit_prepared_tx_confirmed_with_slot(
        self, tx: V1Transaction
    ) -> ConfirmedTransaction:
        """Confirm a fee-prepared v1 message without changing any message bytes."""
        return await self.sign_and_submit_tx_confirmed_with_slot(tx)

    async def _sign_and_submit_tx_confirmed_with_strategy(
        self, tx: V1Transaction, strategy: SigningStrategy
    ) -> str:
        """Confirm with a strategy already validated by the calling operation."""
        signature, height = await self._sign_and_submit_tx_inner(tx, strategy)
        await self.rpc().confirm_signature_status(signature, height)
        return signature

    async def _sign_and_submit_tx_inner(
        self,
        tx: V1Transaction,
        strategy: SigningStrategy | None = None,
        sponsorship_enabled: bool | None = None,
    ) -> tuple[str, int]:
        """Preserve message authority through funding, signing, and one send."""
        if not isinstance(tx, V1Transaction):
            raise SdkError("only validated Solana v1 transactions are supported")
        if strategy is None:
            strategy, sponsorship_enabled = self._require_transaction_signing_context()
        elif sponsorship_enabled is None:
            sponsorship_enabled = self._transaction_sponsorship_enabled
        strategy = copy(strategy)
        self._validate_transaction_signing_context(
            tx.message.account_keys[0], strategy, bool(sponsorship_enabled)
        )
        await self._preflight_transaction_fee_funding(
            tx, strategy, bool(sponsorship_enabled)
        )
        await self.rpc().ensure_v1_supported()
        if strategy.kind == SigningStrategyKind.NATIVE:
            signed = tx.sign([strategy.keypair])
        elif strategy.kind == SigningStrategyKind.WALLET_ADAPTER:
            if strategy.signer is None:
                raise SdkError("external signer is required")
            try:
                wire = await strategy.signer.sign_transaction(tx.to_wire_bytes())
            except Exception as error:
                raise classify_signer_error(str(error)) from error
            signed = tx.accept_signed_bytes(wire)
        else:
            raise SdkError(f"Unsupported signing strategy: {strategy.kind}")
        signature = await self.rpc().submit_signed_transaction(signed)
        return signature, tx.context.last_valid_block_height

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
    def auth_token(self) -> str | None:
        """Current ``auth_token`` cookie value, if any.

        Populated by the SDK after a successful login, then attached on
        every authed request. Useful for forwarding the token through
        ``*_with_cookies`` methods or persisting the session across
        processes.
        """
        return self._http.auth_token

    def clear_auth_token(self) -> None:
        """Clear the cached ``auth_token``.

        Subsequent authed calls will go out without a ``Cookie`` header
        (and 401) unless they use a ``*_with_cookies`` variant.
        """
        self._http.clear_auth_token()

    def set_credential_restorer(self, restorer: CredentialRestorer) -> None:
        """Register the credential restorer consulted when a request 401s.

        The restorer attempts to restore credentials (e.g. re-run a login so
        the auth cookie is valid again); on success the transport replays the
        request once IF it declared itself retry-safe (``RetryPolicy.NONE``
        mutations are never auto-replayed). See :mod:`lightcone_sdk.http.credential_restorer`.
        Without a restorer, 401s propagate to callers unchanged.

        Common use: set once at app startup, alongside the signing strategy.
        """
        self._http.set_credential_restorer(restorer)

    def clear_credential_restorer(self) -> None:
        """Remove the credential restorer (e.g. in tests); 401s propagate again."""
        self._http.clear_credential_restorer()

    async def close(self) -> None:
        """Close the HTTP session."""
        await self._http.close()

    async def __aenter__(self) -> LightconeClient:
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        await self.close()


class LightconeClientBuilder:
    """Builder for constructing LightconeClient instances."""

    def __init__(self):
        environment = LightconeEnv.PROD
        self._base_url: str = environment.api_url
        self._ws_url: str = environment.ws_url
        self._auth_credentials: AuthCredentials | None = None
        self._ws_config: WsConfig | None = None
        self._timeout: int = DEFAULT_TIMEOUT_SECS
        self._program_id: Pubkey | None = environment.program_id
        self._deposit_source: DepositSource = DepositSource.GLOBAL
        self._signing_strategy: SigningStrategy | None = None
        self._transaction_sponsorship_enabled = False
        self._transaction_resources: V1ResourceConfig | None = None
        self._primary_rpc_url: str | None = environment.rpc_url
        self._backup_rpc_url: str | None = None
        self._connection: object | None = None

    def env(self, environment: LightconeEnv) -> LightconeClientBuilder:
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

    def base_url(self, url: str) -> LightconeClientBuilder:
        self._base_url = url
        return self

    def ws_url(self, url: str) -> LightconeClientBuilder:
        self._ws_url = url
        return self

    def auth(self, credentials: AuthCredentials) -> LightconeClientBuilder:
        self._auth_credentials = credentials
        return self

    def ws_config(self, config: WsConfig) -> LightconeClientBuilder:
        self._ws_config = config
        return self

    def timeout(self, timeout: int) -> LightconeClientBuilder:
        self._timeout = timeout
        return self

    def program_id(self, pid: Pubkey) -> LightconeClientBuilder:
        """Set a custom on-chain program ID (defaults to canonical Lightcone program)."""
        self._program_id = pid
        return self

    def deposit_source(self, source: DepositSource) -> LightconeClientBuilder:
        """Set the default deposit source for orders, deposits, and withdrawals.

        Defaults to ``DepositSource.GLOBAL``. Can be overridden per-call.
        """
        self._deposit_source = source
        return self

    def native_signer(self, keypair: object) -> LightconeClientBuilder:
        """Set a native keypair for signing orders, cancels, and transactions."""
        self._signing_strategy = SigningStrategy.native(keypair)
        return self

    def external_signer(self, signer: ExternalSigner) -> LightconeClientBuilder:
        """Set an external signer for browser wallet adapters."""
        self._signing_strategy = SigningStrategy.wallet_adapter(signer)
        return self

    def privy_wallet_id(
        self, wallet_id: str, wallet_address: str | None = None
    ) -> LightconeClientBuilder:
        """Set a Privy embedded wallet ID for signing."""
        self._signing_strategy = SigningStrategy.privy(wallet_id, wallet_address)
        return self

    def transaction_resources(
        self, resources: V1ResourceConfig
    ) -> LightconeClientBuilder:
        """Choose inline compute/account limits and total priority fee lamports."""
        if not isinstance(resources, V1ResourceConfig):
            raise SdkError("explicit V1ResourceConfig is required")
        self._transaction_resources = resources
        return self

    def transaction_sponsorship(self, enabled: bool) -> LightconeClientBuilder:
        """Set the initial trusted Transaction Sponsorship Capability.

        The capability defaults to false.
        """
        self._transaction_sponsorship_enabled = enabled
        return self

    def rpc_url(self, url: str) -> LightconeClientBuilder:
        """Set the primary Solana RPC URL for blockhash fetching, transaction submission, and on-chain reads."""
        self._primary_rpc_url = url
        return self

    def backup_rpc_url(self, url: str) -> LightconeClientBuilder:
        """Set a backup Solana RPC URL for automatic failover."""
        self._backup_rpc_url = url
        return self

    def rpc_connection(self, connection: object) -> LightconeClientBuilder:
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
            from solana.rpc.commitment import Confirmed

            connection = AsyncClient(
                self._primary_rpc_url, commitment=Confirmed, max_transport_retries=0
            )

        backup_connection = None
        if self._backup_rpc_url is not None:
            from solana.rpc.async_api import AsyncClient
            from solana.rpc.commitment import Confirmed

            backup_connection = AsyncClient(
                self._backup_rpc_url, commitment=Confirmed, max_transport_retries=0
            )

        return LightconeClient(
            http=http,
            ws_config=ws_config,
            auth_credentials=self._auth_credentials,
            program_id=self._program_id,
            connection=connection,
            backup_connection=backup_connection,
            deposit_source=self._deposit_source,
            signing_strategy=self._signing_strategy,
            transaction_sponsorship_enabled=self._transaction_sponsorship_enabled,
            transaction_resources=self._transaction_resources,
            primary_rpc_url=self._primary_rpc_url,
            backup_rpc_url=self._backup_rpc_url,
        )


__all__ = [
    "LightconeClient",
    "LightconeClientBuilder",
]
