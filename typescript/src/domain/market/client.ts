import { Transaction, type PublicKey, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { requireConnection } from "../../context";
import { MintCompleteSetBuilder, MergeCompleteSetBuilder } from "./builders";
import { SdkError } from "../../error";
import { ProgramSdkError } from "../../program/error";
import { RetryPolicy } from "../../http";
import {
  buildMintCompleteSetIx,
  buildMergeCompleteSetIx,
} from "../../program/instructions";
import {
  getMarketPda,
  getAllConditionalMintPdas,
} from "../../program/pda";
import { deserializeMarket as deserializeProgramMarket } from "../../program/accounts";
import { deriveConditionId } from "../../program/utils";
import type {
  Market as ProgramMarket,
  MintCompleteSetParams,
  MergeCompleteSetParams,
} from "../../program/types";
import { marketFromWire } from "./convert";
import { Status, type Market } from "./index";
import type { MarketSearchResult, MarketsResponse, SingleMarketResponse } from "./wire";

export interface MarketsResult {
  markets: Market[];
  validationErrors: string[];
}

export class Markets {
  constructor(private readonly client: ClientContext) {}

  // ── PDA helpers ──────────────────────────────────────────────────────

  pda(marketId: bigint): PublicKey {
    return getMarketPda(marketId, this.client.programId)[0];
  }

  // ── Market helpers ───────────────────────────────────────────────────

  deriveConditionId(
    oracle: PublicKey,
    questionId: Buffer,
    numOutcomes: number
  ): Buffer {
    return deriveConditionId(oracle, questionId, numOutcomes);
  }

  getConditionalMints(
    market: PublicKey,
    depositMint: PublicKey,
    numOutcomes: number
  ): PublicKey[] {
    return getAllConditionalMintPdas(
      market,
      depositMint,
      numOutcomes,
      this.client.programId
    ).map(([mint]) => mint);
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  async get(cursor?: number, limit?: number): Promise<MarketsResult> {
    const search = new URLSearchParams();
    if (cursor !== undefined) search.set("cursor", String(cursor));
    if (limit !== undefined) search.set("limit", String(limit));

    const suffix = search.size > 0 ? `?${search.toString()}` : "";
    const url = `${this.client.http.baseUrl()}/api/markets${suffix}`;

    const response = await this.client.http.get<MarketsResponse>(url, RetryPolicy.Idempotent);
    const markets: Market[] = [];
    const validationErrors: string[] = [];

    for (const marketWire of response.markets) {
      try {
        const market = marketFromWire(marketWire);
        if (market.status === Status.Active || market.status === Status.Resolved) {
          markets.push(market);
        }
      } catch (error) {
        validationErrors.push(error instanceof Error ? error.message : String(error));
      }
    }

    return { markets, validationErrors };
  }

  async getBySlug(slug: string): Promise<Market> {
    const url = `${this.client.http.baseUrl()}/api/markets/by-slug/${encodeURIComponent(slug)}`;
    const response = await this.client.http.get<SingleMarketResponse>(url, RetryPolicy.Idempotent);

    try {
      return marketFromWire(response.market);
    } catch (error) {
      throw SdkError.validation(error instanceof Error ? error.message : String(error));
    }
  }

  async getByPubkey(pubkey: string): Promise<Market> {
    const url = `${this.client.http.baseUrl()}/api/markets/${encodeURIComponent(pubkey)}`;
    const response = await this.client.http.get<SingleMarketResponse>(url, RetryPolicy.Idempotent);

    try {
      return marketFromWire(response.market);
    } catch (error) {
      throw SdkError.validation(error instanceof Error ? error.message : String(error));
    }
  }

  async search(query: string, limit?: number): Promise<MarketSearchResult[]> {
    const encoded = encodeURIComponent(query);
    const suffix = limit !== undefined ? `?limit=${limit}` : "";
    const url = `${this.client.http.baseUrl()}/api/markets/search/by-query/${encoded}${suffix}`;
    return this.client.http.get<MarketSearchResult[]>(url, RetryPolicy.Idempotent);
  }

  async featured(): Promise<MarketSearchResult[]> {
    const url = `${this.client.http.baseUrl()}/api/markets/search/featured`;
    const result = await this.client.http.get<MarketSearchResult[]>(url, RetryPolicy.Idempotent);
    return result.filter(
      (item) => item.market_status === Status.Active || item.market_status === Status.Resolved
    );
  }

  // ── On-chain transaction builders ────────────────────────────────────

  mintCompleteSetIx(
    params: MintCompleteSetParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildMintCompleteSetIx(params, numOutcomes, this.client.programId);
  }

  mergeCompleteSetIx(
    params: MergeCompleteSetParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildMergeCompleteSetIx(params, numOutcomes, this.client.programId);
  }

  // ── Transaction builders (_tx convenience wrappers) ─────────────────

  mintCompleteSetTx(
    params: MintCompleteSetParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.mintCompleteSetIx(params, numOutcomes);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  mergeCompleteSetTx(
    params: MergeCompleteSetParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.mergeCompleteSetIx(params, numOutcomes);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  // ── Builder factories ──────────────────────────────────────────────

  mintCompleteSet(): MintCompleteSetBuilder {
    return new MintCompleteSetBuilder(this.client);
  }

  mergeCompleteSet(): MergeCompleteSetBuilder {
    return new MergeCompleteSetBuilder(this.client);
  }

  // ── On-chain account fetchers (require Connection) ──────────────────

  async getOnchain(market: PublicKey): Promise<ProgramMarket> {
    const connection = requireConnection(this.client);
    const accountInfo = await connection.getAccountInfo(market);
    if (!accountInfo) {
      throw ProgramSdkError.accountNotFound(`Market at ${market.toBase58()}`);
    }
    return deserializeProgramMarket(accountInfo.data as Buffer);
  }

  async getByIdOnchain(marketId: bigint): Promise<ProgramMarket> {
    const marketPda = this.pda(marketId);
    return this.getOnchain(marketPda);
  }

  async nextId(): Promise<bigint> {
    const { Rpc } = await import("../../rpc");
    const rpc = new Rpc(this.client);
    const exchange = await rpc.getExchange();
    return exchange.marketCount;
  }
}
