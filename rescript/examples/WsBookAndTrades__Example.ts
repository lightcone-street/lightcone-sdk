// TypeScript example — WebSocket: the live order book (a full-precision view plus a
// grouped 5-sig-fig view on one connection) and the trade tape for the first orderbook
// of the first market. Driven entirely through the gentype facade — this proves the WS
// layer is reachable from TypeScript without importing any `.res.mjs` or `@solana/kit`.
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
        case "BookUpdate":
          eventCount += 1;
          console.log("BookUpdate received");
          break;
        case "Trades":
          eventCount += 1;
          // wsTrade carries price / size / side; log the price to avoid guessing.
          console.log(`trade @ ${kind._0.price}`);
          break;
        default:
          break;
      }
    },
    undefined,
    undefined,
  );

  // One connection, two book views: full precision (pricing) + a grouped view (5 sig
  // figs, mantissa 2 — display), plus the trade tape.
  WsClient.subscribe(connection, { TAG: "Books", _0: { orderbookIds: [orderbookId] } } as wsSubscription);
  WsClient.subscribe(
    connection,
    { TAG: "Books", _0: { orderbookIds: [orderbookId], nSigFigs: 5, mantissa: 2 } } as wsSubscription,
  );
  WsClient.subscribe(connection, { TAG: "Trades", _0: [orderbookId] } as wsSubscription);

  console.log(`subscribed to book + trades for ${orderbookId}`);
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
