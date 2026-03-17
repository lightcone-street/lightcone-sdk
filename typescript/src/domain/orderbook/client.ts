import type { PublicKey } from "@solana/web3.js";
import type { ClientContext, DecimalsCache } from "../../context";
import { requireConnection } from "../../context";
import { RetryPolicy } from "../../http";
import { getOrderbookPda } from "../../program/pda";
import { deserializeOrderbook as deserializeProgramOrderbook } from "../../program/accounts";
import type { Orderbook as ProgramOrderbook } from "../../program/types";
import type { DecimalsResponse, OrderbookDepthResponse } from "./wire";

export class Orderbooks {
  constructor(
    private readonly client: ClientContext,
    private readonly cache: DecimalsCache
  ) {}

  // ── PDA helpers ──────────────────────────────────────────────────────

  pda(mintA: PublicKey, mintB: PublicKey): PublicKey {
    return getOrderbookPda(mintA, mintB, this.client.programId)[0];
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  async get(orderbookId: string, depth?: number): Promise<OrderbookDepthResponse> {
    const query = depth !== undefined ? `?depth=${depth}` : "";
    const url = `${this.client.http.baseUrl()}/api/orderbook/${encodeURIComponent(orderbookId)}${query}`;
    return this.client.http.get<OrderbookDepthResponse>(url, RetryPolicy.Idempotent);
  }

  async decimals(orderbookId: string): Promise<DecimalsResponse> {
    const cached = this.cache.get(orderbookId) as DecimalsResponse | undefined;
    if (cached) {
      return cached;
    }

    const url = `${this.client.http.baseUrl()}/api/orderbooks/${encodeURIComponent(orderbookId)}/decimals`;
    const response = await this.client.http.get<DecimalsResponse>(url, RetryPolicy.Idempotent);
    this.cache.set(orderbookId, response);
    return response;
  }

  async clearCache(): Promise<void> {
    this.cache.clear();
  }

  // ── On-chain account fetchers (require Connection) ──────────────────

  async getOnchain(mintA: PublicKey, mintB: PublicKey): Promise<ProgramOrderbook> {
    const connection = requireConnection(this.client);
    const orderbookPda = this.pda(mintA, mintB);
    const accountInfo = await connection.getAccountInfo(orderbookPda);
    if (!accountInfo) {
      throw new Error(
        `Orderbook not found for ${mintA.toBase58()} / ${mintB.toBase58()}`
      );
    }
    return deserializeProgramOrderbook(accountInfo.data as Buffer);
  }
}
