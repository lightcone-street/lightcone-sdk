import { PublicKey, Transaction, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { requireConnection } from "../../context";
import { RetryPolicy } from "../../http";
import {
  buildRedeemWinningsIx,
  buildWithdrawFromPositionIx,
  buildInitPositionTokensIx,
  buildExtendPositionTokensIx,
  buildDepositToGlobalIx,
  buildGlobalToMarketDepositIx,
  buildWithdrawFromGlobalIx,
} from "../../program/instructions";
import { getPositionPda } from "../../program/pda";
import { deserializePosition as deserializeProgramPosition } from "../../program/accounts";
import type {
  Position as ProgramPosition,
  RedeemWinningsParams,
  WithdrawFromPositionParams,
  InitPositionTokensParams,
  ExtendPositionTokensParams,
  DepositToGlobalParams,
  GlobalToMarketDepositParams,
  WithdrawFromGlobalParams,
} from "../../program/types";
import type { PubkeyStr } from "../../shared";
import type { DepositTokenBalance } from "./index";
import type { MarketPositionsResponse, PositionsResponse } from "./wire";
import {
  DepositBuilder,
  MergeBuilder,
  WithdrawBuilder,
  RedeemWinningsBuilder,
  WithdrawFromPositionBuilder,
  InitPositionTokensBuilder,
  ExtendPositionTokensBuilder,
  DepositToGlobalBuilder,
  WithdrawFromGlobalBuilder,
  GlobalToMarketDepositBuilder,
} from "./builders";

export class Positions {
  constructor(private readonly client: ClientContext) {}

  // ── PDA helpers ──────────────────────────────────────────────────────

  pda(owner: PublicKey, market: PublicKey): PublicKey {
    return getPositionPda(owner, market, this.client.programId)[0];
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  async get(userPubkey: string): Promise<PositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/${encodeURIComponent(userPubkey)}/positions`;
    return this.client.http.get<PositionsResponse>(url, RetryPolicy.Idempotent);
  }

  async getForMarket(userPubkey: string, marketPubkey: string): Promise<MarketPositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/${encodeURIComponent(userPubkey)}/markets/${encodeURIComponent(marketPubkey)}/positions`;
    return this.client.http.get<MarketPositionsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Get all conditional-token positions for the authenticated user across
   * every market. The wallet is resolved server-side from the `auth_token`
   * cookie, so no parameter is required. Same response shape as `get()`.
   *
   * `GET /api/users/positions`
   */
  async positions(): Promise<PositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/positions`;
    return this.client.http.get<PositionsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Get SPL deposit-token balances for the authenticated user.
   *
   * The wallet is resolved server-side from the `auth_token` cookie, so no
   * parameter is required. Returns balances keyed by mint pubkey for every
   * deposit token registered in the backend's `deposit_token_metadata`.
   * An empty object means the user has none of the tracked balances — this
   * is not an error.
   */
  async depositTokenBalances(): Promise<Record<PubkeyStr, DepositTokenBalance>> {
    const url = `${this.client.http.baseUrl()}/api/users/deposit-token-balances`;
    return this.client.http.get<Record<PubkeyStr, DepositTokenBalance>>(
      url,
      RetryPolicy.Idempotent,
    );
  }

  // ── On-chain transaction builders ────────────────────────────────────

  redeemWinningsIx(
    params: RedeemWinningsParams,
    winningOutcome: number
  ): TransactionInstruction {
    return buildRedeemWinningsIx(params, winningOutcome, this.client.programId);
  }

  withdrawFromPositionIx(
    params: WithdrawFromPositionParams,
    isToken2022: boolean
  ): TransactionInstruction {
    return buildWithdrawFromPositionIx(params, isToken2022, this.client.programId);
  }

  initPositionTokensIx(
    params: InitPositionTokensParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildInitPositionTokensIx(params, numOutcomes, this.client.programId);
  }

  extendPositionTokensIx(
    params: ExtendPositionTokensParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildExtendPositionTokensIx(params, numOutcomes, this.client.programId);
  }

  depositToGlobalIx(params: DepositToGlobalParams): TransactionInstruction {
    return buildDepositToGlobalIx(params, this.client.programId);
  }

  globalToMarketDepositIx(
    params: GlobalToMarketDepositParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildGlobalToMarketDepositIx(params, numOutcomes, this.client.programId);
  }

  withdrawFromGlobalIx(params: WithdrawFromGlobalParams): TransactionInstruction {
    return buildWithdrawFromGlobalIx(params, this.client.programId);
  }

  // ── Transaction builders (_tx convenience wrappers) ─────────────────

  redeemWinningsTx(
    params: RedeemWinningsParams,
    winningOutcome: number
  ): Transaction {
    const ix = this.redeemWinningsIx(params, winningOutcome);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawFromPositionTx(
    params: WithdrawFromPositionParams,
    isToken2022: boolean
  ): Transaction {
    const ix = this.withdrawFromPositionIx(params, isToken2022);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  initPositionTokensTx(
    params: InitPositionTokensParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.initPositionTokensIx(params, numOutcomes);
    return new Transaction({ feePayer: params.payer }).add(ix);
  }

  extendPositionTokensTx(
    params: ExtendPositionTokensParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.extendPositionTokensIx(params, numOutcomes);
    return new Transaction({ feePayer: params.payer }).add(ix);
  }

  depositToGlobalTx(params: DepositToGlobalParams): Transaction {
    const ix = this.depositToGlobalIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  globalToMarketDepositTx(
    params: GlobalToMarketDepositParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.globalToMarketDepositIx(params, numOutcomes);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawFromGlobalTx(params: WithdrawFromGlobalParams): Transaction {
    const ix = this.withdrawFromGlobalIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  // ── Builder factories ──────────────────────────────────────────────

  deposit(): DepositBuilder {
    return new DepositBuilder(this.client, this.client.depositSource);
  }

  merge(): MergeBuilder {
    return new MergeBuilder(this.client);
  }

  withdraw(): WithdrawBuilder {
    return new WithdrawBuilder(this.client, this.client.depositSource);
  }

  redeemWinnings(): RedeemWinningsBuilder {
    return new RedeemWinningsBuilder(this.client);
  }

  withdrawFromPosition(): WithdrawFromPositionBuilder {
    return new WithdrawFromPositionBuilder(this.client);
  }

  initPositionTokens(): InitPositionTokensBuilder {
    return new InitPositionTokensBuilder(this.client);
  }

  extendPositionTokens(): ExtendPositionTokensBuilder {
    return new ExtendPositionTokensBuilder(this.client);
  }

  depositToGlobal(): DepositToGlobalBuilder {
    return new DepositToGlobalBuilder(this.client);
  }

  withdrawFromGlobal(): WithdrawFromGlobalBuilder {
    return new WithdrawFromGlobalBuilder(this.client);
  }

  globalToMarketDeposit(): GlobalToMarketDepositBuilder {
    return new GlobalToMarketDepositBuilder(this.client);
  }

  // ── On-chain account fetchers (require Connection) ──────────────────

  async getOnchain(owner: PublicKey, market: PublicKey): Promise<ProgramPosition | null> {
    const connection = requireConnection(this.client);
    const positionPda = this.pda(owner, market);
    const accountInfo = await connection.getAccountInfo(positionPda);
    if (!accountInfo) {
      return null;
    }
    return deserializeProgramPosition(accountInfo.data as Buffer);
  }
}
