// TypeScript example — cancel the first open limit order for the authenticated
// wallet, entirely through the gentype facade, then withdraw the released
// collateral back out of the global pool. Mirrors CancelOrder__Example.res.
// Unlike the ReScript core, the facade signs the cancel with the client's native
// signer internally, so no @solana/kit keypair/address helpers are needed —
// `cancel` takes just the order hash.
//
// Orders arrive as a tagged union ({ TAG: "Limit" | "Trigger", _0: payload });
// we cancel the first Limit one. Run with: npx tsx examples/CancelOrder__Example.ts
import * as fs from "node:fs";
import * as os from "node:os";
import {
  makeForEnv,
  useNativeSigner,
  AuthClient,
  MarketClient,
  OrderClient,
  PositionClient,
} from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

// Mirrors the constant in SubmitOrder__Example.ts: when we cancel the order that
// example left open, we withdraw the same quote amount back from the global pool
// so the deposit/submit/cancel/withdraw cycle is net-neutral across runs.
const ORDER_QUOTE_AMOUNT = 1_100_000n; // 0.55 * 2 USDC

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
  const order = snapshot.orders.find((o) => o.TAG === "Limit");
  if (!order) {
    console.log("No open limit orders to cancel.");
    return;
  }

  const cancelled = await OrderClient.cancel(client, order._0.common.orderHash);
  console.log(`cancelled: ${cancelled.orderHash} remaining=${cancelled.remaining}`);

  // Withdraw the released collateral (the first market pair's quote deposit asset).
  const page = await MarketClient.get(client, undefined, 1);
  const pair = page.markets[0]?.orderbookPairs.find((p) => p.active) ?? page.markets[0]?.orderbookPairs[0];
  if (!pair) {
    console.log("withdraw_from_global: no orderbook pair found");
    return;
  }
  const withdrawSignature = await PositionClient.withdrawFromGlobal(
    client,
    pair.quote.depositAsset,
    ORDER_QUOTE_AMOUNT,
  );
  console.log(`withdraw_from_global: confirmed ${withdrawSignature}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
