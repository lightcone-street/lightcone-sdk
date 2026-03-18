import type { Connection, PublicKey } from "@solana/web3.js";
import type { LightconeHttp } from "./http";

export interface ClientContext {
  readonly http: LightconeHttp;
  readonly programId: PublicKey;
  readonly connection?: Connection;
}

export function requireConnection(ctx: ClientContext): Connection {
  if (!ctx.connection) {
    throw new Error(
      "RPC client not configured — use .rpcUrl() on the builder"
    );
  }
  return ctx.connection;
}
