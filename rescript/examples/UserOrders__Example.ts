// TypeScript example — the signed-in user's open orders snapshot via the gentype
// facade. Authenticate with a wallet keypair first; orders arrive as a tagged
// union ({ TAG: "Limit" | "Trigger", _0: payload }) with the shared fields under
// `.common`. Run with: npx tsx examples/UserOrders.ts
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

  const snapshot = await OrderClient.forUser(client, 10, undefined);

  let limitOrders = 0;
  let triggerOrders = 0;
  for (const order of snapshot.orders) {
    if (order.TAG === "Trigger") {
      triggerOrders += 1;
    } else {
      limitOrders += 1;
    }
  }
  console.log(`orders: ${limitOrders} limit / ${triggerOrders} trigger`);
  console.log(`market balances: ${snapshot.marketBalances.length}`);
  console.log(`has more: ${snapshot.hasMore}`);

  const first = snapshot.orders[0];
  if (first) {
    if (first.TAG === "Trigger") {
      const trigger = first._0;
      console.log(
        `first trigger: ${trigger.triggerOrderId} ${trigger.common.side} @ ${trigger.common.price} ` +
          `(trigger ${trigger.triggerPrice})`,
      );
    } else {
      const limit = first._0;
      console.log(`first limit: ${limit.common.orderHash} ${limit.common.side} @ ${limit.common.price}`);
    }
  }

  // Follow the cursor once, if the backend paginated the snapshot.
  if (snapshot.nextCursor !== undefined) {
    const next = await OrderClient.forUser(client, 10, snapshot.nextCursor);
    console.log(`next page: ${next.orders.length} order(s)`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
