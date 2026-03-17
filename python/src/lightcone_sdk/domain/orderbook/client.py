"""Orderbooks sub-client — depth, decimals, PDA helpers, and on-chain orderbook operations."""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING

from solders.pubkey import Pubkey

from .wire import OrderbookDepthResponse, DecimalsResponse
from ...program.accounts import deserialize_orderbook
from ...program.errors import AccountNotFoundError
from ...program.pda import get_orderbook_pda
from ...program.types import Orderbook as OnchainOrderbook
from ...rpc import require_connection

if TYPE_CHECKING:
    from ...client import LightconeClient


class Orderbooks:
    """Orderbook operations sub-client."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    # ── PDA helpers ──────────────────────────────────────────────────────

    def pda(self, mint_a: Pubkey, mint_b: Pubkey) -> Pubkey:
        """Get the Orderbook PDA."""
        addr, _ = get_orderbook_pda(mint_a, mint_b, self._client.program_id)
        return addr

    # ── HTTP methods ─────────────────────────────────────────────────────

    async def get(self, orderbook_id: str, depth: Optional[int] = None) -> OrderbookDepthResponse:
        """Get orderbook depth."""
        url = f"/api/orderbook/{orderbook_id}"
        if depth is not None:
            url += f"?depth={depth}"
        data = await self._client._http.get(url)
        return OrderbookDepthResponse.from_dict(data)

    async def decimals(self, orderbook_id: str) -> DecimalsResponse:
        """Get decimal configuration (cached)."""
        if orderbook_id in self._client._decimals_cache:
            return self._client._decimals_cache[orderbook_id]

        data = await self._client._http.get(f"/api/orderbooks/{orderbook_id}/decimals")
        result = DecimalsResponse.from_dict(data)
        self._client._decimals_cache[orderbook_id] = result
        return result

    def clear_cache(self) -> None:
        """Clear the decimals cache."""
        self._client._decimals_cache.clear()

    # ── On-chain account fetchers (require connection) ───────────────────

    async def get_onchain(self, mint_a: Pubkey, mint_b: Pubkey) -> OnchainOrderbook:
        """Fetch an Orderbook account by mint pair."""
        conn = require_connection(self._client)
        addr = self.pda(mint_a, mint_b)
        response = await conn.get_account_info(addr)
        if response.value is None:
            raise AccountNotFoundError(str(addr))
        return deserialize_orderbook(response.value.data)
