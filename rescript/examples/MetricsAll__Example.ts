// TypeScript example — exercise several metrics endpoints via the gentype facade.
// Each call returns a throwing Promise (the facade unwraps the SDK result), exactly
// as a TS user migrating from the old TS SDK would consume it.
// Run with: npx tsx examples/MetricsAll.ts
import {
  makeForEnv,
  MetricsClient,
} from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  const platform = await MetricsClient.platform(client);
  console.log(
    `platform: volume_24h_usd=${platform.volume24hUsd}, volume_7d_usd=${platform.volume7dUsd}, ` +
      `open_interest_usd=${platform.openInterestUsd}, active_markets=${platform.activeMarkets}, ` +
      `active_orderbooks=${platform.activeOrderbooks}`,
  );
  console.log(`  deposit token volumes: ${platform.depositTokenVolumes.length}`);

  const markets = await MetricsClient.markets(client);
  console.log(`markets: ${markets.markets.length} entries (total=${markets.total})`);
  for (const entry of markets.markets.slice(0, 3)) {
    console.log(
      `  - ${entry.marketName ?? "?"} — volume_24h_usd=${entry.volume24hUsd} ` +
        `(platform_share_24h=${entry.platformVolumeShare24hPct}%)`,
    );
  }

  const categories = await MetricsClient.categories(client);
  console.log(`categories: ${categories.categories.length}`);
  const firstCategory = categories.categories[0];
  if (firstCategory) {
    console.log(
      `  first '${firstCategory.category}': volume_24h_usd=${firstCategory.volume24hUsd}, ` +
        `unique_traders_24h=${firstCategory.uniqueTraders24h}`,
    );
  }

  const depositTokens = await MetricsClient.depositTokens(client);
  console.log(`deposit tokens: ${depositTokens.depositTokens.length}`);
  for (const token of depositTokens.depositTokens.slice(0, 3)) {
    console.log(`  ${token.symbol ?? "?"} — volume_24h_usd=${token.volume24hUsd}`);
  }

  const tickers = await MetricsClient.orderbookTickers(client, undefined);
  console.log(`orderbook tickers: ${tickers.tickers.length}`);
  const firstTicker = tickers.tickers[0];
  if (firstTicker) {
    console.log(
      `  ${firstTicker.orderbookId}: best_bid=${firstTicker.bestBid ?? "-"} ` +
        `best_ask=${firstTicker.bestAsk ?? "-"} midpoint=${firstTicker.midpoint ?? "-"}`,
    );
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
