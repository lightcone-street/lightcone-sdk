"""Markets sub-client — fetch, search, PDA helpers, and on-chain market operations."""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING
from urllib.parse import quote as url_quote

from solders.pubkey import Pubkey

from . import (
    GlobalDepositAsset,
    GlobalDepositAssetsResult,
    Market,
    MarketsResult,
    Status,
)
from .wire import (
    GlobalDepositAssetsListWire,
    MarketResponse,
    MarketSearchResult,
    MarketWire,
)
from .convert import (
    global_deposit_asset_from_wire,
    market_from_wire,
    validation_errors_from_wire,
)
from ...error import SdkError
from ...program.accounts import deserialize_market
from ...program.errors import AccountNotFoundError
from ...program.pda import (
    get_all_conditional_mint_pdas,
    get_market_pda,
)
from ...program.types import Market as OnchainMarket
from ...program.utils import derive_condition_id
from ...rpc import require_connection

if TYPE_CHECKING:
    from ...client import LightconeClient


class Markets:
    """Markets query operations."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    # ── PDA helpers ──────────────────────────────────────────────────────

    def pda(self, market_id: int) -> Pubkey:
        """Get the Market PDA for a given market ID."""
        addr, _ = get_market_pda(market_id, self._client.program_id)
        return addr

    # ── Market helpers ───────────────────────────────────────────────────

    def derive_condition_id(
        self, oracle: Pubkey, question_id: bytes, num_outcomes: int
    ) -> bytes:
        """Derive the condition ID for a market."""
        return derive_condition_id(oracle, question_id, num_outcomes)

    def get_conditional_mints(
        self, market: Pubkey, deposit_mint: Pubkey, num_outcomes: int
    ) -> list[Pubkey]:
        """Get all conditional mint addresses for a market."""
        return [
            addr for addr, _ in get_all_conditional_mint_pdas(
                market, deposit_mint, num_outcomes, self._client.program_id
            )
        ]

    # ── HTTP methods ─────────────────────────────────────────────────────

    async def get(
        self,
        cursor: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> MarketsResult:
        """Get markets with Rust-aligned filtering and validation reporting."""
        url = "/api/markets"
        query_parts: list[str] = []
        if cursor is not None:
            query_parts.append(f"cursor={cursor}")
        if limit is not None:
            query_parts.append(f"limit={limit}")
        if query_parts:
            url += "?" + "&".join(query_parts)

        data = await self._client._http.get(url)
        resp = MarketResponse.from_dict(data)
        markets: list[Market] = []
        validation_errors: list[str] = []

        for wire_market in resp.markets:
            errors = validation_errors_from_wire(wire_market)
            validation_errors.extend(errors)
            if errors:
                continue

            market = market_from_wire(wire_market)
            if market.status in {Status.ACTIVE, Status.RESOLVED}:
                markets.append(market)

        return MarketsResult(
            markets=markets,
            validation_errors=validation_errors,
        )

    async def get_by_slug(self, slug: str) -> Market:
        """Get a market by its URL slug."""
        data = await self._client._http.get(f"/api/markets/by-slug/{url_quote(slug, safe='')}")
        wire = MarketWire.from_dict(data.get("market", data))
        errors = validation_errors_from_wire(wire)
        if errors:
            raise SdkError("; ".join(errors))
        return market_from_wire(wire)

    async def get_by_pubkey(self, pubkey: str) -> Market:
        """Get a market by its pubkey."""
        data = await self._client._http.get(f"/api/markets/{url_quote(pubkey, safe='')}")
        wire = MarketWire.from_dict(data.get("market", data))
        errors = validation_errors_from_wire(wire)
        if errors:
            raise SdkError("; ".join(errors))
        return market_from_wire(wire)

    async def search(self, query: str, limit: Optional[int] = None) -> list[MarketSearchResult]:
        """Search markets by query string."""
        encoded = url_quote(query, safe='')
        url = f"/api/markets/search/by-query/{encoded}"
        if limit is not None:
            url += f"?limit={limit}"
        data = await self._client._http.get(url)
        markets_data = data if isinstance(data, list) else data.get("markets", [])
        return [MarketSearchResult.from_dict(m) for m in markets_data]

    async def featured(self) -> list[MarketSearchResult]:
        """Get featured markets."""
        data = await self._client._http.get("/api/markets/search/featured")
        markets_data = data if isinstance(data, list) else data.get("markets", [])
        results = [MarketSearchResult.from_dict(m) for m in markets_data]
        return [
            result for result in results
            if result.market_status in {"Active", "Resolved"}
        ]

    async def global_deposit_assets(self) -> GlobalDepositAssetsResult:
        """Fetch the active global deposit asset whitelist.

        Platform-scoped (not market-bound). Assets that fail validation are
        skipped and their errors are returned in
        ``GlobalDepositAssetsResult.validation_errors``.
        """
        data = await self._client._http.get("/api/global-deposit-assets")
        response = GlobalDepositAssetsListWire.from_dict(data)

        assets: list[GlobalDepositAsset] = []
        validation_errors: list[str] = []
        for wire_asset in response.assets:
            try:
                assets.append(global_deposit_asset_from_wire(wire_asset))
            except SdkError as error:
                validation_errors.append(str(error))

        return GlobalDepositAssetsResult(
            assets=assets,
            validation_errors=validation_errors,
        )

    # ── On-chain account fetchers (require connection) ───────────────────

    async def get_onchain(self, market_address: Pubkey) -> OnchainMarket:
        """Fetch a Market account by on-chain pubkey."""
        conn = require_connection(self._client)
        response = await conn.get_account_info(market_address)
        if response.value is None:
            raise AccountNotFoundError(str(market_address))
        return deserialize_market(response.value.data)

    async def get_by_id_onchain(self, market_id: int) -> OnchainMarket:
        """Fetch a Market account by ID."""
        addr = self.pda(market_id)
        return await self.get_onchain(addr)

    async def next_id(self) -> int:
        """Get the next available market ID."""
        exchange = await self._client.rpc().get_exchange()
        return exchange.market_count
