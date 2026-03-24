import { PublicKey, Transaction, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { requireConnection, resolveDepositSource } from "../../context";
import { SdkError } from "../../error";
import { RetryPolicy } from "../../http";
import {
  buildDepositIx,
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
import { DepositSource } from "../../shared";
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
  type DepositParams,
  type WithdrawParams,
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

  // ── Unified deposit/withdraw (dispatch by deposit source) ───────────

  depositIx(params: DepositParams): TransactionInstruction {
    const source = resolveDepositSource(this.client, params.depositSource);
    switch (source) {
      case DepositSource.Global:
        return this.depositToGlobalIx({
          user: params.user,
          mint: params.mint,
          amount: params.amount,
        });
      case DepositSource.Market: {
        const market = params.market;
        if (!market) {
          throw SdkError.missingMarketContext("market is required for Market deposit");
        }
        const marketPubkey = new PublicKey(market.pubkey);
        const numOutcomes = market.outcomes.length;
        return buildDepositIx(
          {
            user: params.user,
            market: marketPubkey,
            depositMint: params.mint,
            amount: params.amount,
          },
          numOutcomes,
          this.client.programId,
        );
      }
    }
  }

  depositTx(params: DepositParams): Transaction {
    const ix = this.depositIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawIx(params: WithdrawParams): TransactionInstruction {
    const source = resolveDepositSource(this.client, params.depositSource);
    switch (source) {
      case DepositSource.Global:
        return this.withdrawFromGlobalIx({
          user: params.user,
          mint: params.mint,
          amount: params.amount,
        });
      case DepositSource.Market: {
        const ctx = params.marketContext;
        if (!ctx) {
          throw SdkError.missingMarketContext("market_context is required for Market withdrawal");
        }
        const marketPubkey = new PublicKey(ctx.market.pubkey);
        return this.withdrawFromPositionIx(
          {
            user: params.user,
            market: marketPubkey,
            mint: params.mint,
            amount: params.amount,
            outcomeIndex: ctx.outcomeIndex,
          },
          ctx.isToken2022,
        );
      }
    }
  }

  withdrawTx(params: WithdrawParams): Transaction {
    const ix = this.withdrawIx(params);
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
