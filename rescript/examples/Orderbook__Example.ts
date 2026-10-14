// TypeScript example — live orderbook depth via the gentype facade.
import { makeForEnv, MarketClient, OrderbookClient } from "../src/TypeScriptApi.gen.ts";
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

  // Depth is capped server-side at 20 levels per side.
  const depth = await OrderbookClient.get(client, pair.orderbookId, 10);
  console.log(`market: ${market.slug}`);
  console.log(`orderbook: ${pair.orderbookId}`);
  console.log(`best bid: ${depth.bestBid ?? "—"}, best ask: ${depth.bestAsk ?? "—"}`);
  console.log(`levels: ${depth.bids.length} bids / ${depth.asks.length} asks`);
  console.log(`token decimals: base=${pair.base.decimals}, quote=${pair.quote.decimals}`);
  if (depth.decimals) {
    console.log(`depth decimals: price=${depth.decimals.price}, size=${depth.decimals.size}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
