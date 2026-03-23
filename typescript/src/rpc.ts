import type { Connection, PublicKey } from "@solana/web3.js";
import type { ClientContext } from "./context";
import { requireConnection } from "./context";
import {
  getExchangePda,
  getGlobalDepositTokenPda,
  getUserGlobalDepositPda,
} from "./program/pda";
import {
  deserializeExchange,
  deserializeGlobalDepositToken,
} from "./program/accounts";
import type { Exchange, GlobalDepositToken } from "./program/types";

export class Rpc {
  constructor(private readonly client: ClientContext) {}

  /** Get the underlying Connection, or throw if not configured. */
  inner(): Connection {
    return requireConnection(this.client);
  }

  // ── PDA helpers (sync, no Connection needed) ──────────────────────────

  getExchangePda(): PublicKey {
    return getExchangePda(this.client.programId)[0];
  }

  getGlobalDepositTokenPda(mint: PublicKey): PublicKey {
    return getGlobalDepositTokenPda(mint, this.client.programId)[0];
  }

  getUserGlobalDepositPda(user: PublicKey, mint: PublicKey): PublicKey {
    return getUserGlobalDepositPda(user, mint, this.client.programId)[0];
  }

  // ── Account fetchers (async, require Connection) ──────────────────────

  async getLatestBlockhash(): Promise<{
    blockhash: string;
    lastValidBlockHeight: number;
  }> {
    const connection = requireConnection(this.client);
    return connection.getLatestBlockhash();
  }

  async getExchange(): Promise<Exchange> {
    const connection = requireConnection(this.client);
    const pda = this.getExchangePda();
    const accountInfo = await connection.getAccountInfo(pda);
    if (!accountInfo) {
      throw new Error("Exchange account not found");
    }
    return deserializeExchange(accountInfo.data as Buffer);
  }

  async getGlobalDepositToken(mint: PublicKey): Promise<GlobalDepositToken> {
    const connection = requireConnection(this.client);
    const pda = this.getGlobalDepositTokenPda(mint);
    const accountInfo = await connection.getAccountInfo(pda);
    if (!accountInfo) {
      throw new Error(
        `GlobalDepositToken not found for mint ${mint.toBase58()}`
      );
    }
    return deserializeGlobalDepositToken(accountInfo.data as Buffer);
  }
}
