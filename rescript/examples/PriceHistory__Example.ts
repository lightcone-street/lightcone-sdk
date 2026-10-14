// TypeScript example — orderbook price history (midpoint candles) via the gentype facade.
import { makeForEnv, MarketClient, PriceHistoryClient } from "../src/TypeScriptApi.gen.ts";
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
  const pair = market.orderbookPairs[0];
  if (!pair) {
    console.log("market has no orderbooks");
    return;
  }

  const toMs = Date.now();
  const fromMs = toMs - 7 * 24 * 60 * 60 * 1000;
  const history = await PriceHistoryClient.get(client, pair.orderbookId, "1h", fromMs, toMs);

  console.log(`market: ${market.slug}`);
  console.log(`orderbook: ${history.orderbookId}`);
  console.log(`${history.resolution} candles: ${history.prices.length} (has_more=${history.hasMore})`);
  console.log(`decimals: price=${history.decimals.price}, volume=${history.decimals.volume}`);
  for (const candle of history.prices.slice(0, 5)) {
    console.log(`  t=${candle.t} mid=${candle.m ?? "—"}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
