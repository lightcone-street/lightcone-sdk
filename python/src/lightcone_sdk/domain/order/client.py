"""Orders sub-client — submit, cancel, query, PDA helpers, order helpers, tx builders, and on-chain ops."""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING

from solders.instruction import Instruction
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solders.transaction import Transaction

from . import (
    SubmitOrderResponse,
    CancelBody,
    CancelSuccess,
    CancelAllBody,
    CancelAllSuccess,
    CancelTriggerBody,
    CancelTriggerSuccess,
    TriggerOrderResponse,
    UserOrdersResponse,
    UserSnapshotOrder,
    UserSnapshotBalance,
)
from .convert import submit_response_from_dict
from ...error import SdkError, SigningError
from ...program.accounts import deserialize_order_status, deserialize_user_nonce
from ...program.errors import ArithmeticOverflowError
from ...program.envelope import LimitOrderEnvelope, TriggerOrderEnvelope
from ...program.instructions import (
    build_cancel_order_instruction,
    build_increment_nonce_instruction,
)
from ...program.orders import (
    create_ask_order as _create_ask_order,
    create_bid_order as _create_bid_order,
    create_signed_ask_order as _create_signed_ask_order,
    create_signed_bid_order as _create_signed_bid_order,
    generate_cancel_all_salt as _generate_cancel_all_salt,
    hash_order as _hash_order,
    sign_order as _sign_order,
)
from ...program.pda import get_order_status_pda, get_user_nonce_pda
from ...program.types import (
    AskOrderParams,
    BidOrderParams,
    OrderStatus as OnchainOrderStatus,
    SignedOrder,
)
from ...rpc import require_connection
from ...shared.types import (
    SubmitOrderRequest,
    SubmitTriggerOrderRequest,
)

if TYPE_CHECKING:
    from ...client import LightconeClient


class Orders:
    """Order operations sub-client."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    # ── PDA helpers ──────────────────────────────────────────────────────

    def status_pda(self, order_hash: bytes) -> Pubkey:
        """Get the Order Status PDA."""
        addr, _ = get_order_status_pda(order_hash, self._client.program_id)
        return addr

    def nonce_pda(self, user: Pubkey) -> Pubkey:
        """Get the User Nonce PDA."""
        addr, _ = get_user_nonce_pda(user, self._client.program_id)
        return addr

    # ── Order helpers ────────────────────────────────────────────────────

    def create_bid_order(self, params: BidOrderParams) -> SignedOrder:
        """Create an unsigned bid order."""
        return _create_bid_order(params)

    def create_ask_order(self, params: AskOrderParams) -> SignedOrder:
        """Create an unsigned ask order."""
        return _create_ask_order(params)

    def create_signed_bid_order(self, params: BidOrderParams, keypair: Keypair) -> SignedOrder:
        """Create and sign a bid order."""
        return _create_signed_bid_order(params, keypair)

    def create_signed_ask_order(self, params: AskOrderParams, keypair: Keypair) -> SignedOrder:
        """Create and sign an ask order."""
        return _create_signed_ask_order(params, keypair)

    def hash_order(self, order: SignedOrder) -> bytes:
        """Compute the keccak256 hash of an order."""
        return _hash_order(order)

    def sign_order(self, order: SignedOrder, keypair: Keypair) -> bytes:
        """Sign an order with a keypair."""
        return _sign_order(order, keypair)

    def generate_cancel_all_salt(self) -> str:
        """Generate a random salt for cancel-all replay protection."""
        return _generate_cancel_all_salt()

    # ── Envelope factories ────────────────────────────────────────────────

    def limit_order(self) -> LimitOrderEnvelope:
        """Create a LimitOrderEnvelope pre-seeded with the client's deposit source.

        Users can still override the deposit source on the returned envelope
        by calling ``.deposit_source()`` before signing.
        """
        return LimitOrderEnvelope().deposit_source(self._client.deposit_source)

    def trigger_order(self) -> TriggerOrderEnvelope:
        """Create a TriggerOrderEnvelope pre-seeded with the client's deposit source.

        Users can still override the deposit source on the returned envelope
        by calling ``.deposit_source()`` before signing.
        """
        return TriggerOrderEnvelope().deposit_source(self._client.deposit_source)

    # ── HTTP methods ─────────────────────────────────────────────────────

    async def submit(self, request: SubmitOrderRequest) -> SubmitOrderResponse:
        """Submit a limit order."""
        data = await self._client._http.post("/api/orders/submit", request.to_dict())
        place = _unwrap_status(
            data,
            success_statuses={"accepted", "partial_fill", "filled"},
            rejected_statuses={"rejected"},
        )
        return submit_response_from_dict(place)

    async def cancel(self, body: CancelBody) -> CancelSuccess:
        """Cancel a single order."""
        data = await self._client._http.post("/api/orders/cancel", body.to_dict())
        data = _unwrap_status(data, success_statuses={"cancelled"})
        return CancelSuccess(
            order_hash=data.get("order_hash", body.order_hash),
            remaining=data.get("remaining", 0),
        )

    async def cancel_all(self, body: CancelAllBody) -> CancelAllSuccess:
        """Cancel all orders for a user."""
        data = await self._client._http.post("/api/orders/cancel-all", body.to_dict())
        data = _unwrap_status(data, success_statuses={"success"})
        return CancelAllSuccess(
            cancelled_order_hashes=data.get("cancelled_order_hashes", []),
            count=data.get("count", 0),
            user_pubkey=data.get("user_pubkey", body.user_pubkey),
            orderbook_id=data.get("orderbook_id", body.orderbook_id or ""),
            message=data.get("message", ""),
        )

    async def submit_trigger(self, request: SubmitTriggerOrderRequest) -> TriggerOrderResponse:
        """Submit a trigger order."""
        data = await self._client._http.post("/api/orders/submit", request.to_dict())
        data = _unwrap_status(data, success_statuses={"accepted"})
        return TriggerOrderResponse(
            trigger_order_id=data.get("trigger_order_id", ""),
            order_hash=data.get("order_hash", ""),
        )

    async def cancel_trigger(self, body: CancelTriggerBody) -> CancelTriggerSuccess:
        """Cancel a trigger order."""
        data = await self._client._http.post("/api/orders/cancel", body.to_dict())
        data = _unwrap_status(data, success_statuses={"cancelled"})
        return CancelTriggerSuccess(
            trigger_order_id=data.get("trigger_order_id", body.trigger_order_id),
        )

    async def get_user_orders(
        self,
        wallet: str,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> UserOrdersResponse:
        """Get user's orders with pagination."""
        url = f"/api/users/orders?wallet_address={wallet}"
        if limit is not None:
            url += f"&limit={limit}"
        if cursor is not None:
            url += f"&cursor={cursor}"

        data = await self._client._http.get(url)

        return UserOrdersResponse(
            user_pubkey=data.get("user_pubkey", wallet),
            orders=[UserSnapshotOrder.from_dict(o) for o in data.get("orders", [])],
            balances=[UserSnapshotBalance.from_dict(b) for b in data.get("balances", [])],
            next_cursor=data.get("next_cursor"),
            has_more=data.get("has_more", False),
        )

    # ── Unified cancel (dispatches based on client signing strategy) ────

    async def cancel_order_signed(
        self, order_hash: str, maker: str,
    ) -> CancelSuccess:
        """Cancel an order using the client's signing strategy."""
        from ...shared.signing import SigningStrategyKind, classify_signer_error
        from ...program.orders import cancel_order_message, sign_cancel_order

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
            import bs58 as _bs58
            sig_hex = sig_bytes.hex()
            body = CancelBody(order_hash=order_hash, maker=maker, signature=sig_hex)
            return await self.cancel(body)

        elif strategy.kind == SigningStrategyKind.PRIVY:
            result = await self._client.privy().sign_and_cancel_order(
                strategy.wallet_id, order_hash, maker,
            )
            return CancelSuccess(
                order_hash=result.get("order_hash", order_hash),
                remaining=result.get("remaining", 0),
            )

        raise SigningError(f"Unsupported signing strategy: {strategy.kind}")

    async def cancel_all_signed(
        self,
        user_pubkey: str,
        timestamp: int,
        salt: str,
        orderbook_id: Optional[str] = None,
    ) -> CancelAllSuccess:
        """Cancel all orders using the client's signing strategy."""
        from ...shared.signing import SigningStrategyKind, classify_signer_error
        from ...program.orders import cancel_all_message, sign_cancel_all

        strategy = self._client._require_signing_strategy()
        resolved_ob_id = orderbook_id or ""

        if strategy.kind == SigningStrategyKind.NATIVE:
            body = CancelAllBody(
                user_pubkey=user_pubkey,
                orderbook_id=resolved_ob_id,
                signature=sign_cancel_all(user_pubkey, resolved_ob_id, timestamp, salt, strategy.keypair),
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
            sig_hex = sig_bytes.hex()
            body = CancelAllBody(
                user_pubkey=user_pubkey,
                orderbook_id=resolved_ob_id,
                signature=sig_hex,
                timestamp=timestamp,
                salt=salt,
            )
            return await self.cancel_all(body)

        elif strategy.kind == SigningStrategyKind.PRIVY:
            result = await self._client.privy().sign_and_cancel_all_orders(
                strategy.wallet_id, user_pubkey, resolved_ob_id, timestamp, salt,
            )
            return CancelAllSuccess(
                cancelled_order_hashes=result.get("cancelled_order_hashes", []),
                count=result.get("count", 0),
                user_pubkey=result.get("user_pubkey", user_pubkey),
                orderbook_id=result.get("orderbook_id", resolved_ob_id),
                message=result.get("message", ""),
            )

        raise SigningError(f"Unsupported signing strategy: {strategy.kind}")

    async def cancel_trigger_signed(
        self, trigger_order_id: str, maker: str,
    ) -> CancelTriggerSuccess:
        """Cancel a trigger order using the client's signing strategy."""
        from ...shared.signing import SigningStrategyKind, classify_signer_error
        from ...program.orders import cancel_trigger_order_message

        strategy = self._client._require_signing_strategy()

        if strategy.kind == SigningStrategyKind.NATIVE:
            message = cancel_trigger_order_message(trigger_order_id)
            from solders.keypair import Keypair as _Keypair
            keypair: _Keypair = strategy.keypair
            sig = keypair.sign_message(message)
            body = CancelTriggerBody(
                trigger_order_id=trigger_order_id,
                maker=maker,
                signature=bytes(sig).hex(),
            )
            return await self.cancel_trigger(body)

        elif strategy.kind == SigningStrategyKind.WALLET_ADAPTER:
            message = cancel_trigger_order_message(trigger_order_id)
            try:
                sig_bytes = await strategy.signer.sign_message(message)
            except Exception as exc:
                raise classify_signer_error(str(exc)) from exc
            sig_hex = sig_bytes.hex()
            body = CancelTriggerBody(
                trigger_order_id=trigger_order_id, maker=maker, signature=sig_hex,
            )
            return await self.cancel_trigger(body)

        elif strategy.kind == SigningStrategyKind.PRIVY:
            result = await self._client.privy().sign_and_cancel_trigger_order(
                strategy.wallet_id, trigger_order_id, maker,
            )
            return CancelTriggerSuccess(
                trigger_order_id=result.get("trigger_order_id", trigger_order_id),
            )

        raise SigningError(f"Unsupported signing strategy: {strategy.kind}")

    # ── On-chain instruction builders ────────────────────────────────────

    def cancel_order_ix(
        self, maker: Pubkey, market: Pubkey, order: SignedOrder
    ) -> Instruction:
        """Build CancelOrder instruction (on-chain cancellation)."""
        return build_cancel_order_instruction(
            maker, market, order, self._client.program_id
        )

    def increment_nonce_ix(self, user: Pubkey) -> Instruction:
        """Build IncrementNonce instruction."""
        return build_increment_nonce_instruction(user, self._client.program_id)

    # ── On-chain transaction builders ────────────────────────────────────

    def cancel_order_tx(
        self, maker: Pubkey, market: Pubkey, order: SignedOrder
    ) -> Transaction:
        """Build CancelOrder transaction."""
        ix = self.cancel_order_ix(maker, market, order)
        return Transaction.new_with_payer([ix], maker)

    def increment_nonce_tx(self, user: Pubkey) -> Transaction:
        """Build IncrementNonce transaction."""
        ix = self.increment_nonce_ix(user)
        return Transaction.new_with_payer([ix], user)

    # ── On-chain account fetchers (require connection) ───────────────────

    async def get_status(self, order_hash: bytes) -> Optional[OnchainOrderStatus]:
        """Fetch an OrderStatus account (returns None if not found)."""
        conn = require_connection(self._client)
        addr = self.status_pda(order_hash)
        response = await conn.get_account_info(addr)
        if response.value is None:
            return None
        return deserialize_order_status(response.value.data)

    async def get_nonce(self, user: Pubkey) -> int:
        """Fetch a user's current nonce (returns 0 if not initialized)."""
        conn = require_connection(self._client)
        addr = self.nonce_pda(user)
        response = await conn.get_account_info(addr)
        if response.value is None:
            return 0
        user_nonce = deserialize_user_nonce(response.value.data)
        return user_nonce.nonce

    async def current_nonce(self, user: Pubkey) -> int:
        """Get the current on-chain nonce for a user as u32."""
        nonce = await self.get_nonce(user)
        if nonce > 0xFFFFFFFF:
            raise ArithmeticOverflowError()
        return nonce


def _unwrap_status(
    data: dict,
    *,
    success_statuses: set[str],
    rejected_statuses: Optional[set[str]] = None,
) -> dict:
    status = data.get("status")
    if status is None or status in success_statuses:
        return data

    rejected_statuses = rejected_statuses or set()
    if status in rejected_statuses:
        error = data.get("error")
        details = data.get("details")
        if error and details:
            raise SdkError(f"{error}: {details}")
        if error:
            raise SdkError(error)
        if details:
            raise SdkError(details)

        parts = ["Rejected"]
        if data.get("order_hash"):
            parts.append(f"hash={data['order_hash']}")
        if data.get("filled") is not None:
            parts.append(f"filled={data['filled']}")
        if data.get("remaining") is not None:
            parts.append(f"remaining={data['remaining']}")
        raise SdkError(", ".join(parts))

    message = data.get("message")
    error = data.get("error")
    details = data.get("details")

    if error and details:
        raise SdkError(f"{error}: {details}")
    if error:
        raise SdkError(error)
    if message:
        raise SdkError(message)
    if details:
        raise SdkError(details)

    raise SdkError(f"Unexpected status: {status}")
