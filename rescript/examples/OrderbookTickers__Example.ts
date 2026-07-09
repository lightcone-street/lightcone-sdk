// TypeScript example — orderbook tickers (batch BBO + midpoint) via the gentype
// facade. Pass an optional deposit-asset mint as the first CLI arg to filter.
import { makeForEnv, MetricsClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  const depositAsset = process.argv[2];
  const response = await MetricsClient.orderbookTickers(client, depositAsset);

  console.log(`orderbooks with tickers: ${response.tickers.length}`);
  for (const entry of response.tickers.slice(0, 10)) {
    const mid = entry.midpoint ?? "—";
    const outcome = entry.outcomeIndex ?? "—";
    console.log(`  ${entry.orderbookId} (market ${entry.marketPubkey}, outcome ${outcome}) mid=${mid}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
