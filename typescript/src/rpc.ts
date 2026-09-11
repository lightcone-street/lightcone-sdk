import { parseJsonExact, stringifyJsonExact } from "./shared/json";
import {
  V1Transaction,
  validateV1Resources,
  type V1ResourceConfig,
  type V1TransactionContext,
} from "./program/transaction";
import type {
  Connection,
  PublicKey,
  SignatureStatus,
  SignatureStatusConfig,
} from "@solana/web3.js";
import {
  AccountLayout,
  AccountState,
  ACCOUNT_SIZE,
  getAssociatedTokenAddressSync,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
  unpackAccount,
} from "@solana/spl-token";
import type { ClientContext } from "./context";
import { requireConnection, connectionWithFailover } from "./context";
import { SdkError } from "./error";
import { sleep } from "./rpcFailover";
import { ProgramSdkError } from "./program/error";
import {
  getEventAuthorityPda,
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

/** Largest lamport value that fits Solana's unsigned 64-bit account fields. */
const MAX_SOLANA_LAMPORTS = 0xffff_ffff_ffff_ffffn;

/** Convert a JSON number only when it still represents exact lamports. */
function rpcLamports(value: number | bigint, label: string): bigint {
  if (typeof value === "bigint") {
    if (value < 0n || value > MAX_SOLANA_LAMPORTS) throw SdkError.validation(`${label} must fit unsigned 64-bit lamports`);
    return value;
  }
  if (!Number.isSafeInteger(value) || value < 0) {
    throw SdkError.validation(`${label} must be a non-negative safe integer`);
  }
  return BigInt(value);
}

/**
 * Stores exact live facts for a validated canonical Tokenkeg WSOL account.
 *
 * `canonicalWsolAccountInfo` returns all fields from one confirmed account read.
 * Direct structural construction does not perform that validation.
 */
export interface CanonicalWsolAccountInfo {
  /** Full account lamports, including native amount, rent, and direct donations. */
  accountLamports: bigint;
  /** Decoded SPL native-token amount in lamports. */
  tokenAmountLamports: bigint;
  /** Decoded native-account rent reserve in lamports. */
  nativeReserveLamports: bigint;
}

/** True once the cluster has voted the transaction to `confirmed` or beyond. */
function isTransactionConfirmed(status: SignatureStatus): boolean {
  return (
    status.confirmationStatus === "confirmed" ||
    status.confirmationStatus === "finalized"
  );
}

/** Reject legacy JavaScript callers before transaction RPC work begins. */
function requireV1Transaction(transaction: unknown): asserts transaction is V1Transaction {
  if (!(transaction instanceof V1Transaction)) throw SdkError.validation("only validated Solana v1 transactions are supported");
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

  /** Event-authority PDA appended to every public instruction. */
  getEventAuthorityPda(): PublicKey {
    return getEventAuthorityPda(this.client.programId)[0];
  }

  getGlobalDepositTokenPda(mint: PublicKey): PublicKey {
    return getGlobalDepositTokenPda(mint, this.client.programId)[0];
  }

  getUserGlobalDepositPda(user: PublicKey, mint: PublicKey): PublicKey {
    return getUserGlobalDepositPda(user, mint, this.client.programId)[0];
  }

  // ── Account fetchers (async, require Connection) ──────────────────────

  /**
   * Get the latest blockhash and its expiry height, at `confirmed`
   * commitment (pinned, not the Connection's default — matching the Rust
   * and Python SDKs).
   */
  async getLatestBlockhash(): Promise<{
    blockhash: string;
    lastValidBlockHeight: number;
  }> {
    return connectionWithFailover(this.client, (connection) =>
      connection.getLatestBlockhash("confirmed")
    );
  }

  /** Get the current block height at `confirmed` commitment. */
  async getBlockHeight(): Promise<number> {
    return connectionWithFailover(this.client, (connection) =>
      connection.getBlockHeight("confirmed")
    );
  }

  /** Distinguish a missing account from an unavailable confirmed RPC read. */
  async accountExists(address: PublicKey): Promise<boolean> {
    const account = await connectionWithFailover(this.client, (connection) =>
      connection.getAccountInfo(address, "confirmed")
    );
    return account !== null;
  }

  /**
   * Return exact confirmed facts for a wallet's canonical WSOL account.
   *
   * `address` must equal the supplied wallet's Tokenkeg native-mint ATA. Missing
   * accounts return `null`. A present account must have the legacy Token Program
   * owner, native mint, wallet authority, initialized state, native reserve, and
   * no foreign close authority. Token amount plus native reserve must fit Solana's
   * unsigned 64-bit range and cannot exceed the account balance. Unsafe JSON-number
   * account lamports return an error instead of being rounded.
   */
  async canonicalWsolAccountInfo(
    address: PublicKey,
    wallet: PublicKey
  ): Promise<CanonicalWsolAccountInfo | null> {
    let canonicalAddress: PublicKey;
    try {
      canonicalAddress = getAssociatedTokenAddressSync(
        NATIVE_MINT,
        wallet,
        false,
        TOKEN_PROGRAM_ID
      );
    } catch (error) {
      throw SdkError.validation(
        `canonical WSOL address derivation failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
    if (!address.equals(canonicalAddress)) {
      throw SdkError.validation(
        "canonical WSOL address does not match the Trading Wallet Tokenkeg ATA"
      );
    }
    const info = await connectionWithFailover(this.client, (connection) =>
      connection.getAccountInfo(address, "confirmed")
    );
    if (!info) return null;
    if (!info.owner.equals(TOKEN_PROGRAM_ID) || info.data.length !== ACCOUNT_SIZE) {
      throw SdkError.validation(
        "canonical WSOL account is not a legacy Token Program account"
      );
    }

    let rawAccount;
    try {
      rawAccount = AccountLayout.decode(info.data);
    } catch (error) {
      throw SdkError.validation(
        `canonical WSOL token account is invalid: ${error instanceof Error ? error.message : String(error)}`
      );
    }
    if (
      rawAccount.state !== AccountState.Initialized ||
      (rawAccount.delegateOption !== 0 && rawAccount.delegateOption !== 1) ||
      rawAccount.isNativeOption !== 1 ||
      (rawAccount.closeAuthorityOption !== 0 &&
        rawAccount.closeAuthorityOption !== 1)
    ) {
      throw SdkError.validation(
        "canonical WSOL token account has incompatible state or option tags"
      );
    }

    let account;
    try {
      account = unpackAccount(address, info, TOKEN_PROGRAM_ID);
    } catch (error) {
      throw SdkError.validation(
        `canonical WSOL token account is invalid: ${error instanceof Error ? error.message : String(error)}`
      );
    }
    if (
      !account.mint.equals(NATIVE_MINT) ||
      !account.owner.equals(wallet) ||
      !account.isInitialized ||
      account.isFrozen ||
      !account.isNative ||
      account.rentExemptReserve === null ||
      (account.closeAuthority !== null &&
        !account.closeAuthority.equals(wallet))
    ) {
      throw SdkError.validation(
        "canonical WSOL token account has incompatible mint, authority, or native state"
      );
    }
    const accountLamports = rpcLamports(
      info.lamports,
      "canonical WSOL account lamports"
    );
    const accountedLamports = account.amount + account.rentExemptReserve;
    if (
      accountedLamports > MAX_SOLANA_LAMPORTS ||
      accountedLamports > accountLamports
    ) {
      throw SdkError.validation(
        "canonical WSOL token amount and native reserve exceed account lamports"
      );
    }
    return {
      accountLamports,
      tokenAmountLamports: account.amount,
      nativeReserveLamports: account.rentExemptReserve,
    };
  }

  /**
   * Return whether the validated canonical WSOL account is present.
   *
   * This compatibility method delegates all address, owner, state, authority, and
   * lamport checks to `canonicalWsolAccountInfo`.
   */
  async canonicalWsolAccountExists(
    address: PublicKey,
    wallet: PublicKey
  ): Promise<boolean> {
    return (await this.canonicalWsolAccountInfo(address, wallet)) !== null;
  }

  /** Return the current rent-exempt minimum in lamports for `dataLength` account bytes. */
  async minimumBalanceForRentExemption(dataLength: number): Promise<bigint> {
    const lamports = await connectionWithFailover(this.client, (connection) =>
      connection.getMinimumBalanceForRentExemption(dataLength, "confirmed")
    );
    return rpcLamports(lamports, "rent-exempt minimum");
  }

  /** Return the confirmed Native SOL Balance for `feePayer`, in lamports. */
  async balanceLamports(feePayer: PublicKey): Promise<bigint> {
    const balance = await connectionWithFailover(this.client, (connection) =>
      connection.getBalance(feePayer, "confirmed")
    );
    return rpcLamports(balance, "fee-payer balance");
  }

  /** Fetch a blockhash/expiry pair with explicitly configured transaction resources. */
  async transactionContext(): Promise<V1TransactionContext> {
    if (!this.client.transactionResources)
      throw SdkError.validation(
        "transaction resources are required; configure transactionResources on the client builder"
      );
    return this.transactionContextWithResources(
      this.client.transactionResources
    );
  }

  /** Fetch a fresh context with caller-selected limits and total priority fee lamports. */
  async transactionContextWithResources(
    resources: V1ResourceConfig
  ): Promise<V1TransactionContext> {
    validateV1Resources(resources);
    const snapshot = Object.freeze({ ...resources });
    const lifetime = await this.getLatestBlockhash();
    return Object.freeze({ ...lifetime, resources: snapshot });
  }

  /** Estimate the immutable message without changing its blockhash or resources. */
  async prepareAndEstimateTransactionFee(
    transaction: V1Transaction
  ): Promise<bigint> {
    return this.estimatePreparedTransactionFee(transaction);
  }

  /** Return the exact message fee in lamports; a missing estimate fails closed. */
  async estimatePreparedTransactionFee(
    transaction: V1Transaction
  ): Promise<bigint> {
    requireV1Transaction(transaction);
    const params = [
      Buffer.from(transaction.messageBytes()).toString("base64"),
      { commitment: "confirmed" }
    ];
    const result = await connectionWithFailover(this.client, async connection => {
      try {
        return await this.v1Request("getFeeForMessage", params, connection);
      } catch (error) {
        // Preserve the original transport error for infrastructure classification.
        throw error instanceof SdkError && error.causeError ? error.causeError : error;
      }
    }).catch(error => { throw SdkError.from(error); }) as { value?: number | bigint | null };
    if (result.value === null || result.value === undefined)
      throw SdkError.validation("transaction fee estimate is unavailable");
    return rpcLamports(result.value, "transaction fee estimate");
  }

  /** Require feature activation in the same confirmed bank used for the account read. */
  async ensureV1Supported(): Promise<void> {
    const result = (await this.v1Request("getAccountInfo", [
      "txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL",
      { commitment: "confirmed", encoding: "base64" }
    ])) as {
      context?: { slot?: number };
      value?: {
        owner?: string;
        executable?: boolean;
        data?: [string, string];
      } | null;
    };
    const account = result.value;
    const slot = result.context?.slot;
    if (
      !account ||
      account.owner !== "Feature111111111111111111111111111111111111" ||
      account.executable !== false ||
      !Array.isArray(account.data) ||
      account.data[1] !== "base64" ||
      typeof account.data[0] !== "string" ||
      slot === undefined ||
      !Number.isSafeInteger(slot) ||
      slot < 0
    )
      throw SdkError.validation(
        "Solana v1 feature is unavailable or inactive on this RPC cluster"
      );
    const data = Buffer.from(account.data[0], "base64");
    if (
      data.toString("base64") !== account.data[0] ||
      data.length < 9 ||
      data[0] !== 1 ||
      data.readBigUInt64LE(1) > BigInt(slot)
    )
      throw SdkError.validation(
        "Solana v1 feature is unavailable or inactive on this RPC cluster"
      );
  }

  /** Simulate exact fully signed bytes with signature checks and no blockhash replacement. */
  async simulateTransaction(
    transaction: V1Transaction
  ): Promise<TransactionSimulation> {
    requireV1Transaction(transaction);
    transaction.verifySignatures();
    await this.ensureV1Supported();
    const result = (await this.v1Request("simulateTransaction", [
      Buffer.from(transaction.toWireBytes()).toString("base64"),
      {
        encoding: "base64",
        commitment: "confirmed",
        sigVerify: true,
        replaceRecentBlockhash: false
      }
    ])) as {
      context?: { slot?: number };
      value?: {
        err?: unknown;
        unitsConsumed?: number;
        loadedAccountsDataSize?: number;
        logs?: string[];
      };
    };
    if (!result.value || result.value.err !== null)
      throw SdkError.validation(
        `v1 simulation failed: ${stringifyJsonExact(result.value?.err)}`
      );
    if (
      !Number.isSafeInteger(result.context?.slot) ||
      (result.context?.slot ?? -1) < 0
    )
      throw SdkError.validation("v1 simulation response is missing its slot");
    return {
      slot: result.context?.slot as number,
      unitsConsumed: result.value.unitsConsumed,
      loadedAccountsDataSize: result.value.loadedAccountsDataSize,
      logs: result.value.logs ?? []
    };
  }

  /** Verify, simulate, and send once. An ambiguous result retains signature and expiry. */
  async submitSignedTransaction(transaction: V1Transaction): Promise<string> {
    requireV1Transaction(transaction);
    transaction.verifySignatures();
    await this.simulateTransaction(transaction);
    const signature = transaction.signature;
    try {
      const returned = await this.v1Request("sendTransaction", [
        Buffer.from(transaction.toWireBytes()).toString("base64"),
        {
          encoding: "base64",
          skipPreflight: false,
          preflightCommitment: "confirmed",
          maxRetries: 0
        }
      ]);
      if (returned !== signature)
        throw new Error(
          "RPC returned a missing or different transaction signature"
        );
      return signature;
    } catch (error) {
      throw SdkError.submissionUnknown(
        signature,
        transaction.lastValidBlockHeight,
        error instanceof Error ? error.message : String(error)
      );
    }
  }

  /** One HTTP attempt without the legacy Connection's rate-limit retries. */
  private async v1Request(method: string, params: unknown[], connection?: Connection): Promise<unknown> {
    try {
      const response = await (this.client.rpcFetch ?? fetch)(
        (connection ?? requireConnection(this.client)).rpcEndpoint,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
          signal: AbortSignal.timeout(180_000)
        }
      );
      if (!response.ok) throw new Error(`RPC HTTP ${response.status}`);
      const result = parseJsonExact<{ error?: unknown; result?: unknown }>(
        await response.text()
      );
      if (result.error !== undefined)
        throw new Error(
          `RPC ${method} failed: ${stringifyJsonExact(result.error)}`
        );
      if (!("result" in result))
        throw new Error(`RPC ${method} omitted its result`);
      return result.result;
    } catch (error) {
      throw SdkError.from(error);
    }
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
   * transaction's original lifetime is unknown (for example a historical
   * signature imported without its context) — expiry is then never reported and only the poll cap
   * ends the wait. Terminal outcomes:
   *
   * - `"TransactionFailed"` — the transaction landed but errored on-chain;
   *   resubmitting the same transaction would fail again.
   * - `"TransactionExpired"` — the chain moved past `lastValidBlockHeight`
   *   on consecutive height samples and a history-searching status check
   *   still cannot see the signature; reconcile the signature and authoritative state before rebuilding.
   * - `"ConfirmationTimeout"` — the outcome could not be determined
   *   (persistent RPC errors or the poll cap); check the signature on-chain
   *   before resubmitting.
   */
  async confirmSignature(
    signature: string,
    lastValidBlockHeight: number | null
  ): Promise<void> {
    await this.confirmSignatureStatus(signature, lastValidBlockHeight);
  }

  /**
   * Same as {@link confirmSignature}, but returns the confirmed status so
   * callers can use the transaction's processing slot.
   */
  async confirmSignatureStatus(
    signature: string,
    lastValidBlockHeight: number | null
  ): Promise<SignatureStatus> {
    let consecutiveFailures = 0;
    let overBoundSamples = 0;

    for (let poll = 0; poll < MAX_CONFIRMATION_POLLS; poll++) {
      let statuses: (SignatureStatus | null)[] | undefined;
      try {
        statuses = await this.getSignatureStatuses([signature]);
        consecutiveFailures = 0;
      } catch {
        consecutiveFailures += 1;
        // A failed poll is a gap in expiry evidence — restart it.
        overBoundSamples = 0;
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
          return status;
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
            // Height unavailable — reset: expiry evidence must be strictly
            // consecutive over-bound readings.
            overBoundSamples = 0;
          }
          if (overBoundSamples >= EXPIRY_HEIGHT_SAMPLES) {
            // Search ledger history before declaring expiry — the
            // recent-status cache can evict landed transactions, and
            // expiry still requires final history evidence.
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
                return landed;
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

/** Resource usage reported for the exact signed v1 message. */
export interface TransactionSimulation {
  slot: number;
  unitsConsumed?: number;
  loadedAccountsDataSize?: number;
  logs: string[];
}
