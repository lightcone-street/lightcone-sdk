// TypeScript example — cancel the first open limit order for the authenticated
// wallet, entirely through the gentype facade. Mirrors CancelOrder.example.res.
// Unlike the ReScript core, the facade signs the cancel with the client's native
// signer internally, so no @solana/kit keypair/address helpers are needed —
// `cancelOrder` takes just the order hash.
//
// Orders carry an `orderType` discriminator ("limit" | "trigger"); we cancel the
// first "limit" one. Run with: npx tsx examples/CancelOrder.example.ts
import * as fs from "node:fs";
import * as os from "node:os";
import { makeForEnv, useNativeSigner, AuthClient, OrderClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

function walletSecretKey(): Uint8Array {
  const path = process.env.LIGHTCONE_WALLET_PATH ?? "~/.config/solana/id.json";
  const resolved = path.startsWith("~") ? path.replace("~", os.homedir()) : path;
  return Uint8Array.from(JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[]);
}

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  await useNativeSigner(client, walletSecretKey());
  await AuthClient.login(client, undefined);

  const snapshot = await OrderClient.forUser(client, 50, undefined);
  const order = snapshot.orders.find((o) => o.orderType === "limit");
  if (!order) {
    console.log("No open limit orders to cancel.");
    return;
  }

  const cancelled = await OrderClient.cancel(client, order.orderHash);
  console.log(`cancelled: ${cancelled.orderHash} remaining=${cancelled.remaining}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
