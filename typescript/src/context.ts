import type { Connection, PublicKey } from "@solana/web3.js";
import { SdkError } from "./error";
import type { LightconeHttp } from "./http";
import type { DepositSource } from "./shared";
import type { SigningStrategy } from "./shared/signing";

export interface ClientContext {
  readonly http: LightconeHttp;
  readonly programId: PublicKey;
  readonly connection?: Connection;
  readonly depositSource: DepositSource;
  readonly signingStrategy?: SigningStrategy;
}

export function requireConnection(ctx: ClientContext): Connection {
  if (!ctx.connection) {
    throw SdkError.validation("RPC client not configured — use .rpcUrl() on the builder");
  }
  return ctx.connection;
}

export function resolveDepositSource(
  ctx: ClientContext,
  overrideSource: DepositSource | undefined
): DepositSource {
  return overrideSource ?? ctx.depositSource;
}

export function requireSigningStrategy(ctx: ClientContext): SigningStrategy {
  if (!ctx.signingStrategy) {
    throw SdkError.validation("Signing strategy not configured — use .nativeSigner(), .externalSigner(), or .privyWalletId() on the builder");
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
  const connection = requireConnection(ctx);
  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;

  switch (strategy.type) {
    case "native": {
      tx.partialSign(strategy.keypair);
      return connection.sendRawTransaction(tx.serialize());
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
      return connection.sendRawTransaction(signedBytes);
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
