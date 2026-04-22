import { Transaction, type PublicKey, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { RetryPolicy } from "../../http";
import {
  buildInitializeIx,
  buildCreateMarketIx,
  buildAddDepositMintIx,
  buildActivateMarketIx,
  buildSettleMarketIx,
  buildSetPausedIx,
  buildSetOperatorIx,
  buildSetAuthorityIx,
  buildWhitelistDepositTokenIx,
  buildCreateOrderbookIx,
  buildMatchOrdersMultiIx,
  buildDepositAndSwapIx,
} from "../../program/instructions";
import type {
  CreateMarketParams,
  AddDepositMintParams,
  ActivateMarketParams,
  SettleMarketParams,
  SetAuthorityParams,
  WhitelistDepositTokenParams,
  CreateOrderbookParams,
  MatchOrdersMultiParams,
  DepositAndSwapParams,
} from "../../program/types";
import type {
  AdminLoginRequest,
  AdminLoginResponse,
  AdminNonceResponse,
} from "./index";
import type {
  AdminLogEvent,
  AdminLogEventsQuery,
  AdminLogEventsResponse,
  AdminLogMetricHistoryQuery,
  AdminLogMetricHistoryResponse,
  AdminLogMetricsQuery,
  AdminLogMetricsResponse,
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
  WhitelistRequest,
  WhitelistResponse,
} from "./wire";

export class Admin {
  constructor(private readonly client: ClientContext) {}

  // ── Admin auth ───────────────────────────────────────────────────────

  async getAdminNonce(): Promise<AdminNonceResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/nonce`;
    return this.client.http.get<AdminNonceResponse>(url, RetryPolicy.None);
  }

  async adminLogin(request: AdminLoginRequest): Promise<AdminLoginResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/login`;
    return this.client.http.post<AdminLoginResponse, AdminLoginRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async adminLogout(): Promise<void> {
    const url = `${this.client.http.baseUrl()}/api/admin/logout`;
    try {
      await this.client.http.adminPost(url, {}, RetryPolicy.None);
    } catch {
      // Backend cookie clear can fail in local/dev setups; still clear local state.
    }
    this.client.http.clearAdminToken();
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  async upsertMetadata(
    request: UnifiedMetadataRequest
  ): Promise<UnifiedMetadataResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/metadata`;
    return this.client.http.adminPost<UnifiedMetadataResponse, UnifiedMetadataRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async allocateCodes(
    request: AllocateCodesRequest
  ): Promise<AllocateCodesResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/allocate`;
    return this.client.http.adminPost<AllocateCodesResponse, AllocateCodesRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async whitelist(request: WhitelistRequest): Promise<WhitelistResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/whitelist`;
    return this.client.http.adminPost<WhitelistResponse, WhitelistRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async revoke(request: RevokeRequest): Promise<RevokeResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/revoke`;
    return this.client.http.adminPost<RevokeResponse, RevokeRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async unrevoke(request: UnrevokeRequest): Promise<UnrevokeResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/unrevoke`;
    return this.client.http.adminPost<UnrevokeResponse, UnrevokeRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async createNotification(
    request: CreateNotificationRequest
  ): Promise<CreateNotificationResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/notifications`;
    return this.client.http.adminPost<CreateNotificationResponse, CreateNotificationRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async dismissNotification(
    request: DismissNotificationRequest
  ): Promise<DismissNotificationResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/notifications/dismiss`;
    return this.client.http.adminPost<DismissNotificationResponse, DismissNotificationRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  // ── Referral config / codes ──────────────────────────────────────────

  async getReferralConfig(): Promise<ReferralConfig> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/config/get`;
    return this.client.http.adminPost<ReferralConfig, Record<string, never>>(
      url,
      {},
      RetryPolicy.None
    );
  }

  async updateReferralConfig(
    request: UpdateConfigRequest
  ): Promise<ReferralConfig> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/config/update`;
    return this.client.http.adminPost<ReferralConfig, UpdateConfigRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async listReferralCodes(
    request: ListCodesRequest
  ): Promise<ListCodesResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/codes`;
    return this.client.http.adminPost<ListCodesResponse, ListCodesRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  async updateReferralCode(
    request: UpdateCodeRequest
  ): Promise<UpdateCodeResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/codes/update`;
    return this.client.http.adminPost<UpdateCodeResponse, UpdateCodeRequest>(
      url,
      request,
      RetryPolicy.None
    );
  }

  // ── Admin logs ───────────────────────────────────────────────────────

  async listLogEvents(query: AdminLogEventsQuery): Promise<AdminLogEventsResponse> {
    const qs = buildQueryString(query);
    const url = `${this.client.http.baseUrl()}/api/admin/logs/events${qs}`;
    return this.client.http.adminGet<AdminLogEventsResponse>(url, RetryPolicy.Idempotent);
  }

  async getLogEvent(publicId: string): Promise<AdminLogEvent> {
    const url = `${this.client.http.baseUrl()}/api/admin/logs/events/${encodeURIComponent(publicId)}`;
    return this.client.http.adminGet<AdminLogEvent>(url, RetryPolicy.Idempotent);
  }

  async logMetrics(query: AdminLogMetricsQuery): Promise<AdminLogMetricsResponse> {
    const qs = buildQueryString(query);
    const url = `${this.client.http.baseUrl()}/api/admin/logs/metrics${qs}`;
    return this.client.http.adminGet<AdminLogMetricsResponse>(url, RetryPolicy.Idempotent);
  }

  async logMetricHistory(
    query: AdminLogMetricHistoryQuery
  ): Promise<AdminLogMetricHistoryResponse> {
    const qs = buildQueryString(query);
    const url = `${this.client.http.baseUrl()}/api/admin/logs/metrics/history${qs}`;
    return this.client.http.adminGet<AdminLogMetricHistoryResponse>(url, RetryPolicy.Idempotent);
  }

  // ── On-chain transaction builders ────────────────────────────────────

  initializeIx(authority: PublicKey): TransactionInstruction {
    return buildInitializeIx({ authority }, this.client.programId);
  }

  createMarketIx(params: CreateMarketParams, marketId: bigint): TransactionInstruction {
    return buildCreateMarketIx(params, marketId, this.client.programId);
  }

  addDepositMintIx(
    params: AddDepositMintParams,
    market: PublicKey,
    numOutcomes: number
  ): TransactionInstruction {
    return buildAddDepositMintIx(params, market, numOutcomes, this.client.programId);
  }

  activateMarketIx(params: ActivateMarketParams): TransactionInstruction {
    return buildActivateMarketIx(params, this.client.programId);
  }

  settleMarketIx(params: SettleMarketParams): TransactionInstruction {
    return buildSettleMarketIx(params, this.client.programId);
  }

  setPausedIx(authority: PublicKey, paused: boolean): TransactionInstruction {
    return buildSetPausedIx(authority, paused, this.client.programId);
  }

  setOperatorIx(authority: PublicKey, newOperator: PublicKey): TransactionInstruction {
    return buildSetOperatorIx(authority, newOperator, this.client.programId);
  }

  setAuthorityIx(params: SetAuthorityParams): TransactionInstruction {
    return buildSetAuthorityIx(params, this.client.programId);
  }

  whitelistDepositTokenIx(params: WhitelistDepositTokenParams): TransactionInstruction {
    return buildWhitelistDepositTokenIx(params, this.client.programId);
  }

  createOrderbookIx(params: CreateOrderbookParams): TransactionInstruction {
    return buildCreateOrderbookIx(params, this.client.programId);
  }

  matchOrdersMultiIx(params: MatchOrdersMultiParams): TransactionInstruction {
    return buildMatchOrdersMultiIx(params, this.client.programId);
  }

  depositAndSwapIx(params: DepositAndSwapParams): TransactionInstruction {
    return buildDepositAndSwapIx(params, this.client.programId);
  }

  // ── Transaction builders (_tx convenience wrappers) ─────────────────

  initializeTx(authority: PublicKey): Transaction {
    const ix = this.initializeIx(authority);
    return new Transaction({ feePayer: authority }).add(ix);
  }

  createMarketTx(params: CreateMarketParams, marketId: bigint): Transaction {
    const ix = this.createMarketIx(params, marketId);
    return new Transaction({ feePayer: params.authority }).add(ix);
  }

  addDepositMintTx(
    params: AddDepositMintParams,
    market: PublicKey,
    numOutcomes: number
  ): Transaction {
    const ix = this.addDepositMintIx(params, market, numOutcomes);
    return new Transaction({ feePayer: params.authority }).add(ix);
  }

  activateMarketTx(params: ActivateMarketParams): Transaction {
    const ix = this.activateMarketIx(params);
    return new Transaction({ feePayer: params.authority }).add(ix);
  }

  settleMarketTx(params: SettleMarketParams): Transaction {
    const ix = this.settleMarketIx(params);
    return new Transaction({ feePayer: params.oracle }).add(ix);
  }

  setPausedTx(authority: PublicKey, paused: boolean): Transaction {
    const ix = this.setPausedIx(authority, paused);
    return new Transaction({ feePayer: authority }).add(ix);
  }

  setOperatorTx(authority: PublicKey, newOperator: PublicKey): Transaction {
    const ix = this.setOperatorIx(authority, newOperator);
    return new Transaction({ feePayer: authority }).add(ix);
  }

  setAuthorityTx(params: SetAuthorityParams): Transaction {
    const ix = this.setAuthorityIx(params);
    return new Transaction({ feePayer: params.currentAuthority }).add(ix);
  }

  whitelistDepositTokenTx(params: WhitelistDepositTokenParams): Transaction {
    const ix = this.whitelistDepositTokenIx(params);
    return new Transaction({ feePayer: params.authority }).add(ix);
  }

  createOrderbookTx(params: CreateOrderbookParams): Transaction {
    const ix = this.createOrderbookIx(params);
    return new Transaction({ feePayer: params.authority }).add(ix);
  }

  matchOrdersMultiTx(params: MatchOrdersMultiParams): Transaction {
    const ix = this.matchOrdersMultiIx(params);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }

  depositAndSwapTx(params: DepositAndSwapParams): Transaction {
    const ix = this.depositAndSwapIx(params);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }
}

function buildQueryString(query: object): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null) continue;
    params.append(key, String(value));
  }
  const qs = params.toString();
  return qs ? `?${qs}` : "";
}
