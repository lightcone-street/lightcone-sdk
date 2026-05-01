import { type PublicKey } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { requireConnection } from "../../context";
import { SdkError } from "../../error";
import { ProgramSdkError } from "../../program/error";
import { RetryPolicy } from "../../http";
import {
  getMarketPda,
  getAllConditionalMintPdas,
} from "../../program/pda";
import { deserializeMarket as deserializeProgramMarket } from "../../program/accounts";
import { deriveConditionId } from "../../program/utils";
import type { Market as ProgramMarket } from "../../program/types";
import { globalDepositAssetFromWire, marketFromWire } from "./convert";
import { Status, type GlobalDepositAsset, type Market } from "./index";
import type {
  DepositMintsResponse,
  GlobalDepositAssetsListResponse,
  MarketSearchResult,
  MarketsResponse,
  SingleMarketResponse,
} from "./wire";

export interface MarketsResult {
  markets: Market[];
  validationErrors: string[];
}

/**
 * Result of fetching the global deposit asset whitelist. Assets that fail
 * validation are skipped with their errors reported separately.
 */
export interface GlobalDepositAssetsResult {
  assets: GlobalDepositAsset[];
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

  /**
   * Fetch deposit assets registered for a specific market, including each
   * asset's conditional mints.
   */
  async depositAssets(marketPubkey: string): Promise<DepositMintsResponse> {
    const url = `${this.client.http.baseUrl()}/api/markets/${encodeURIComponent(marketPubkey)}/deposit-assets`;
    return this.client.http.get<DepositMintsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Fetch the active global deposit asset whitelist (platform-scoped, not
   * market-bound).
   *
   * Assets that fail validation are skipped and their errors are returned in
   * `GlobalDepositAssetsResult.validationErrors`.
   */
  async globalDepositAssets(): Promise<GlobalDepositAssetsResult> {
    const url = `${this.client.http.baseUrl()}/api/global-deposit-assets`;
    const response = await this.client.http.get<GlobalDepositAssetsListResponse>(
      url,
      RetryPolicy.Idempotent
    );

    const assets: GlobalDepositAsset[] = [];
    const validationErrors: string[] = [];
    for (const wireAsset of response.assets) {
      try {
        assets.push(globalDepositAssetFromWire(wireAsset));
      } catch (error) {
        validationErrors.push(error instanceof Error ? error.message : String(error));
      }
    }

    return { assets, validationErrors };
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
