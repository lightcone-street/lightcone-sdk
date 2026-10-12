"""Orders sub-client — submit, cancel, query, PDA helpers, order helpers, tx builders, and on-chain ops."""

from __future__ import annotations

from typing import TYPE_CHECKING

from solders.instruction import Instruction
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from ...error import SigningError, ValidationError
from ...program.accounts import deserialize_order_status
from ...program.envelope import LimitOrderEnvelope
from ...program.errors import InvalidSideError, InvalidSignatureError
from ...program.instructions import (
    build_cancel_order_instruction,
    build_close_order_status_instruction,
)
from ...program.orders import (
    create_ask_order,
    create_bid_order,
    create_signed_ask_order,
    create_signed_bid_order,
    generate_cancel_all_salt,
    hash_order,
    sign_order,
)
from ...program.pda import get_order_status_pda
from ...program.transaction import V1Transaction, V1TransactionContext
from ...program.types import (
    AskOrderParams,
    BidOrderParams,
    CloseOrderStatusParams,
    OrderSide,
    OrderStatus,
    SignedOrder,
)
from ...program.utils import encode_i64, encode_u64
from ...rpc import require_connection
from ...shared.query import query_u32
from ...shared.scaling import (
    OrderbookRules,
    validate_raw_amounts,
)
from ...shared.types import (
    SubmitOrderRequest,
)
from . import (
    CancelAllBody,
    CancelAllSuccess,
    CancelBody,
    CancelSuccess,
    SubmitOrderResponse,
    UserOrderFillsResponse,
    UserOrdersResponse,
)

if TYPE_CHECKING:
    from ...client import LightconeClient

_PRIVY_ORDERS_UNSUPPORTED = (
    "Privy order signing is not supported; configure a native signer or an "
    "ExternalSigner"
)


class Orders:
    """Order operations sub-client."""

    def __init__(self, client: LightconeClient):
        self._client = client

    # ── PDA helpers ──────────────────────────────────────────────────────

    def status_pda(self, order_hash: bytes) -> Pubkey:
        """Get the Order Status PDA."""
        addr, _ = get_order_status_pda(order_hash, self._client.program_id)
        return addr

    # ── Order helpers ────────────────────────────────────────────────────

    def create_bid_order(self, params: BidOrderParams) -> SignedOrder:
        """Create an unsigned bid order."""
        return create_bid_order(params)

    def create_ask_order(self, params: AskOrderParams) -> SignedOrder:
        """Create an unsigned ask order."""
        return create_ask_order(params)

    def create_signed_bid_order(
        self, params: BidOrderParams, keypair: Keypair, rules: OrderbookRules
    ) -> SignedOrder:
        """Create and sign a bid order."""
        return create_signed_bid_order(params, keypair, rules)

    def create_signed_ask_order(
        self, params: AskOrderParams, keypair: Keypair, rules: OrderbookRules
    ) -> SignedOrder:
        """Create and sign an ask order."""
        return create_signed_ask_order(params, keypair, rules)

    def hash_order(self, order: SignedOrder) -> bytes:
        """Compute the keccak256 hash of an order."""
        return hash_order(order)

    def sign_order(
        self, order: SignedOrder, keypair: Keypair, rules: OrderbookRules
    ) -> bytes:
        """Sign an order with a keypair."""
        return sign_order(order, keypair, rules)

    def generate_cancel_all_salt(self) -> str:
        """Generate a random salt for cancel-all replay protection."""
        return generate_cancel_all_salt()

    # ── Envelope factories ────────────────────────────────────────────────

    def limit_order(self) -> LimitOrderEnvelope:
        """Create a LimitOrderEnvelope pre-seeded with the client's deposit source.

        Users can still override the deposit source on the returned envelope
        by calling ``.deposit_source()`` before signing.
        """
        return LimitOrderEnvelope().deposit_source(self._client.deposit_source)

    # ── HTTP methods ─────────────────────────────────────────────────────

    async def submit(self, request: SubmitOrderRequest) -> SubmitOrderResponse:
        """Submit a signed limit order. Not retried.

        An ``ENGINE_UNAVAILABLE`` error means the outcome is unknown. Resolve
        it by submitting the identical signed ``request`` again: a
        ``DUPLICATE_ORDER`` rejection or an ``ALREADY_EXISTS`` error proves
        the first submission was accepted; any other result is the outcome of
        this one. Never sign a replacement with a new salt, which is a
        different order. Order reads list only open and pending orders, so
        they cannot prove an order was never accepted.
        """
        await self._preflight_submit(request)
        data = await self._client._http.post("/api/orders/submit", request.to_dict())
        return SubmitOrderResponse.from_dict(data)

    async def cancel(self, body: CancelBody) -> CancelSuccess:
        """Cancel one order's open remainder. Not retried."""
        data = await self._client._http.post("/api/orders/cancel", body.to_dict())
        return CancelSuccess.from_dict(data)

    async def cancel_all(self, body: CancelAllBody) -> CancelAllSuccess:
        """Close every order of the wallet (optionally on one orderbook)
        accepted up to a committed cutoff. Not retried.

        ``body.timestamp`` must be within 30 seconds of server time, and
        ``body.salt`` non-empty, at most 128 bytes, and never reused.
        """
        data = await self._client._http.post("/api/orders/cancel-all", body.to_dict())
        return CancelAllSuccess.from_dict(data)

    async def _preflight_submit(self, request: SubmitOrderRequest) -> None:
        """Check the signed fields against current orderbook rules before submission.

        The side must be BID or ASK, the amounts must form an exact admitted
        ratio, and the salt and expiration must fit the order's ``u64`` and
        ``i64`` fields, which the Rust SDK's types guarantee.
        """
        rules = await self._client.orderbooks().decimals(request.orderbook_id)
        if (
            isinstance(request.side, bool)
            or not isinstance(request.side, int)
            or request.side not in (OrderSide.BID, OrderSide.ASK)
        ):
            raise InvalidSideError(request.side)
        validate_raw_amounts(request.amount_in, request.amount_out, request.side, rules)
        encode_u64(request.salt)
        encode_i64(request.expiration)

    async def get_user_orders(
        self,
        limit: int | None = None,
        cursor: str | None = None,
    ) -> UserOrdersResponse:
        """Fetch the authenticated user's open and pending orders, with a first
        page of funding accounts.

        The wallet is resolved server-side from the auth cookie. Open orders
        are JWT-only; there is intentionally no public path-based variant.
        ``limit`` defaults to 200 and is clamped to 1..256 server-side;
        ``cursor`` is a previous ``next_cursor``.
        """
        data = await self._client._http.get(
            "/api/users/orders", params=_user_orders_params(limit, cursor)
        )
        return UserOrdersResponse.from_dict(data)

    async def get_user_orders_with_cookies(
        self,
        limit: int | None,
        cursor: str | None,
        cookie_header: str,
    ) -> UserOrdersResponse:
        """Same as :meth:`get_user_orders`, with an explicit per-call ``cookie_header``.

        Intended for server-side cookie forwarding (SSR / route handlers)
        where the per-request browser cookie can't propagate to the SDK's
        process-wide cookie store. The token is used only for this call and
        never written back to the shared store.
        """
        data = await self._client._http.get_with_cookies(
            "/api/users/orders",
            cookie_header=cookie_header,
            params=_user_orders_params(limit, cursor),
        )
        return UserOrdersResponse.from_dict(data)

    async def get_user_order_fills(
        self,
        market_pubkey: str | None = None,
        limit: int | None = None,
        cursor: str | None = None,
        fills: int | None = None,
    ) -> UserOrderFillsResponse:
        """Fetch the authenticated user's filled orders with nested fill events.

        The wallet is resolved server-side from the auth cookie. Includes
        orders where the user was either maker or taker. Optionally filter by
        market. Orders are sorted by most recent fill first; each carries its
        oldest-first page of ``fills`` fills (default and maximum 16; ``0``
        reads none and only reports ``fills_has_more``). ``limit`` defaults to
        20 and is clamped to 1..100 server-side.
        """
        data = await self._client._http.get(
            "/api/users/order-fills",
            params=_order_fills_params(
                market_pubkey=market_pubkey, limit=limit, cursor=cursor, fills=fills
            ),
        )
        return UserOrderFillsResponse.from_dict(data)

    async def get_user_order_fills_with_cookies(
        self,
        market_pubkey: str | None,
        limit: int | None,
        cursor: str | None,
        cookie_header: str,
        *,
        fills: int | None = None,
    ) -> UserOrderFillsResponse:
        """Same as :meth:`get_user_order_fills`, with an explicit per-call ``cookie_header``.

        Intended for server-side cookie forwarding (SSR / route handlers)
        where the per-request browser cookie can't propagate to the SDK's
        process-wide cookie store. The token is used only for this call and
        never written back to the shared store.
        """
        data = await self._client._http.get_with_cookies(
            "/api/users/order-fills",
            cookie_header=cookie_header,
            params=_order_fills_params(
                market_pubkey=market_pubkey, limit=limit, cursor=cursor, fills=fills
            ),
        )
        return UserOrderFillsResponse.from_dict(data)

    async def get_user_order_fills_by_wallet(
        self,
        wallet_address: str,
        market_pubkey: str | None = None,
        limit: int | None = None,
        cursor: str | None = None,
        fills: int | None = None,
    ) -> UserOrderFillsResponse:
        """Public variant of :meth:`get_user_order_fills`.

        Takes the user's wallet via the URL path
        (``GET /api/users/{wallet}/order-fills``) and requires no auth.
        """
        data = await self._client._http.get(
            f"/api/users/{wallet_address}/order-fills",
            params=_order_fills_params(
                market_pubkey=market_pubkey, limit=limit, cursor=cursor, fills=fills
            ),
        )
        return UserOrderFillsResponse.from_dict(data)

    async def get_order_fill_page(
        self,
        order_hash: str,
        fill_cursor: str | None = None,
        fills: int | None = None,
    ) -> UserOrderFillsResponse:
        """Fetch one order of the authenticated user with the next page of its fills.

        Pass the order's ``fills_next_cursor`` (or a submission's
        ``fills_next_cursor``) as ``fill_cursor``; ``None`` starts at the
        first fill. ``fills`` is the page size (default and maximum 16; a
        ``fill_cursor`` needs at least 1). ``order_hash`` must be 64 lowercase
        hex characters.
        """
        data = await self._client._http.get(
            "/api/users/order-fills",
            params=_order_fills_params(
                order_hash=order_hash, fill_cursor=fill_cursor, fills=fills
            ),
        )
        return UserOrderFillsResponse.from_dict(data)

    async def get_order_fill_page_with_cookies(
        self,
        order_hash: str,
        fill_cursor: str | None,
        cookie_header: str,
        *,
        fills: int | None = None,
    ) -> UserOrderFillsResponse:
        """Same as :meth:`get_order_fill_page`, forwarding the supplied raw
        ``Cookie`` header for server-side cookie forwarding."""
        data = await self._client._http.get_with_cookies(
            "/api/users/order-fills",
            cookie_header=cookie_header,
            params=_order_fills_params(
                order_hash=order_hash, fill_cursor=fill_cursor, fills=fills
            ),
        )
        return UserOrderFillsResponse.from_dict(data)

    async def get_order_fill_page_by_wallet(
        self,
        wallet_address: str,
        order_hash: str,
        fill_cursor: str | None = None,
        fills: int | None = None,
    ) -> UserOrderFillsResponse:
        """Public variant of :meth:`get_order_fill_page` for any wallet."""
        data = await self._client._http.get(
            f"/api/users/{wallet_address}/order-fills",
            params=_order_fills_params(
                order_hash=order_hash, fill_cursor=fill_cursor, fills=fills
            ),
        )
        return UserOrderFillsResponse.from_dict(data)

    # ── Unified cancel (dispatches based on client signing strategy) ────

    async def cancel_order_signed(
        self,
        order_hash: str,
        maker: str,
    ) -> CancelSuccess:
        """Cancel an order using the client's signing strategy."""
        from ...program.orders import cancel_order_message, sign_cancel_order
        from ...shared.signing import SigningStrategyKind, classify_signer_error

        strategy = self._client._require_signing_strategy()

        if strategy.kind == SigningStrategyKind.NATIVE:
            body = CancelBody(
                order_hash=order_hash,
                maker=maker,
                signature=sign_cancel_order(order_hash, strategy.keypair),
            )
            return await self.cancel(body)

        elif strategy.kind == SigningStrategyKind.WALLET_ADAPTER:
            message = cancel_order_message(order_hash)
            try:
                sig_bytes = await strategy.signer.sign_message(message)
            except Exception as exc:
                raise classify_signer_error(str(exc)) from exc

            sig_hex = _signature_hex(sig_bytes)
            body = CancelBody(order_hash=order_hash, maker=maker, signature=sig_hex)
            return await self.cancel(body)

        elif strategy.kind == SigningStrategyKind.PRIVY:
            raise ValidationError(_PRIVY_ORDERS_UNSUPPORTED)

        raise SigningError(f"Unsupported signing strategy: {strategy.kind}")

    async def cancel_all_signed(
        self,
        user_pubkey: str,
        timestamp: int,
        salt: str,
        orderbook_id: str | None = None,
    ) -> CancelAllSuccess:
        """Cancel all orders using the client's signing strategy."""
        from ...program.orders import cancel_all_message, sign_cancel_all
        from ...shared.signing import SigningStrategyKind, classify_signer_error

        strategy = self._client._require_signing_strategy()
        resolved_ob_id = orderbook_id or ""

        if strategy.kind == SigningStrategyKind.NATIVE:
            body = CancelAllBody(
                user_pubkey=user_pubkey,
                orderbook_id=resolved_ob_id,
                signature=sign_cancel_all(
                    user_pubkey, resolved_ob_id, timestamp, salt, strategy.keypair
                ),
                timestamp=timestamp,
                salt=salt,
            )
            return await self.cancel_all(body)

        elif strategy.kind == SigningStrategyKind.WALLET_ADAPTER:
            message = cancel_all_message(user_pubkey, resolved_ob_id, timestamp, salt)
            try:
                sig_bytes = await strategy.signer.sign_message(message.encode())
            except Exception as exc:
                raise classify_signer_error(str(exc)) from exc
            sig_hex = _signature_hex(sig_bytes)
            body = CancelAllBody(
                user_pubkey=user_pubkey,
                orderbook_id=resolved_ob_id,
                signature=sig_hex,
                timestamp=timestamp,
                salt=salt,
            )
            return await self.cancel_all(body)

        elif strategy.kind == SigningStrategyKind.PRIVY:
            raise ValidationError(_PRIVY_ORDERS_UNSUPPORTED)

        raise SigningError(f"Unsupported signing strategy: {strategy.kind}")

    # ── On-chain instruction builders ────────────────────────────────────

    def cancel_order_ix(
        self, operator: Pubkey, market: Pubkey, order: SignedOrder
    ) -> Instruction:
        """Build CancelOrder instruction (on-chain cancellation)."""
        return build_cancel_order_instruction(
            operator, market, order, self._client.program_id
        )

    def close_order_status_ix(
        self,
        params: CloseOrderStatusParams,
    ) -> Instruction:
        """Build CloseOrderStatus instruction."""
        return build_close_order_status_instruction(params, self._client.program_id)

    # ── On-chain transaction builders ────────────────────────────────────

    def cancel_order_tx(
        self,
        operator: Pubkey,
        market: Pubkey,
        order: SignedOrder,
        context: V1TransactionContext,
    ) -> V1Transaction:
        """Build CancelOrder transaction."""
        ix = self.cancel_order_ix(operator, market, order)
        return V1Transaction.compile([ix], operator, context)

    def close_order_status_tx(
        self, params: CloseOrderStatusParams, context: V1TransactionContext
    ) -> V1Transaction:
        """Build CloseOrderStatus transaction."""
        ix = self.close_order_status_ix(params)
        return V1Transaction.compile([ix], params.operator, context)

    # ── On-chain account fetchers (require connection) ───────────────────

    async def get_status(self, order_hash: bytes) -> OrderStatus | None:
        """Fetch an OrderStatus account (returns None if not found)."""
        conn = require_connection(self._client)
        addr = self.status_pda(order_hash)
        response = await conn.get_account_info(addr)
        if response.value is None:
            return None
        return deserialize_order_status(response.value.data)


def _signature_hex(signature: bytes) -> str:
    """Hex of an Ed25519 signature returned by an external signer; anything
    but 64 bytes raises ``InvalidSignatureError``, as the Rust SDK does."""
    if not isinstance(signature, (bytes, bytearray)) or len(signature) != 64:
        raise InvalidSignatureError("Signature must be exactly 64 bytes")
    return bytes(signature).hex()


def _user_orders_params(limit: int | None, cursor: str | None) -> dict[str, str] | None:
    params: dict[str, str] = {}
    if limit is not None:
        params["limit"] = query_u32("limit", limit)
    if cursor is not None:
        params["cursor"] = cursor
    return params or None


def _order_fills_params(
    *,
    market_pubkey: str | None = None,
    limit: int | None = None,
    cursor: str | None = None,
    order_hash: str | None = None,
    fill_cursor: str | None = None,
    fills: int | None = None,
) -> dict[str, str] | None:
    """Query for the order-fills routes.

    Only the parameters given are sent (``fills=0`` included): the backend
    rejects unknown parameters, combining ``cursor`` with ``order_hash``, and
    a ``fill_cursor`` with ``fills`` of ``0``.
    """
    params: dict[str, str] = {}
    if market_pubkey is not None:
        params["market_pubkey"] = market_pubkey
    if limit is not None:
        params["limit"] = query_u32("limit", limit)
    if cursor is not None:
        params["cursor"] = cursor
    if order_hash is not None:
        params["order_hash"] = order_hash
    if fill_cursor is not None:
        params["fill_cursor"] = fill_cursor
    if fills is not None:
        params["fills"] = query_u32("fills", fills)
    return params or None
