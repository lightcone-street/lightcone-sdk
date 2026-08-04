"""Orderbooks sub-client — depth and on-chain orderbook operations."""

from __future__ import annotations

import asyncio
from typing import Optional, TYPE_CHECKING

from solders.instruction import Instruction
from solders.pubkey import Pubkey
from solders.transaction import Transaction

from .aggregation import BookAggregation
from .wire import DecimalsResponse, OrderbookDepthResponse
from ...shared.scaling import OrderbookRules
from ...program.accounts import deserialize_orderbook
from ...program.errors import AccountNotFoundError
from ...program.instructions import (
    build_close_orderbook_alt_instruction,
    build_close_orderbook_instruction,
)
from ...program.pda import get_orderbook_pda
from ...program.types import (
    CloseOrderbookAltParams,
    CloseOrderbookParams,
    Orderbook as OnchainOrderbook,
)
from ...rpc import require_connection

if TYPE_CHECKING:
    from ...client import LightconeClient


class Orderbooks:
    """Orderbook operations sub-client."""

    def __init__(self, client: "LightconeClient"):
        self._client = client
        self._rules_cache: dict[str, OrderbookRules] = {}
        self._rules_in_flight: dict[str, asyncio.Task[OrderbookRules]] = {}

    # ── PDA helpers ──────────────────────────────────────────────────────

    def pda(self, mint_a: Pubkey, mint_b: Pubkey) -> Pubkey:
        """Get the Orderbook PDA."""
        addr, _ = get_orderbook_pda(mint_a, mint_b, self._client.program_id)
        return addr

    # ── On-chain instruction builders ────────────────────────────────────

    def close_orderbook_alt_ix(
        self,
        params: CloseOrderbookAltParams,
    ) -> Instruction:
        """Build CloseOrderbookAlt instruction."""
        return build_close_orderbook_alt_instruction(params, self._client.program_id)

    def close_orderbook_ix(self, params: CloseOrderbookParams) -> Instruction:
        """Build CloseOrderbook instruction."""
        return build_close_orderbook_instruction(params, self._client.program_id)

    # ── On-chain transaction builders ────────────────────────────────────

    def close_orderbook_alt_tx(
        self,
        params: CloseOrderbookAltParams,
    ) -> Transaction:
        """Build CloseOrderbookAlt transaction."""
        ix = self.close_orderbook_alt_ix(params)
        return Transaction.new_with_payer([ix], params.operator)

    def close_orderbook_tx(self, params: CloseOrderbookParams) -> Transaction:
        """Build CloseOrderbook transaction."""
        ix = self.close_orderbook_ix(params)
        return Transaction.new_with_payer([ix], params.operator)

    # ── HTTP methods ─────────────────────────────────────────────────────

    async def get(
        self,
        orderbook_id: str,
        depth: Optional[int] = None,
        *,
        n_sig_figs: Optional[int] = None,
        mantissa: Optional[int] = None,
    ) -> OrderbookDepthResponse:
        """Get live orderbook depth, optionally aggregated (Hyperliquid-style).

        ``depth`` is capped server-side at 20 levels per side (omitted, 0, or
        >20 all serve 20). Invalid aggregation combinations raise
        ``ValueError`` client-side before any request is made (the server
        would 400 with ``INVALID_ORDERBOOK_QUERY``), and unknown query params
        are rejected server-side — only ``depth``, ``nSigFigs``, and
        ``mantissa`` are ever sent.
        """
        aggregation = BookAggregation.validate(n_sig_figs, mantissa)
        params: dict[str, str] = {}
        if depth is not None:
            params["depth"] = str(depth)
        if aggregation.n_sig_figs is not None:
            params["nSigFigs"] = str(aggregation.n_sig_figs)
        if aggregation.mantissa is not None:
            params["mantissa"] = str(aggregation.mantissa)
        data = await self._client._http.get(
            f"/api/orderbook/{orderbook_id}", params=params or None
        )
        return OrderbookDepthResponse.from_dict(data)

    async def decimals(self, orderbook_id: str) -> OrderbookRules:
        """Fetch and cache immutable exact admission rules for an active book."""
        cached = self._rules_cache.get(orderbook_id)
        if cached is not None:
            return cached
        task = self._rules_in_flight.get(orderbook_id)
        if task is None:
            async def _fetch() -> OrderbookRules:
                data = await self._client._http.get(
                    f"/api/orderbooks/{orderbook_id}/decimals"
                )
                rules = DecimalsResponse.from_dict(data).to_rules()
                rules.validate_for_orderbook(orderbook_id)
                return rules

            task = asyncio.create_task(_fetch())
            self._rules_in_flight[orderbook_id] = task
        try:
            rules = await task
            self._rules_cache[orderbook_id] = rules
            return rules
        finally:
            if self._rules_in_flight.get(orderbook_id) is task:
                self._rules_in_flight.pop(orderbook_id, None)

    def invalidate_decimals(self, orderbook_id: str) -> None:
        self._rules_cache.pop(orderbook_id, None)

    def clear_decimals_cache(self) -> None:
        self._rules_cache.clear()

    # ── On-chain account fetchers (require connection) ───────────────────

    async def get_onchain(self, mint_a: Pubkey, mint_b: Pubkey) -> OnchainOrderbook:
        """Fetch an Orderbook account by mint pair."""
        conn = require_connection(self._client)
        addr = self.pda(mint_a, mint_b)
        response = await conn.get_account_info(addr)
        if response.value is None:
            raise AccountNotFoundError(str(addr))
        return deserialize_orderbook(response.value.data)
