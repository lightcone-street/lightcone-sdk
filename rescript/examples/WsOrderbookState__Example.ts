// TypeScript example — WebSocket: maintain a LIVE order book (LiveOrderbook) and a rolling
// price-history series (LivePriceHistory) for the first orderbook of the first market,
// printing best bid / ask / mid / spread as frames arrive. Entirely through the gentype
// facade — the stateful WS containers (ported from rust `*/state.rs`) reachable from TS.
import {
  makeForEnv,
  MarketClient,
  WsClient,
  LiveOrderbook,
  LivePriceHistory,
} from "../src/TypeScriptApi.gen.ts";
import type {
  WsClient_wsMessage as wsMessage,
  WsClient_wsSubscription as wsSubscription,
} from "../src/TypeScriptApi.gen.ts";
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

  const book = LiveOrderbook.make(orderbookId);
  const history = LivePriceHistory.make();
  let eventCount = 0;
  const show = (value: string | undefined): string => value ?? "-";

  let connection: ReturnType<typeof WsClient.connect> | undefined = undefined;
  connection = WsClient.connect(
    client,
    (msg: wsMessage) => {
      const kind = msg.kind;
      if (typeof kind === "string") {
        return; // "Pong" heartbeat — no payload.
      }
      switch (kind.TAG) {
        case "BookUpdate": {
          const result = LiveOrderbook.apply(book, kind._0);
          if (result === "Applied") {
            eventCount += 1;
            console.log(
              `book: bid=${show(LiveOrderbook.bestBid(book))} ask=${show(
                LiveOrderbook.bestAsk(book),
              )} mid=${show(LiveOrderbook.midPrice(book))} spread=${show(LiveOrderbook.spread(book))}`,
            );
          } else if (connection) {
            // resync — re-subscribe to pull a fresh seq-0 snapshot.
            WsClient.unsubscribe(connection, { TAG: "Books", _0: { orderbookIds: [orderbookId] } } as wsSubscription);
            WsClient.subscribe(connection, { TAG: "Books", _0: { orderbookIds: [orderbookId] } } as wsSubscription);
          }
          break;
        }
        case "PriceHistory": {
          const event = kind._0;
          if (event.TAG === "Snapshot") {
            eventCount += 1;
            LivePriceHistory.applySnapshot(history, event._0.orderbookId, event._0.resolution, event._0.prices);
            console.log(`price-history snapshot: ${event._0.prices.length} candles`);
          } else if (event.TAG === "Update") {
            LivePriceHistory.applyUpdate(history, event._0.orderbookId, event._0.resolution, event._0.candle);
          }
          break;
        }
        default:
          break;
      }
    },
    undefined,
    undefined,
  );

  WsClient.subscribe(connection, { TAG: "Books", _0: { orderbookIds: [orderbookId] } } as wsSubscription);
  WsClient.subscribe(
    connection,
    { TAG: "PriceHistory", _0: { orderbookId, resolution: "1h", includeOhlcv: true } } as wsSubscription,
  );

  console.log(`subscribed to book + price history for ${orderbookId}`);
  await new Promise((r) => setTimeout(r, 15000));
  WsClient.disconnect(connection);

  const series = LivePriceHistory.get(history, orderbookId, "1h");
  if (series) {
    console.log(`final price-history series: ${series.length} points`);
  }
  if (eventCount === 0) {
    console.log("received no websocket events — connection may be unreachable");
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
