"""Admin sub-client — metadata, referral management, and on-chain admin operations."""

from __future__ import annotations

from typing import TYPE_CHECKING
from urllib.parse import quote as url_quote
from urllib.parse import urlencode

from solders.instruction import Instruction
from solders.pubkey import Pubkey
from solders.transaction import Transaction

from ...program.instructions import (
    build_activate_market_instruction,
    build_add_deposit_mint_instruction,
    build_create_market_instruction,
    build_create_orderbook_instruction,
    build_deposit_and_swap_instruction,
    build_initialize_instruction,
    build_match_orders_multi_instruction,
    build_set_authority_instruction,
    build_set_manager_instruction,
    build_set_operator_instruction,
    build_set_paused_instruction,
    build_settle_market_instruction,
    build_whitelist_deposit_token_instruction,
)
from ...program.types import (
    ActivateMarketParams,
    AddDepositMintParams,
    CreateOrderbookParams,
    DepositAndSwapParams,
    MatchOrdersMultiParams,
    SetAuthorityParams,
    SetManagerParams,
    SettleMarketParams,
    WhitelistDepositTokenParams,
)
from . import (
    AdminLogEvent,
    AdminLogEventsQuery,
    AdminLogEventsResponse,
    AdminLoginRequest,
    AdminLoginResponse,
    AdminLogMetricHistoryQuery,
    AdminLogMetricHistoryResponse,
    AdminLogMetricsQuery,
    AdminLogMetricsResponse,
    AdminNonceResponse,
    AllocateCodesRequest,
    AllocateCodesResponse,
    CreateNotificationRequest,
    CreateNotificationResponse,
    DismissNotificationRequest,
    DismissNotificationResponse,
    ListCodesRequest,
    ListCodesResponse,
    ReferralConfig,
    RevokeRequest,
    RevokeResponse,
    UnifiedMetadataRequest,
    UnifiedMetadataResponse,
    UnrevokeRequest,
    UnrevokeResponse,
    UpdateCodeRequest,
    UpdateCodeResponse,
    UpdateConfigRequest,
    UploadMarketDeploymentAssetsRequest,
    UploadMarketDeploymentAssetsResponse,
    WhitelistRequest,
    WhitelistResponse,
)

if TYPE_CHECKING:
    from ...client import LightconeClient


class Admin:
    """Admin operations sub-client."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    # ── Admin auth ───────────────────────────────────────────────────────

    async def get_admin_nonce(self) -> AdminNonceResponse:
        """Fetch admin login nonce and message to sign."""
        data = await self._client._http.get("/api/admin/nonce")
        return AdminNonceResponse.from_dict(data)

    async def admin_login(
        self,
        message: str,
        signature_bs58: str,
        pubkey_bytes: list[int],
    ) -> AdminLoginResponse:
        """Admin login -- verifies signature and stores admin cookie for subsequent admin requests."""
        request = AdminLoginRequest(
            message=message,
            signature_bs58=signature_bs58,
            pubkey_bytes=pubkey_bytes,
        )
        data = await self._client._http.post("/api/admin/login", request.to_dict())
        return AdminLoginResponse.from_dict(data)

    async def admin_logout(self) -> None:
        """Logout admin -- best-effort server logout, always clears local admin token."""
        try:
            await self._client._http.admin_post("/api/admin/logout", {})
        except Exception:
            pass
        self._client._http.clear_admin_token()

    # ── Admin API methods ────────────────────────────────────────────────

    async def upsert_metadata(
        self, request: UnifiedMetadataRequest
    ) -> UnifiedMetadataResponse:
        """Upsert market/token metadata. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/metadata", request.to_dict()
        )
        return UnifiedMetadataResponse.from_dict(data)

    async def upload_market_deployment_assets(
        self, request: UploadMarketDeploymentAssetsRequest
    ) -> UploadMarketDeploymentAssetsResponse:
        """Upload banner/icon/outcome/token images and metadata for a newly created market.

        Returns the uploaded URLs. Requires prior admin_login().
        """
        data = await self._client._http.admin_post(
            "/api/admin/metadata/upload-market-deployment-assets", request.to_dict()
        )
        return UploadMarketDeploymentAssetsResponse.from_dict(data)

    async def allocate_codes(
        self, request: AllocateCodesRequest
    ) -> AllocateCodesResponse:
        """Allocate referral codes. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/referral/allocate", request.to_dict()
        )
        return AllocateCodesResponse.from_dict(data)

    async def whitelist(self, request: WhitelistRequest) -> WhitelistResponse:
        """Whitelist wallet addresses. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/referral/whitelist", request.to_dict()
        )
        return WhitelistResponse.from_dict(data)

    async def revoke(self, request: RevokeRequest) -> RevokeResponse:
        """Revoke access. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/referral/revoke", request.to_dict()
        )
        return RevokeResponse.from_dict(data)

    async def unrevoke(self, request: UnrevokeRequest) -> UnrevokeResponse:
        """Unrevoke access. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/referral/unrevoke", request.to_dict()
        )
        return UnrevokeResponse.from_dict(data)

    async def create_notification(
        self, request: CreateNotificationRequest
    ) -> CreateNotificationResponse:
        """Create a notification. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/notifications", request.to_dict()
        )
        return CreateNotificationResponse.from_dict(data)

    async def dismiss_notification(
        self, request: DismissNotificationRequest
    ) -> DismissNotificationResponse:
        """Dismiss a notification. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/notifications/dismiss", request.to_dict()
        )
        return DismissNotificationResponse.from_dict(data)

    # ── Referral config / codes ──────────────────────────────────────────

    async def get_referral_config(self) -> ReferralConfig:
        """Fetch the platform-wide referral configuration. Requires prior admin_login()."""
        data = await self._client._http.admin_post("/api/admin/referral/config/get", {})
        return ReferralConfig.from_dict(data)

    async def update_referral_config(
        self, request: UpdateConfigRequest
    ) -> ReferralConfig:
        """Update the platform-wide referral configuration. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/referral/config/update", request.to_dict()
        )
        return ReferralConfig.from_dict(data)

    async def list_referral_codes(self, request: ListCodesRequest) -> ListCodesResponse:
        """List referral codes with optional owner/batch/code filters. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/referral/codes", request.to_dict()
        )
        return ListCodesResponse.from_dict(data)

    async def update_referral_code(
        self, request: UpdateCodeRequest
    ) -> UpdateCodeResponse:
        """Update the max_uses for a referral code. Requires prior admin_login()."""
        data = await self._client._http.admin_post(
            "/api/admin/referral/codes/update", request.to_dict()
        )
        return UpdateCodeResponse.from_dict(data)

    # ── Admin logs ───────────────────────────────────────────────────────

    async def list_log_events(
        self, query: AdminLogEventsQuery
    ) -> AdminLogEventsResponse:
        """List structured log events with cursor-based pagination. Requires prior admin_login()."""
        url = "/api/admin/logs/events"
        params = query.to_query()
        if params:
            url += "?" + urlencode(params)
        data = await self._client._http.admin_get(url)
        return AdminLogEventsResponse.from_dict(data)

    async def get_log_event(self, public_id: str) -> AdminLogEvent:
        """Fetch a single log event by its public_id. Requires prior admin_login()."""
        data = await self._client._http.admin_get(
            f"/api/admin/logs/events/{url_quote(public_id, safe='')}"
        )
        return AdminLogEvent.from_dict(data)

    async def log_metrics(self, query: AdminLogMetricsQuery) -> AdminLogMetricsResponse:
        """Fetch rolled-up log metrics broken down by window and scope. Requires prior admin_login()."""
        url = "/api/admin/logs/metrics"
        params = query.to_query()
        if params:
            url += "?" + urlencode(params)
        data = await self._client._http.admin_get(url)
        return AdminLogMetricsResponse.from_dict(data)

    async def log_metric_history(
        self, query: AdminLogMetricHistoryQuery
    ) -> AdminLogMetricHistoryResponse:
        """Fetch the history of log metric buckets for a given scope. Requires prior admin_login()."""
        url = "/api/admin/logs/metrics/history?" + urlencode(query.to_query())
        data = await self._client._http.admin_get(url)
        return AdminLogMetricHistoryResponse.from_dict(data)

    # ── On-chain instruction builders ────────────────────────────────────

    def initialize_ix(self, authority: Pubkey) -> Instruction:
        """Build Initialize instruction."""
        return build_initialize_instruction(authority, self._client.program_id)

    async def create_market_ix(
        self,
        manager: Pubkey,
        num_outcomes: int,
        oracle: Pubkey,
        question_id: bytes,
    ) -> Instruction:
        """Build CreateMarket instruction.

        Async because it fetches the next market ID from on-chain state.
        """
        market_id = await self._client.markets().next_id()
        return build_create_market_instruction(
            manager=manager,
            market_id=market_id,
            num_outcomes=num_outcomes,
            oracle=oracle,
            question_id=question_id,
            program_id=self._client.program_id,
        )

    def add_deposit_mint_ix(
        self,
        params: AddDepositMintParams,
        market: Pubkey,
        num_outcomes: int,
    ) -> Instruction:
        """Build AddDepositMint instruction."""
        return build_add_deposit_mint_instruction(
            manager=params.manager,
            market=market,
            deposit_mint=params.deposit_mint,
            outcome_metadata=params.outcome_metadata,
            num_outcomes=num_outcomes,
            program_id=self._client.program_id,
        )

    def activate_market_ix(self, params: ActivateMarketParams) -> Instruction:
        """Build ActivateMarket instruction."""
        return build_activate_market_instruction(
            manager=params.manager,
            market_id=params.market_id,
            program_id=self._client.program_id,
        )

    def settle_market_ix(self, params: SettleMarketParams) -> Instruction:
        """Build SettleMarket instruction."""
        return build_settle_market_instruction(
            oracle=params.oracle,
            market_id=params.market_id,
            payout_numerators=params.payout_numerators,
            program_id=self._client.program_id,
        )

    def set_paused_ix(self, authority: Pubkey, paused: bool) -> Instruction:
        """Build SetPaused instruction."""
        return build_set_paused_instruction(authority, paused, self._client.program_id)

    def set_operator_ix(self, authority: Pubkey, new_operator: Pubkey) -> Instruction:
        """Build SetOperator instruction."""
        return build_set_operator_instruction(
            authority, new_operator, self._client.program_id
        )

    def set_authority_ix(self, params: SetAuthorityParams) -> Instruction:
        """Build SetAuthority instruction."""
        return build_set_authority_instruction(
            current_authority=params.current_authority,
            new_authority=params.new_authority,
            program_id=self._client.program_id,
        )

    def set_manager_ix(self, params: SetManagerParams) -> Instruction:
        """Build SetManager instruction."""
        return build_set_manager_instruction(
            authority=params.authority,
            new_manager=params.new_manager,
            program_id=self._client.program_id,
        )

    def whitelist_deposit_token_ix(
        self, params: WhitelistDepositTokenParams
    ) -> Instruction:
        """Build WhitelistDepositToken instruction."""
        return build_whitelist_deposit_token_instruction(
            authority=params.authority,
            mint=params.mint,
            program_id=self._client.program_id,
        )

    def create_orderbook_ix(self, params: CreateOrderbookParams) -> Instruction:
        """Build CreateOrderbook instruction."""
        return build_create_orderbook_instruction(
            manager=params.manager,
            market=params.market,
            mint_a=params.mint_a,
            mint_b=params.mint_b,
            mint_a_deposit_mint=params.mint_a_deposit_mint,
            mint_b_deposit_mint=params.mint_b_deposit_mint,
            recent_slot=params.recent_slot,
            base_index=params.base_index,
            mint_a_outcome_index=params.mint_a_outcome_index,
            mint_b_outcome_index=params.mint_b_outcome_index,
            program_id=self._client.program_id,
        )

    def match_orders_multi_ix(self, params: MatchOrdersMultiParams) -> Instruction:
        """Build MatchOrdersMulti instruction."""
        return build_match_orders_multi_instruction(
            operator=params.operator,
            market=params.market,
            base_mint=params.base_mint,
            quote_mint=params.quote_mint,
            taker_order=params.taker_order,
            maker_orders=params.maker_orders,
            maker_fill_amounts=params.maker_fill_amounts,
            taker_fill_amounts=params.taker_fill_amounts,
            full_fill_bitmask=params.full_fill_bitmask,
            program_id=self._client.program_id,
        )

    def deposit_and_swap_ix(self, params: DepositAndSwapParams) -> Instruction:
        """Build DepositAndSwap instruction."""
        return build_deposit_and_swap_instruction(
            operator=params.operator,
            market=params.market,
            base_mint=params.base_mint,
            quote_mint=params.quote_mint,
            taker_order=params.taker_order,
            taker_is_full_fill=params.taker_is_full_fill,
            taker_is_deposit=params.taker_is_deposit,
            taker_deposit_mint=params.taker_deposit_mint,
            num_outcomes=params.num_outcomes,
            makers=params.makers,
            program_id=self._client.program_id,
        )

    # ── On-chain transaction builders ────────────────────────────────────

    def initialize_tx(self, authority: Pubkey) -> Transaction:
        """Build Initialize transaction."""
        ix = self.initialize_ix(authority)
        return Transaction.new_with_payer([ix], authority)

    async def create_market_tx(
        self,
        manager: Pubkey,
        num_outcomes: int,
        oracle: Pubkey,
        question_id: bytes,
    ) -> Transaction:
        """Build CreateMarket transaction.

        Async because it fetches the next market ID from on-chain state.
        """
        ix = await self.create_market_ix(manager, num_outcomes, oracle, question_id)
        return Transaction.new_with_payer([ix], manager)

    def add_deposit_mint_tx(
        self,
        params: AddDepositMintParams,
        market: Pubkey,
        num_outcomes: int,
    ) -> Transaction:
        """Build AddDepositMint transaction."""
        ix = self.add_deposit_mint_ix(params, market, num_outcomes)
        return Transaction.new_with_payer([ix], params.manager)

    def activate_market_tx(self, params: ActivateMarketParams) -> Transaction:
        """Build ActivateMarket transaction."""
        ix = self.activate_market_ix(params)
        return Transaction.new_with_payer([ix], params.manager)

    def settle_market_tx(self, params: SettleMarketParams) -> Transaction:
        """Build SettleMarket transaction."""
        ix = self.settle_market_ix(params)
        return Transaction.new_with_payer([ix], params.oracle)

    def set_paused_tx(self, authority: Pubkey, paused: bool) -> Transaction:
        """Build SetPaused transaction."""
        ix = self.set_paused_ix(authority, paused)
        return Transaction.new_with_payer([ix], authority)

    def set_operator_tx(self, authority: Pubkey, new_operator: Pubkey) -> Transaction:
        """Build SetOperator transaction."""
        ix = self.set_operator_ix(authority, new_operator)
        return Transaction.new_with_payer([ix], authority)

    def set_authority_tx(self, params: SetAuthorityParams) -> Transaction:
        """Build SetAuthority transaction."""
        ix = self.set_authority_ix(params)
        return Transaction.new_with_payer([ix], params.current_authority)

    def set_manager_tx(self, params: SetManagerParams) -> Transaction:
        """Build SetManager transaction."""
        ix = self.set_manager_ix(params)
        return Transaction.new_with_payer([ix], params.authority)

    def whitelist_deposit_token_tx(
        self, params: WhitelistDepositTokenParams
    ) -> Transaction:
        """Build WhitelistDepositToken transaction."""
        ix = self.whitelist_deposit_token_ix(params)
        return Transaction.new_with_payer([ix], params.authority)

    def create_orderbook_tx(self, params: CreateOrderbookParams) -> Transaction:
        """Build CreateOrderbook transaction."""
        ix = self.create_orderbook_ix(params)
        return Transaction.new_with_payer([ix], params.manager)

    def match_orders_multi_tx(self, params: MatchOrdersMultiParams) -> Transaction:
        """Build MatchOrdersMulti transaction."""
        ix = self.match_orders_multi_ix(params)
        return Transaction.new_with_payer([ix], params.operator)

    def deposit_and_swap_tx(self, params: DepositAndSwapParams) -> Transaction:
        """Build DepositAndSwap transaction."""
        ix = self.deposit_and_swap_ix(params)
        return Transaction.new_with_payer([ix], params.operator)
