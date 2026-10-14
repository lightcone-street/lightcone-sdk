// TypeScript example — deposit assets registered for a market via the gentype facade.
import { makeForEnv, MarketClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  const page = await MarketClient.get(client, undefined, 1);
  const market = page.markets[0];
  if (!market) {
    console.log("no markets found");
    return;
  }

  const response = await MarketClient.depositMints(client, market.pubkey);
  console.log(`market ${market.slug} (${response.marketPubkey}): ${response.total} deposit assets`);
  for (const asset of response.depositAssets) {
    const symbol = asset.symbol ?? "?";
    console.log(`  - ${symbol} (${asset.depositAsset}) — ${asset.conditionalMints.length} conditional mints`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
