import { PriceHistoryState, Resolution, type LineData, type WsEvent } from "../src";
import { marketAndOrderbook, restClient, withTimeout } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;
  const resolution = Resolution.Minute1;

  const history = new PriceHistoryState();
  const ws = client.ws();
  let hits = 0;

  let resolveDone!: () => void;
  const done = new Promise<void>((resolve) => {
    resolveDone = resolve;
  });

  const unsubscribe = ws.on((event: WsEvent) => {
    if (event.type === "Message" && event.message.type === "ticker") {
      const ticker = event.message.data;
      console.log(`ticker: bid=${ticker.best_bid} ask=${ticker.best_ask} mid=${ticker.mid}`);
      hits += 1;
    } else if (
      event.type === "Message" &&
      event.message.type === "price_history" &&
      event.message.data.event_type === "snapshot"
    ) {
      const data = event.message.data;
      const prices: LineData[] = data.prices.map((c) => ({
        time: c.t,
        value: c.m ?? "0",
      }));
      history.applySnapshot(data.orderbook_id, data.resolution, prices);
      const candles = history.get(orderbookId, resolution)?.length ?? 0;
      console.log(`price snapshot: ${candles} candle(s)`);
      hits += 1;
    } else if (
      event.type === "Message" &&
      event.message.type === "price_history" &&
      event.message.data.event_type === "update"
    ) {
      const data = event.message.data;
      const point: LineData = {
        time: data.t,
        value: data.m ?? "0",
      };
      history.applyUpdate(data.orderbook_id, data.resolution, point);
      console.log("latest candle:", history.get(orderbookId, resolution)?.at(-1));
      hits += 1;
    } else if (
      event.type === "Message" &&
      event.message.type === "price_history" &&
      event.message.data.event_type === "heartbeat"
    ) {
      console.log(`heartbeat: ${event.message.data.server_time}`);
    } else if (event.type === "Error") {
      console.error("ws error:", event.error);
    }

    if (hits >= 4) {
      resolveDone();
    }
  });

  try {
    await ws.connect();
    ws.subscribe({ type: "ticker", orderbook_ids: [orderbookId] });
    ws.subscribe({
      type: "price_history",
      orderbook_id: orderbookId,
      resolution,
      include_ohlcv: false,
    });
    await withTimeout(done, 15_000, "timed out waiting for websocket data");
  } finally {
    unsubscribe();
    await ws.disconnect();
  }
}

main().catch(console.error);
