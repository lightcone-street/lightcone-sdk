// TypeScript example — per-call cookie forwarding for SSR / server-function use.
//
// NOTE: the per-call cookie-forwarding variants are ReScript-core only. The
// throwing TS facade (`TypeScriptApi.gen.ts`) exposes the read endpoints WITHOUT a
// `cookieHeader` parameter — they rely on the SDK's process-wide auth-token store
// captured at login (shown below). To forward a raw `Cookie` header for a single
// call (e.g. relaying the browser's `lightcone-token` from an incoming SSR request
// instead of the stored token), call the ReScript domain functions directly, which
// take an optional `~cookieHeader`:
//     Position.depositTokenBalances(client, ~cookieHeader="lightcone-token=...")
//     Order.getUserOrders(client, ~limit=50, ~cookieHeader="lightcone-token=...")
// See examples/WithCookies.res for the full cookie-forwarding flow.
//
// Run with: npx tsx examples/WithCookies.ts
import * as fs from "node:fs";
import * as os from "node:os";
import { makeForEnv, useNativeSigner, AuthClient, PositionClient, OrderClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

function walletSecretKey(): Uint8Array {
  const path = process.env.LIGHTCONE_WALLET_PATH ?? "~/.config/solana/id.json";
  const resolved = path.startsWith("~") ? path.replace("~", os.homedir()) : path;
  return Uint8Array.from(JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[]);
}

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  // Login captures the `lightcone-token` cookie internally; the facade read calls
  // below replay it from the SDK's process-wide store.
  await useNativeSigner(client, walletSecretKey());
  await AuthClient.login(client, undefined);

  const balances = await PositionClient.depositTokenBalances(client);
  console.log(`tracked deposit balances: ${Object.keys(balances).length}`);

  const orders = await OrderClient.forUser(client, 50, undefined);
  console.log(`open orders: ${orders.orders.length}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
