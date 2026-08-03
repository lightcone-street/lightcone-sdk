import { Transaction, type PublicKey, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { requireConnection } from "../../context";
import { ProgramSdkError } from "../../program/error";
import { RetryPolicy } from "../../http";
import {
  buildCloseOrderbookAltIx,
  buildCloseOrderbookIx,
} from "../../program/instructions";
import { getOrderbookPda } from "../../program/pda";
import { deserializeOrderbook as deserializeProgramOrderbook } from "../../program/accounts";
import type {
  CloseOrderbookAltParams,
  CloseOrderbookParams,
  Orderbook as ProgramOrderbook,
} from "../../program/types";
import {
  FULL_PRECISION,
  validateAggregation,
  type BookAggregation,
} from "./aggregation";
import type { OrderbookRules } from "../../shared";
import {
  orderbookRulesFromWire,
  tradingRulesFromWire,
  type DecimalsResponse,
  type OrderbookDepthResponse,
  type TradingRulesWire,
} from "./wire";

export class Orderbooks {
  private readonly rulesCache: Map<string, Promise<OrderbookRules>>;

  constructor(private readonly client: ClientContext) {
    this.rulesCache = client.orderbookRulesCache ?? new Map();
  }

  // ── PDA helpers ──────────────────────────────────────────────────────

  pda(mintA: PublicKey, mintB: PublicKey): PublicKey {
    return getOrderbookPda(mintA, mintB, this.client.programId)[0];
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  /**
   * Get live orderbook depth, optionally aggregated (Hyperliquid-style).
   *
   * `depth` is capped server-side at 20 levels per side (omitted, `0`, or
   * `>20` all serve 20). Invalid aggregation combinations throw client-side
   * before any request is made (the server would 400 with
   * `INVALID_ORDERBOOK_QUERY`), and unknown query params are rejected
   * server-side — only `depth`, `nSigFigs`, and `mantissa` are ever sent.
   */
  async get(
    orderbookId: string,
    depth?: number,
    aggregation: BookAggregation = FULL_PRECISION
  ): Promise<OrderbookDepthResponse> {
    const validated = validateAggregation(aggregation);
    const query = new URLSearchParams();
    if (depth !== undefined) {
      query.set("depth", String(depth));
    }
    if (validated.nSigFigs !== undefined) {
      query.set("nSigFigs", String(validated.nSigFigs));
    }
    if (validated.mantissa !== undefined) {
      query.set("mantissa", String(validated.mantissa));
    }
    const queryString = query.toString();
    const url = `${this.client.http.baseUrl()}/api/orderbook/${encodeURIComponent(orderbookId)}${queryString ? `?${queryString}` : ""}`;
    const raw = await this.client.http.get<
      Omit<OrderbookDepthResponse, "trading_rules" | "revision" | "captured_at_ms" | "bids_truncated" | "asks_truncated"> & {
        trading_rules: TradingRulesWire;
        revision: number | bigint;
        captured_at_ms: number | bigint;
        bids_truncated?: boolean;
        asks_truncated?: boolean;
      }
    >(url, RetryPolicy.Idempotent);
    if (
      typeof raw.price_quantum !== "string" ||
      !raw.trading_rules ||
      !raw.decimals ||
      !Number.isSafeInteger(raw.decimals.price) ||
      !Number.isSafeInteger(raw.decimals.size) ||
      raw.decimals.price < 0 ||
      raw.decimals.size < 0 ||
      (raw.bids_truncated !== undefined && typeof raw.bids_truncated !== "boolean") ||
      (raw.asks_truncated !== undefined && typeof raw.asks_truncated !== "boolean") ||
      raw.revision === undefined ||
      raw.captured_at_ms === undefined
    ) {
      throw ProgramSdkError.serialization(
        "orderbook depth is missing required projection or trading-rule metadata"
      );
    }
    const revision = BigInt(raw.revision);
    const capturedAtMs = BigInt(raw.captured_at_ms);
    if (revision < 0n || capturedAtMs < 0n) {
      throw ProgramSdkError.serialization(
        "orderbook projection revision and capture time must be non-negative"
      );
    }
    return {
      ...raw,
      trading_rules: tradingRulesFromWire(raw.trading_rules),
      revision,
      captured_at_ms: capturedAtMs,
      bids_truncated: raw.bids_truncated ?? false,
      asks_truncated: raw.asks_truncated ?? false,
    };
  }

  /** Fetch and cache the immutable exact admission rules for an active book. */
  async decimals(orderbookId: string): Promise<OrderbookRules> {
    const cache = this.rulesCache;
    const existing = cache.get(orderbookId);
    if (existing) return existing;
    const request = this.client.http
      .get<DecimalsResponse>(
        `${this.client.http.baseUrl()}/api/orderbooks/${encodeURIComponent(orderbookId)}/decimals`,
        RetryPolicy.Idempotent
      )
      .then(orderbookRulesFromWire);
    cache.set(orderbookId, request);
    request.catch(() => {
      if (cache.get(orderbookId) === request) cache.delete(orderbookId);
    });
    return request;
  }

  invalidateDecimals(orderbookId: string): void {
    this.rulesCache.delete(orderbookId);
  }

  clearDecimalsCache(): void {
    this.rulesCache.clear();
  }

  // ── On-chain transaction builders ────────────────────────────────────

  closeOrderbookAltIx(params: CloseOrderbookAltParams): TransactionInstruction {
    return buildCloseOrderbookAltIx(params, this.client.programId);
  }

  closeOrderbookIx(params: CloseOrderbookParams): TransactionInstruction {
    return buildCloseOrderbookIx(params, this.client.programId);
  }

  closeOrderbookAltTx(params: CloseOrderbookAltParams): Transaction {
    const ix = this.closeOrderbookAltIx(params);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }

  closeOrderbookTx(params: CloseOrderbookParams): Transaction {
    const ix = this.closeOrderbookIx(params);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }

  // ── On-chain account fetchers (require Connection) ──────────────────

  async getOnchain(mintA: PublicKey, mintB: PublicKey): Promise<ProgramOrderbook> {
    const connection = requireConnection(this.client);
    const orderbookPda = this.pda(mintA, mintB);
    const accountInfo = await connection.getAccountInfo(orderbookPda);
    if (!accountInfo) {
      throw ProgramSdkError.accountNotFound(
        `Orderbook for ${mintA.toBase58()} / ${mintB.toBase58()}`
      );
    }
    return deserializeProgramOrderbook(accountInfo.data as Buffer);
  }
}
