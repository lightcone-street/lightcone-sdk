import { V1Transaction, type V1ResourceConfig } from "./program/transaction";
import { type Connection, type PublicKey, type TransactionInstruction } from "@solana/web3.js";
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
  readonly transactionResources?: V1ResourceConfig;
  /** Fetch implementation used for canonical v1 JSON-RPC requests, including one-shot sends. */
  readonly rpcFetch?: typeof fetch;
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
      : (ctx.backupConnection ?? ctx.primaryConnection);
  if (!conn) {
    throw SdkError.validation(
      "RPC client not configured — use .rpcUrl() on the builder",
    );
  }
  return conn;
}

function resolveConnectionFor(
  ctx: ClientContext,
  target: ActiveRpc,
): Connection | undefined {
  return target === ActiveRpc.Primary
    ? ctx.primaryConnection
    : ctx.backupConnection;
}

export async function connectionWithFailover<T>(
  ctx: ClientContext,
  operation: (conn: Connection) => Promise<T>,
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
  overrideSource: DepositSource | undefined,
): DepositSource {
  return overrideSource ?? ctx.depositSource;
}

export function requireSigningStrategy(ctx: ClientContext): SigningStrategy {
  if (!ctx.signingStrategy) {
    throw SdkError.validation(
      "Signing strategy not configured — use .nativeSigner(), .externalSigner(), or .privyWalletId() on the builder",
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
  feePayer: PublicKey,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean,
): void {
  if (!feePayer) {
    throw SdkError.validation("transaction is missing a declared fee payer");
  }
  if (sponsorshipEnabled) {
    if (strategy.type === "native") {
      throw SdkError.validation(
        "transaction sponsorship is not supported with local-keypair signing",
      );
    }
    return;
  }

  const signingAddress = signingStrategyWalletAddress(strategy);
  if (signingAddress && signingAddress !== feePayer.toBase58()) {
    throw SdkError.validation(
      "signing strategy does not control transaction fee payer",
    );
  }
}

/** Validate local signing inputs before any transaction work can yield. */
function validateTransactionSigningContext(
  feePayer: PublicKey,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean,
): asserts strategy is Exclude<SigningStrategy, { type: "privy" }> {
  if (strategy.type === "privy")
    throw SdkError.validation(
      "Privy sign-and-send cannot verify v1 signed bytes; configure an ExternalSigner that returns signed transaction bytes",
    );
  validateTransactionFeeFundingContext(feePayer, strategy, sponsorshipEnabled);
  if (!sponsorshipEnabled && !signingStrategyWalletAddress(strategy))
    throw SdkError.validation("signing strategy wallet identity is required");
}

/** @internal Submit builder-owned instructions after local validation, before fetching context. */
export async function signAndSubmitInstructions(
  ctx: ClientContext,
  instructions: TransactionInstruction[],
  payer: PublicKey,
): Promise<string> {
  const { strategy, sponsorshipEnabled } = requireTransactionSigningContext(ctx);
  validateTransactionSigningContext(payer, strategy, sponsorshipEnabled);
  const { Rpc } = await import("./rpc");
  const context = await new Rpc(ctx).transactionContext();
  const transaction = V1Transaction.compile(instructions, payer, context);
  return signAndSubmitTxInner(ctx, transaction, strategy, sponsorshipEnabled);
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
  tx: V1Transaction,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean,
): Promise<void> {
  validateTransactionFeeFundingContext(tx.feePayer, strategy, sponsorshipEnabled);
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
      requiredLamports,
    );
  }
}

/** Confirmed signature and the slot required by authoritative balance refresh. */
export interface ConfirmedTransaction {
  signature: string;
  slot: number;
}

/** Sign and send one immutable v1 transaction with its original resources and expiry. */
export async function signAndSubmitTx(
  ctx: ClientContext,
  tx: V1Transaction,
): Promise<string> {
  const { strategy, sponsorshipEnabled } =
    requireTransactionSigningContext(ctx);
  return signAndSubmitTxInner(ctx, tx, strategy, sponsorshipEnabled);
}

/** Sign, submit once, and wait for confirmed commitment using the original expiry. */
export async function signAndSubmitTxConfirmed(
  ctx: ClientContext,
  tx: V1Transaction,
): Promise<string> {
  return (await signAndSubmitTxConfirmedWithSlot(ctx, tx)).signature;
}

/** Sign and confirm the exact prepared message, returning its processing slot. */
export async function signAndSubmitTxConfirmedWithSlot(
  ctx: ClientContext,
  tx: V1Transaction,
): Promise<ConfirmedTransaction> {
  const { strategy, sponsorshipEnabled } =
    requireTransactionSigningContext(ctx);
  return signAndSubmitTxConfirmedWithSlotUsingStrategy(
    ctx,
    tx,
    strategy,
    sponsorshipEnabled,
  );
}

/** Prepared and ordinary v1 messages use the same immutable submission contract. */
export async function signAndSubmitPreparedTxConfirmedWithSlot(
  ctx: ClientContext,
  tx: V1Transaction,
): Promise<ConfirmedTransaction> {
  return signAndSubmitTxConfirmedWithSlot(ctx, tx);
}

/** @internal Submit with an identity-bound strategy captured before planning. */
export async function signAndSubmitTxConfirmedUsingStrategy(
  ctx: ClientContext,
  tx: V1Transaction,
  strategy: SigningStrategy,
): Promise<string> {
  return (
    await signAndSubmitTxConfirmedWithSlotUsingStrategy(
      ctx,
      tx,
      strategy,
      ctx.transactionSponsorshipEnabled ?? false,
    )
  ).signature;
}

async function signAndSubmitTxConfirmedWithSlotUsingStrategy(
  ctx: ClientContext,
  tx: V1Transaction,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean,
): Promise<ConfirmedTransaction> {
  const signature = await signAndSubmitTxInner(
    ctx,
    tx,
    strategy,
    sponsorshipEnabled,
  );
  const { Rpc } = await import("./rpc");
  const status = await new Rpc(ctx).confirmSignatureStatus(
    signature,
    tx.lastValidBlockHeight,
  );
  return { signature, slot: status.slot };
}

async function signAndSubmitTxInner(
  ctx: ClientContext,
  tx: V1Transaction,
  strategy: SigningStrategy,
  sponsorshipEnabled: boolean,
): Promise<string> {
  if (!(tx instanceof V1Transaction))
    throw SdkError.validation(
      "only validated Solana v1 transactions are supported",
    );
  validateTransactionSigningContext(tx.feePayer, strategy, sponsorshipEnabled);
  await preflightTransactionFeeFunding(ctx, tx, strategy, sponsorshipEnabled);
  const { Rpc } = await import("./rpc");
  const rpc = new Rpc(ctx);
  await rpc.ensureV1Supported();
  let signed: V1Transaction;
  if (strategy.type === "native") {
    signed = tx.sign([strategy.keypair]);
  } else {
    const { isUserCancellation } = await import("./shared/signing");
    const bytes = await strategy.signer
      .signTransaction(tx.toWireBytes())
      .catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        if (isUserCancellation(message)) throw SdkError.userCancelled();
        throw SdkError.signing(message);
      });
    signed = tx.acceptSignedBytes(bytes);
  }
  return rpc.submitSignedTransaction(signed);
}
