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
import type { AdminEnvelope } from "./index";
import type {
  AllocateCodesRequest,
  AllocateCodesResponse,
  CreateNotificationRequest,
  CreateNotificationResponse,
  DismissNotificationRequest,
  DismissNotificationResponse,
  RevokeRequest,
  RevokeResponse,
  UnifiedMetadataRequest,
  UnifiedMetadataResponse,
  UnrevokeRequest,
  UnrevokeResponse,
  WhitelistRequest,
  WhitelistResponse,
} from "./wire";

export class Admin {
  constructor(private readonly client: ClientContext) {}

  // ── HTTP methods ─────────────────────────────────────────────────────

  async upsertMetadata(
    envelope: AdminEnvelope<UnifiedMetadataRequest>
  ): Promise<UnifiedMetadataResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/metadata`;
    return this.client.http.post<UnifiedMetadataResponse, AdminEnvelope<UnifiedMetadataRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async allocateCodes(
    envelope: AdminEnvelope<AllocateCodesRequest>
  ): Promise<AllocateCodesResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/allocate`;
    return this.client.http.post<AllocateCodesResponse, AdminEnvelope<AllocateCodesRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async whitelist(envelope: AdminEnvelope<WhitelistRequest>): Promise<WhitelistResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/whitelist`;
    return this.client.http.post<WhitelistResponse, AdminEnvelope<WhitelistRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async revoke(envelope: AdminEnvelope<RevokeRequest>): Promise<RevokeResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/revoke`;
    return this.client.http.post<RevokeResponse, AdminEnvelope<RevokeRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async unrevoke(envelope: AdminEnvelope<UnrevokeRequest>): Promise<UnrevokeResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/unrevoke`;
    return this.client.http.post<UnrevokeResponse, AdminEnvelope<UnrevokeRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async createNotification(
    envelope: AdminEnvelope<CreateNotificationRequest>
  ): Promise<CreateNotificationResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/notifications`;
    return this.client.http.post<CreateNotificationResponse, AdminEnvelope<CreateNotificationRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async dismissNotification(
    envelope: AdminEnvelope<DismissNotificationRequest>
  ): Promise<DismissNotificationResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/notifications/dismiss`;
    return this.client.http.post<DismissNotificationResponse, AdminEnvelope<DismissNotificationRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
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
