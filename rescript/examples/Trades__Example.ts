// TypeScript example — recent trades via the gentype facade.
import { makeForEnv, MarketClient, TradeClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  const page = await MarketClient.get(client, undefined, 1);
  const market = page.markets[0];
  const pair = market?.orderbookPairs[0];
  if (!pair) {
    console.log("no orderbook found");
    return;
  }

  const tradesPage = await TradeClient.forOrderbook(client, pair.orderbookId, 5, undefined);
  console.log(`${tradesPage.trades.length} recent trades for ${pair.orderbookId}:`);
  for (const trade of tradesPage.trades) {
    console.log(`  ${trade.side} ${trade.size} @ ${trade.price}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
