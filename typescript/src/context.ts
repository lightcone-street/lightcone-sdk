import type { Connection, PublicKey } from "@solana/web3.js";
import { SdkError } from "./error";
import type { LightconeHttp } from "./http";
import type { DepositSource } from "./shared";
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
  orderNonce?(): number | undefined;
  setOrderNonce?(nonce: number): void;

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

export async function signAndSubmitTx(
  ctx: ClientContext,
  tx: import("@solana/web3.js").Transaction
): Promise<string> {
  const { isUserCancellation } = await import("./shared/signing");
  const { SdkError } = await import("./error");
  const { RetryPolicy } = await import("./http");

  const strategy = requireSigningStrategy(ctx);

  // Get blockhash with failover.
  const { blockhash } = await connectionWithFailover(ctx, (conn) =>
    conn.getLatestBlockhash()
  );
  tx.recentBlockhash = blockhash;

  switch (strategy.type) {
    case "native": {
      tx.partialSign(strategy.keypair);
      return connectionWithFailover(ctx, (conn) =>
        conn.sendRawTransaction(tx.serialize())
      );
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
      return connectionWithFailover(ctx, (conn) =>
        conn.sendRawTransaction(signedBytes)
      );
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
      return result.hash;
    }
  }
}
