// TypeScript example — WebSocket: the ticker (best bid / ask / mid) and price-history
// candles (1-minute resolution) for the first orderbook of the first market. Driven
// entirely through the gentype facade — this proves the WS layer is reachable from
// TypeScript without importing any `.res.mjs` or `@solana/kit`.
// Note: connects to a live WS server; push messages arrive via the onMessage callback.
import { makeForEnv, MarketClient, WsClient } from "../src/TypeScriptApi.gen.ts";
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
  const pair = market.orderbookPairs.find((p) => p.active) ?? market.orderbookPairs[0];
  if (!pair) {
    console.log("market has no orderbooks");
    return;
  }
  const orderbookId = pair.orderbookId;

  let eventCount = 0;
  const connection = WsClient.connect(
    client,
    (msg: wsMessage) => {
      const kind = msg.kind;
      if (typeof kind === "string") {
        // "Pong" heartbeat — no payload.
        return;
      }
      switch (kind.TAG) {
        case "Ticker":
          eventCount += 1;
          // wsTicker carries orderbookId plus optional bestBid / bestAsk / mid.
          console.log(`ticker for ${kind._0.orderbookId}`);
          break;
        case "PriceHistory":
          eventCount += 1;
          console.log("PriceHistory received");
          break;
        default:
          break;
      }
    },
    undefined,
    undefined,
  );

  // The ticker (best bid / ask / mid) plus 1-minute price-history candles on one socket.
  WsClient.subscribe(connection, { TAG: "Ticker", _0: [orderbookId] } as wsSubscription);
  WsClient.subscribe(
    connection,
    { TAG: "PriceHistory", _0: { orderbookId, resolution: "1m", includeOhlcv: false } } as wsSubscription,
  );

  console.log(`subscribed to ticker + price history for ${orderbookId}`);
  await new Promise((r) => setTimeout(r, 15000));
  WsClient.disconnect(connection);
  if (eventCount === 0) {
    console.log("received no websocket events — connection may be unreachable");
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
