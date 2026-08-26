import { Transaction, type Connection, type PublicKey } from "@solana/web3.js";
import { SdkError } from "./error";
import type { LightconeHttp } from "./http";
import type { AuthCredentials } from "./auth";
import type { DepositSource, OrderbookRules } from "./shared";
import {
  signingStrategyWalletAddress,
  type SigningStrategy,
} from "./shared/signing";
import {
  ActiveRpc,
  type RpcFailoverState,
  isInfrastructureError,
  sleep,
  FAST_RETRY_DELAY_MS,
} from "./rpcFailover";

export interface ClientContext {
  readonly http: LightconeHttp;
  readonly programId: PublicKey;
  readonly primaryConnection?: Connection;
  readonly backupConnection?: Connection;
  readonly rpcFailoverState: RpcFailoverState;
  readonly depositSource: DepositSource;
  readonly signingStrategy?: SigningStrategy;
  /** Trusted application assertion that an external sponsor pays fees; omission is false. */
  readonly transactionSponsorshipEnabled?: boolean;
  /** Optional cached identity for auth-bound operations; callers must check expiry. */
  readonly authCredentials?: AuthCredentials;
  orderNonce?(): number | undefined;
  setOrderNonce?(nonce: number): void;
  readonly orderbookRulesCache?: Map<string, Promise<OrderbookRules>>;

  /** @deprecated Use primaryConnection — kept for backward compat in domain sub-clients. */
  readonly connection?: Connection;
}

export function requireConnection(ctx: ClientContext): Connection {
  ctx.rpcFailoverState.maybeRecoverToPrimary();
  const conn =
    ctx.rpcFailoverState.active === ActiveRpc.Primary
      ? ctx.primaryConnection
      : ctx.backupConnection ?? ctx.primaryConnection;
  if (!conn) {
    throw SdkError.validation(
      "RPC client not configured — use .rpcUrl() on the builder"
    );
  }
  return conn;
}

function resolveConnectionFor(
  ctx: ClientContext,
  target: ActiveRpc
): Connection | undefined {
  return target === ActiveRpc.Primary
    ? ctx.primaryConnection
    : ctx.backupConnection;
}

export async function connectionWithFailover<T>(
  ctx: ClientContext,
  operation: (conn: Connection) => Promise<T>
): Promise<T> {
  ctx.rpcFailoverState.maybeRecoverToPrimary();
  const originalActive = ctx.rpcFailoverState.active;
  const activeConn = requireConnection(ctx);

  // First attempt.
  try {
    return await operation(activeConn);
  } catch (firstError) {
    if (!isInfrastructureError(firstError)) throw firstError;
  }

  // Fast retry on same connection.
  await sleep(FAST_RETRY_DELAY_MS);
  try {
    return await operation(activeConn);
  } catch (retryError) {
    if (!isInfrastructureError(retryError)) throw retryError;

    // Flip and try the other connection.
    const otherTarget =
      originalActive === ActiveRpc.Primary
        ? ActiveRpc.Backup
        : ActiveRpc.Primary;
    const otherConn = resolveConnectionFor(ctx, otherTarget);
    if (otherConn) {
      try {
        const result = await operation(otherConn);
        if (otherTarget === ActiveRpc.Primary) {
          ctx.rpcFailoverState.flipToPrimary();
        } else {
          ctx.rpcFailoverState.flipToBackup();
        }
        return result;
      } catch (bothError) {
        throw bothError;
      }
    }
    throw retryError;
  }
}

export function resolveDepositSource(
  ctx: ClientContext,
  overrideSource: DepositSource | undefined
): DepositSource {
  return overrideSource ?? ctx.depositSource;
}

export function requireSigningStrategy(ctx: ClientContext): SigningStrategy {
  if (!ctx.signingStrategy) {
    throw SdkError.validation(
      "Signing strategy not configured — use .nativeSigner(), .externalSigner(), or .privyWalletId() on the builder"
    );
  }
  return ctx.signingStrategy;
}

/** Capture one signer and sponsorship assertion before transaction work can yield. */
function requireTransactionSigningContext(ctx: ClientContext): {
  strategy: SigningStrategy;
  sponsorshipEnabled: boolean;
} {
  return {
    strategy: requireSigningStrategy(ctx),
    sponsorshipEnabled: ctx.transactionSponsorshipEnabled ?? false,
  };
}

/**
 * Reject invalid payer and sponsorship combinations before submission can yield.
 * Unsponsored known signers must control the payer being classified; sponsored
 * external flows may intentionally use a different payer, while native sponsorship
 * is rejected before blockhash RPC or caller-transaction mutation.
 */
function validateTransactionFeeFundingContext(
  tx: import("@solana/web3.js").Transaction,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean
): void {
  if (!tx.feePayer) {
    throw SdkError.validation("transaction is missing a declared fee payer");
  }
  if (sponsorshipEnabled) {
    if (strategy.type === "native") {
      throw SdkError.validation(
        "transaction sponsorship is not supported with local-keypair signing"
      );
    }
    return;
  }

  const signingAddress = signingStrategyWalletAddress(strategy);
  if (signingAddress && signingAddress !== tx.feePayer.toBase58()) {
    throw SdkError.validation(
      "signing strategy does not control transaction fee payer"
    );
  }
}

/**
 * Own the transaction message before asynchronous submission work begins.
 * A placeholder blockhash exists only long enough to serialize an ordinary
 * transaction that has not received its fresh blockhash yet.
 */
function snapshotTransaction(tx: Transaction): Transaction {
  if (tx.recentBlockhash || tx.nonceInfo) {
    return Transaction.from(
      tx.serialize({ requireAllSignatures: false, verifySignatures: false })
    );
  }

  const serializable = new Transaction({
    feePayer: tx.feePayer,
    blockhash: "11111111111111111111111111111111",
    lastValidBlockHeight: tx.lastValidBlockHeight ?? 0,
    signatures: tx.signatures,
  });
  serializable.instructions = tx.instructions;
  const snapshot = Transaction.from(
    serializable.serialize({ requireAllSignatures: false, verifySignatures: false })
  );
  snapshot.recentBlockhash = undefined;
  snapshot.lastValidBlockHeight = undefined;
  return snapshot;
}

/** Publish local signatures only when the caller still holds the submitted message. */
function copySignaturesIfMessageUnchanged(
  signed: Transaction,
  callerTransaction: Transaction
): void {
  try {
    if (
      !Buffer.from(signed.serializeMessage()).equals(
        Buffer.from(callerTransaction.serializeMessage())
      )
    ) {
      return;
    }
  } catch {
    return;
  }
  callerTransaction.signatures = signed.signatures.map(({ publicKey, signature }) => ({
    publicKey,
    signature: signature ? Buffer.from(signature) : null,
  }));
}

/**
 * Reject proven fee shortfalls before signing while preserving submission on unknown evidence.
 *
 * The transaction's prepared message supplies the exact fee and declared fee
 * payer. Fee or balance lookup failure is deliberately best-effort; planner-owned
 * SOL admission remains fail-closed before reaching this shared boundary. The
 * signer and sponsorship value were captured together before RPC work.
 */
async function preflightTransactionFeeFunding(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean
): Promise<void> {
  validateTransactionFeeFundingContext(tx, strategy, sponsorshipEnabled);
  if (sponsorshipEnabled) return;
  const feePayer = tx.feePayer;
  if (!feePayer) {
    throw SdkError.validation("transaction is missing a declared fee payer");
  }

  const { Rpc } = await import("./rpc");
  const rpc = new Rpc(ctx);
  let requiredLamports: bigint;
  try {
    requiredLamports = await rpc.estimatePreparedTransactionFee(tx);
  } catch {
    return;
  }
  let availableLamports: bigint;
  try {
    availableLamports = await rpc.balanceLamports(feePayer);
  } catch {
    return;
  }
  if (availableLamports < requiredLamports) {
    throw SdkError.insufficientSolForTransactionFees(
      availableLamports,
      requiredLamports
    );
  }
}

/**
 * Sign and submit a transaction using the client's signing strategy.
 *
 * Fetches a recent blockhash automatically. Returns as soon as the RPC
 * accepts the transaction — inclusion is not awaited. When follow-up work
 * depends on this transaction's on-chain effects, use
 * {@link signAndSubmitTxConfirmed} instead.
 * Unsponsored submission checks exact fee funding before signing when both
 * required RPC observations are available.
 */
export async function signAndSubmitTx(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction
): Promise<string> {
  const { strategy, sponsorshipEnabled } = requireTransactionSigningContext(ctx);
  const { signature } = await signAndSubmitTxInner(ctx, tx, strategy, sponsorshipEnabled);
  return signature;
}

/**
 * Sign and submit a transaction, then wait until it reaches `confirmed`
 * commitment on-chain.
 *
 * Sequential flows should prefer this over {@link signAndSubmitTx}: a
 * transaction that depends on a prior transaction's state is only safe to
 * send once that prior transaction has confirmed. See `Rpc.confirmSignature`
 * for the terminal error taxonomy.
 *
 * Expiry (`"TransactionExpired"`) is only ever reported when the submitted
 * transaction provably still carries the blockhash this function fetched:
 * always true for `native`, verified against the signed bytes for
 * `walletAdapter`, and never assumed for `privy` (the backend signs and
 * submits out of the SDK's sight) — those cases end in
 * `"ConfirmationTimeout"` at the poll cap instead.
 */
export async function signAndSubmitTxConfirmed(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction
): Promise<string> {
  const confirmed = await signAndSubmitTxConfirmedWithSlot(ctx, tx);
  return confirmed.signature;
}

export interface ConfirmedTransaction {
  signature: string;
  slot: number;
}

/**
 * Sign and submit a transaction, wait for confirmed commitment, and return
 * both its signature and processing slot.
 */
export async function signAndSubmitTxConfirmedWithSlot(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction
): Promise<ConfirmedTransaction> {
  const { strategy, sponsorshipEnabled } = requireTransactionSigningContext(ctx);
  return signAndSubmitTxConfirmedWithSlotUsingStrategy(
    ctx,
    tx,
    strategy,
    sponsorshipEnabled
  );
}

/**
 * Sign, submit once, and confirm a transaction whose message was fee-estimated.
 *
 * This function preserves the prepared blockhash. A wallet adapter may add
 * signatures but may not replace any message field. Signed bytes are sent once to
 * the active RPC because a transport failure may occur after acceptance.
 * The unchanged message receives the same best-effort fee-funding preflight as an
 * ordinary transaction before the signer runs. A sponsored external signer may
 * differ from the declared fee payer. The caller-owned transaction is snapshotted
 * before the first await so later caller mutation cannot change fee authority.
 */
export async function signAndSubmitPreparedTxConfirmedWithSlot(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction
): Promise<ConfirmedTransaction> {
  if (!tx.recentBlockhash) {
    throw SdkError.validation("prepared transaction is missing a recent blockhash");
  }
  const { strategy, sponsorshipEnabled } = requireTransactionSigningContext(ctx);
  if (!tx.feePayer) {
    throw SdkError.validation("prepared transaction is missing a fee payer");
  }
  validateTransactionFeeFundingContext(tx, strategy, sponsorshipEnabled);
  if (!sponsorshipEnabled && !signingStrategyWalletAddress(strategy)) {
    throw SdkError.validation("signing strategy wallet identity is required");
  }
  const callerTransaction = tx;
  tx = snapshotTransaction(tx);
  await preflightTransactionFeeFunding(ctx, tx, strategy, sponsorshipEnabled);
  const signature = await signAndSubmitPreparedTxInner(
    ctx,
    tx,
    strategy,
    callerTransaction
  );
  const { Rpc } = await import("./rpc");
  const status = await new Rpc(ctx).confirmSignatureStatus(signature, null);
  return { signature, slot: status.slot };
}

/** @internal Submit with a strategy already validated for an identity-bound operation. */
export async function signAndSubmitTxConfirmedUsingStrategy(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction,
  strategy: SigningStrategy
): Promise<string> {
  const sponsorshipEnabled = ctx.transactionSponsorshipEnabled ?? false;
  const confirmed = await signAndSubmitTxConfirmedWithSlotUsingStrategy(
    ctx,
    tx,
    strategy,
    sponsorshipEnabled
  );
  return confirmed.signature;
}

async function signAndSubmitTxConfirmedWithSlotUsingStrategy(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean
): Promise<ConfirmedTransaction> {
  const { signature, lastValidBlockHeight } = await signAndSubmitTxInner(
    ctx,
    tx,
    strategy,
    sponsorshipEnabled
  );
  const { Rpc } = await import("./rpc");
  const status = await new Rpc(ctx).confirmSignatureStatus(
    signature,
    lastValidBlockHeight
  );
  return { signature, slot: status.slot };
}

/**
 * Prepare funding evidence, sign, send, and return the signature and expiry bound.
 *
 * The fresh blockhash feeds best-effort fee preflight before any signer runs.
 * Submission operates on a pre-await snapshot. The fresh blockhash is copied to
 * the caller transaction; local signatures follow only if its message still matches.
 * `lastValidBlockHeight` is `null` when the submitted wire bytes cannot be proven
 * to retain that blockhash because an external signer replaced it or the final
 * bytes were never visible to the SDK.
 */
async function signAndSubmitTxInner(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean
): Promise<{ signature: string; lastValidBlockHeight: number | null }> {
  validateTransactionFeeFundingContext(tx, strategy, sponsorshipEnabled);
  const callerTransaction = tx;
  tx = snapshotTransaction(tx);
  const { isUserCancellation } = await import("./shared/signing");
  const { SdkError } = await import("./error");
  const { RetryPolicy } = await import("./http");

  // Get blockhash with failover, at `confirmed` commitment (pinned, not the
  // Connection's default — matching the Rust and Python SDKs).
  const { blockhash, lastValidBlockHeight } = await connectionWithFailover(
    ctx,
    (conn) => conn.getLatestBlockhash("confirmed")
  );
  tx.recentBlockhash = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  callerTransaction.recentBlockhash = blockhash;
  callerTransaction.lastValidBlockHeight = lastValidBlockHeight;
  await preflightTransactionFeeFunding(ctx, tx, strategy, sponsorshipEnabled);

  switch (strategy.type) {
    case "native": {
      tx.partialSign(strategy.keypair);
      copySignaturesIfMessageUnchanged(tx, callerTransaction);
      const signature = await connectionWithFailover(ctx, (conn) =>
        conn.sendRawTransaction(tx.serialize())
      );
      return { signature, lastValidBlockHeight };
    }
    case "walletAdapter": {
      const expectedMessage = tx.serializeMessage();
      const txBytes = tx.serialize({ requireAllSignatures: false });
      const signedBytes = await strategy.signer
        .signTransaction(txBytes)
        .catch((err: unknown) => {
          const msg = err instanceof Error ? err.message : String(err);
          if (isUserCancellation(msg)) throw SdkError.userCancelled();
          throw SdkError.signing(msg);
        });
      const signedBlockhashIsUnchanged = await validateOrdinarySignedTransaction(
        signedBytes,
        expectedMessage,
        blockhash
      );
      const signature = await connectionWithFailover(ctx, (conn) =>
        conn.sendRawTransaction(signedBytes)
      );
      return {
        signature,
        lastValidBlockHeight: signedBlockhashIsUnchanged
          ? lastValidBlockHeight
          : null,
      };
    }
    case "privy": {
      const txBytes = tx.serialize({ requireAllSignatures: false });
      const base64Tx = Buffer.from(txBytes).toString("base64");
      const url = `${ctx.http.baseUrl()}/api/privy/sign_and_send_tx`;
      const result = await ctx.http.post<{ hash: string }, object>(
        url,
        { wallet_id: strategy.walletId, base64_tx: base64Tx },
        RetryPolicy.None
      );
      // The backend signs and submits server-side; the SDK never sees the
      // final wire bytes, so the blockhash it set cannot be trusted for
      // expiry detection.
      return { signature: result.hash, lastValidBlockHeight: null };
    }
  }
}

/**
 * Sign and submit once without replacing the planner's prepared blockhash.
 *
 * Native signing preserves the message by construction. Wallet-adapter bytes are
 * compared with the prepared message before submission. Privy is excluded because
 * this SDK cannot inspect its final wire message. The native signature is published
 * to an unchanged caller message before the one-shot send so an uncertain outcome
 * remains reconcilable. Both admitted strategies use the active connection directly.
 */
async function signAndSubmitPreparedTxInner(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction,
  strategy: SigningStrategy,
  callerTransaction: import("@solana/web3.js").Transaction
): Promise<string> {
  const { isUserCancellation } = await import("./shared/signing");

  switch (strategy.type) {
    case "native":
      tx.partialSign(strategy.keypair);
      copySignaturesIfMessageUnchanged(tx, callerTransaction);
      return requireConnection(ctx).sendRawTransaction(tx.serialize());
    case "walletAdapter": {
      const expectedMessage = tx.serializeMessage();
      const txBytes = tx.serialize({ requireAllSignatures: false });
      const signedBytes = await strategy.signer
        .signTransaction(txBytes)
        .catch((error: unknown) => {
          const message = error instanceof Error ? error.message : String(error);
          if (isUserCancellation(message)) throw SdkError.userCancelled();
          throw SdkError.signing(message);
        });
      await assertPreparedSignedMessage(signedBytes, expectedMessage);
      return requireConnection(ctx).sendRawTransaction(signedBytes);
    }
    case "privy":
      throw SdkError.validation(
        "prepared transaction submission cannot verify a Privy-signed message"
      );
  }
}

/**
 * Enforce the fee-preflight authority boundary after wallet-adapter signing.
 * Signatures may change, but any account, instruction, fee payer, or blockhash
 * change rejects the bytes before they reach RPC.
 */
async function assertPreparedSignedMessage(
  signedBytes: Uint8Array,
  expectedMessage: Uint8Array
): Promise<void> {
  const { Transaction } = await import("@solana/web3.js");
  let signedMessage: Uint8Array;
  try {
    signedMessage = Transaction.from(signedBytes).serializeMessage();
  } catch (error) {
    throw SdkError.signing(
      `signed transaction is invalid: ${error instanceof Error ? error.message : String(error)}`
    );
  }
  if (!Buffer.from(signedMessage).equals(Buffer.from(expectedMessage))) {
    throw SdkError.validation(
      "wallet changed the fee-prepared transaction message"
    );
  }
}

/**
 * Allow an external signer to replace only the ordinary transaction blockhash.
 *
 * Fee payer, accounts, and instructions are the authority used by fee preflight
 * and must survive signing. A replacement blockhash remains allowed, but its
 * original expiry bound is discarded.
 */
async function validateOrdinarySignedTransaction(
  signedBytes: Uint8Array,
  expectedMessage: Uint8Array,
  expectedBlockhash: string
): Promise<boolean> {
  const { Transaction } = await import("@solana/web3.js");
  let signed: import("@solana/web3.js").Transaction;
  try {
    signed = Transaction.from(signedBytes);
  } catch (error) {
    throw SdkError.signing(
      `signed transaction is invalid: ${error instanceof Error ? error.message : String(error)}`
    );
  }
  const blockhashUnchanged = signed.recentBlockhash === expectedBlockhash;
  signed.recentBlockhash = expectedBlockhash;
  if (!Buffer.from(signed.serializeMessage()).equals(Buffer.from(expectedMessage))) {
    throw SdkError.validation(
      "wallet changed the transaction message beyond recent blockhash"
    );
  }
  return blockhashUnchanged;
}
