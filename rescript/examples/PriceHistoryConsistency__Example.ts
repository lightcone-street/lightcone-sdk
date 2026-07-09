// TypeScript example — compare price-history points from REST (priceLineData)
// with the WS Prices snapshot for the same orderbook + resolution. Entirely
// via the gentype facade. Ported from rust/examples/price_history_consistency.rs.
import { makeForEnv, MarketClient, PriceHistoryClient, WsClient } from "../src/TypeScriptApi.gen.ts";
import type { WsClient_wsMessage as wsMessage, WsClient_wsSubscription as wsSubscription } from "../src/TypeScriptApi.gen.ts";
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
  const pair = market.orderbookPairs.find((candidate) => candidate.active) ?? market.orderbookPairs[0];
  if (!pair) {
    console.log("selected market has no orderbooks");
    return;
  }

  // 1. REST API — the same line-data the app's server function fetches.
  const restPoints = await PriceHistoryClient.lineData(client, pair.orderbookId, "5m", undefined, undefined, undefined, 1000);
  console.log("=== REST API ===");
  console.log(`  points: ${restPoints.length}`);

  // 2. WebSocket snapshot for the same orderbook + resolution.
  let wsFrames = 0;
  const connection = WsClient.connect(
    client,
    (message: wsMessage) => {
      const kind = message.kind;
      if (typeof kind !== "string" && kind.TAG === "PriceHistory") {
        wsFrames += 1;
      }
    },
    undefined,
    undefined,
  );
  WsClient.subscribe(connection, {
    TAG: "PriceHistory",
    _0: { orderbookId: pair.orderbookId, resolution: "5m", includeOhlcv: false },
  } as wsSubscription);
  await new Promise((resolve) => setTimeout(resolve, 5000));
  console.log("=== WebSocket ===");
  console.log(`  price_history frames: ${wsFrames}`);
  WsClient.disconnect(connection);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
