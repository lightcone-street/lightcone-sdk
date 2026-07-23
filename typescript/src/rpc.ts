import type {
  Connection,
  PublicKey,
  SignatureStatus,
  SignatureStatusConfig,
} from "@solana/web3.js";
import type { ClientContext } from "./context";
import { requireConnection, connectionWithFailover } from "./context";
import { SdkError } from "./error";
import { sleep } from "./rpcFailover";
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

// ── Transaction confirmation ──────────────────────────────────────────────

/** Interval between polls while awaiting transaction confirmation. */
const CONFIRMATION_POLL_INTERVAL_MS = 800;

/**
 * Hard cap on confirmation poll iterations (~90 s at the poll interval) — a
 * backstop for when block-height expiry cannot be observed (e.g. a
 * failed-over RPC node with a skewed view of the chain).
 */
const MAX_CONFIRMATION_POLLS = 110;

/** Consecutive failed polls tolerated before the outcome is declared unknown. */
const MAX_CONSECUTIVE_POLL_FAILURES = 3;

/**
 * Consecutive over-bound block-height samples required before expiry may be
 * declared — a single reading can come from a forward-skewed RPC node.
 */
const EXPIRY_HEIGHT_SAMPLES = 2;

/** True once the cluster has voted the transaction to `confirmed` or beyond. */
function isTransactionConfirmed(status: SignatureStatus): boolean {
  return (
    status.confirmationStatus === "confirmed" ||
    status.confirmationStatus === "finalized"
  );
}

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

  /** Get the current block height at `confirmed` commitment. */
  async getBlockHeight(): Promise<number> {
    return connectionWithFailover(this.client, (connection) =>
      connection.getBlockHeight("confirmed")
    );
  }

  /**
   * Get the statuses of recently submitted transactions.
   *
   * Returns one entry per signature, in order; `null` means the cluster has
   * not seen the signature (or, unless `config.searchTransactionHistory` is
   * set, it has aged out of the recent-status cache).
   */
  async getSignatureStatuses(
    signatures: string[],
    config?: SignatureStatusConfig
  ): Promise<(SignatureStatus | null)[]> {
    const response = await connectionWithFailover(this.client, (connection) =>
      connection.getSignatureStatuses(signatures, config)
    );
    return response.value;
  }

  /**
   * Wait until `signature` reaches `confirmed` commitment, or throw a
   * terminal `SdkError`.
   *
   * Polls `getSignatureStatuses` (with automatic failover) until the cluster
   * reports the transaction as `confirmed` or `finalized`.
   * `lastValidBlockHeight` bounds the wait: pass the height returned
   * alongside the transaction's blockhash, or `null` when the submitted
   * transaction's blockhash cannot be proven (e.g. an external signer may
   * have replaced it) — expiry is then never reported and only the poll cap
   * ends the wait. Terminal outcomes:
   *
   * - `"TransactionFailed"` — the transaction landed but errored on-chain;
   *   resubmitting the same transaction would fail again.
   * - `"TransactionExpired"` — the chain moved past `lastValidBlockHeight`
   *   on consecutive height samples and a history-searching status check
   *   still cannot see the signature; the transaction can never land and is
   *   safe to resubmit.
   * - `"ConfirmationTimeout"` — the outcome could not be determined
   *   (persistent RPC errors or the poll cap); check the signature on-chain
   *   before resubmitting.
   */
  async confirmSignature(
    signature: string,
    lastValidBlockHeight: number | null
  ): Promise<void> {
    let consecutiveFailures = 0;
    let overBoundSamples = 0;

    for (let poll = 0; poll < MAX_CONFIRMATION_POLLS; poll++) {
      let statuses: (SignatureStatus | null)[] | undefined;
      try {
        statuses = await this.getSignatureStatuses([signature]);
        consecutiveFailures = 0;
      } catch {
        consecutiveFailures += 1;
        if (consecutiveFailures >= MAX_CONSECUTIVE_POLL_FAILURES) {
          throw SdkError.confirmationTimeout(signature);
        }
      }

      if (statuses) {
        const status = statuses[0];
        if (status && isTransactionConfirmed(status)) {
          if (status.err) {
            throw SdkError.transactionFailed(
              signature,
              JSON.stringify(status.err)
            );
          }
          return;
        }
        if (status) {
          // Seen but below `confirmed` — keep waiting (failed transactions
          // land in blocks like any other, so an on-chain error is also
          // reported once confirmed) and restart expiry evidence: a sighting
          // means the transaction is live, so expiry must be re-proven from
          // scratch afterwards.
          overBoundSamples = 0;
        }
        if (!status && lastValidBlockHeight !== null) {
          // Unseen — sample the block height. Expiry requires
          // EXPIRY_HEIGHT_SAMPLES consecutive over-bound samples (a single
          // reading can come from a forward-skewed node, and each sample
          // follows a fresh unseen status), then is still verified against
          // ledger history before being declared.
          try {
            const blockHeight = await this.getBlockHeight();
            overBoundSamples =
              blockHeight > lastValidBlockHeight ? overBoundSamples + 1 : 0;
          } catch {
            // Height unavailable — don't count this poll.
          }
          if (overBoundSamples >= EXPIRY_HEIGHT_SAMPLES) {
            // Search ledger history before declaring expiry — the
            // recent-status cache can evict landed transactions, and
            // `"TransactionExpired"` promises resubmit safety.
            let history: (SignatureStatus | null)[] | undefined;
            try {
              history = await this.getSignatureStatuses([signature], {
                searchTransactionHistory: true,
              });
            } catch {
              // Could not verify — keep polling until the cap.
            }
            if (history) {
              const landed = history[0];
              if (!landed) {
                throw SdkError.transactionExpired(signature);
              }
              if (isTransactionConfirmed(landed)) {
                if (landed.err) {
                  throw SdkError.transactionFailed(
                    signature,
                    JSON.stringify(landed.err)
                  );
                }
                return;
              }
              // Landed but below `confirmed` — keep waiting and restart
              // expiry evidence.
              overBoundSamples = 0;
            }
          }
        }
      }

      await sleep(CONFIRMATION_POLL_INTERVAL_MS);
    }

    throw SdkError.confirmationTimeout(signature);
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
