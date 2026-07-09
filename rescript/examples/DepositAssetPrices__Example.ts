// TypeScript example — snapshot of current deposit-asset prices via the gentype
// facade. Public endpoint; the facade returns a throwing Promise of a
// `{ [mint]: price }` map. Run with: npx tsx examples/DepositAssetPrices.ts
import { makeForEnv, PriceHistoryClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  const snapshot = await PriceHistoryClient.depositAssetSnapshot(client);
  const entries = Object.entries(snapshot.prices);
  console.log(`deposit-asset-prices-snapshot: ${entries.length} entries`);
  for (const [mint, price] of entries.slice(0, 10)) {
    console.log(`  ${mint} -> ${price}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
