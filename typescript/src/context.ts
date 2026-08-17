import type { Connection, PublicKey } from "@solana/web3.js";
import { SdkError } from "./error";
import type { LightconeHttp } from "./http";
import type { AuthCredentials } from "./auth";
import type { DepositSource, OrderbookRules } from "./shared";
import type { SigningStrategy } from "./shared/signing";
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

/**
 * Sign and submit a transaction using the client's signing strategy.
 *
 * Fetches a recent blockhash automatically. Returns as soon as the RPC
 * accepts the transaction — inclusion is not awaited. When follow-up work
 * depends on this transaction's on-chain effects, use
 * {@link signAndSubmitTxConfirmed} instead.
 */
export async function signAndSubmitTx(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction
): Promise<string> {
  const { signature } = await signAndSubmitTxInner(ctx, tx);
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
  const { Rpc } = await import("./rpc");

  const { signature, lastValidBlockHeight } = await signAndSubmitTxInner(
    ctx,
    tx
  );
  const status = await new Rpc(ctx).confirmSignatureStatus(
    signature,
    lastValidBlockHeight
  );
  return { signature, slot: status.slot };
}

/**
 * Shared submit path: sign, send, and return the signature together with the
 * `lastValidBlockHeight` of the blockhash the submitted wire bytes are known
 * to carry — `null` when that cannot be proven (external signer replaced the
 * blockhash, or the bytes were never visible to the SDK).
 */
async function signAndSubmitTxInner(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction
): Promise<{ signature: string; lastValidBlockHeight: number | null }> {
  const { isUserCancellation } = await import("./shared/signing");
  const { SdkError } = await import("./error");
  const { RetryPolicy } = await import("./http");

  const strategy = requireSigningStrategy(ctx);

  // Get blockhash with failover, at `confirmed` commitment (pinned, not the
  // Connection's default — matching the Rust and Python SDKs).
  const { blockhash, lastValidBlockHeight } = await connectionWithFailover(
    ctx,
    (conn) => conn.getLatestBlockhash("confirmed")
  );
  tx.recentBlockhash = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;

  switch (strategy.type) {
    case "native": {
      tx.partialSign(strategy.keypair);
      const signature = await connectionWithFailover(ctx, (conn) =>
        conn.sendRawTransaction(tx.serialize())
      );
      return { signature, lastValidBlockHeight };
    }
    case "walletAdapter": {
      const txBytes = tx.serialize({ requireAllSignatures: false });
      const signedBytes = await strategy.signer
        .signTransaction(txBytes)
        .catch((err: unknown) => {
          const msg = err instanceof Error ? err.message : String(err);
          if (isUserCancellation(msg)) throw SdkError.userCancelled();
          throw SdkError.signing(msg);
        });
      const signature = await connectionWithFailover(ctx, (conn) =>
        conn.sendRawTransaction(signedBytes)
      );
      return {
        signature,
        lastValidBlockHeight: (await signedBlockhashUnchanged(
          signedBytes,
          blockhash
        ))
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
 * True when the signed wire bytes still carry `expectedBlockhash`. External
 * signers may re-blockhash a transaction before signing; a bound derived
 * from the original blockhash must then not be used for expiry detection.
 */
async function signedBlockhashUnchanged(
  signedBytes: Uint8Array,
  expectedBlockhash: string
): Promise<boolean> {
  try {
    const { Transaction } = await import("@solana/web3.js");
    return Transaction.from(signedBytes).recentBlockhash === expectedBlockhash;
  } catch {
    return false;
  }
}
