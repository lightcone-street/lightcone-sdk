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
import type { OrderbookDepthResponse } from "./wire";

export class Orderbooks {
  constructor(private readonly client: ClientContext) {}

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
