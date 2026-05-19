import type { Connection, PublicKey } from "@solana/web3.js";
import type { ClientContext } from "./context";
import { requireConnection, connectionWithFailover } from "./context";
import { ProgramSdkError } from "./program/error";
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

  /**
   * Get the currently-active Connection, or throw if not configured.
   *
   * Prefer the typed methods (getExchange, etc.) — they include automatic
   * failover. Direct use of inner() bypasses the retry/failover wrapper.
   */
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
    return connectionWithFailover(this.client, (connection) =>
      connection.getLatestBlockhash()
    );
  }

  async getExchange(): Promise<Exchange> {
    const pda = this.getExchangePda();
    const accountInfo = await connectionWithFailover(
      this.client,
      (connection) => connection.getAccountInfo(pda)
    );
    if (!accountInfo) {
      throw ProgramSdkError.accountNotFound("Exchange");
    }
    return deserializeExchange(accountInfo.data as Buffer);
  }

  async getGlobalDepositToken(mint: PublicKey): Promise<GlobalDepositToken> {
    const pda = this.getGlobalDepositTokenPda(mint);
    const accountInfo = await connectionWithFailover(
      this.client,
      (connection) => connection.getAccountInfo(pda)
    );
    if (!accountInfo) {
      throw ProgramSdkError.accountNotFound(
        `GlobalDepositToken for mint ${mint.toBase58()}`
      );
    }
    return deserializeGlobalDepositToken(accountInfo.data as Buffer);
  }
}
