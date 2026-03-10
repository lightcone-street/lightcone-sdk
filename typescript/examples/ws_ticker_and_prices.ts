import { Resolution, asOrderBookId } from "../src/shared/types";
import { PriceHistoryState } from "../src/domain/price_history/state";
import type { LineData } from "../src/domain/price_history";
import type { WsEvent } from "../src/ws";
import { restClient, marketAndOrderbook } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;
  const resolution = Resolution.Minute1;

  // State tracker
  const history = new PriceHistoryState();

  // Connect WebSocket
  const ws = client.ws();
  await ws.connect();
  console.log("connected");

  // Subscribe to ticker and price history
  ws.subscribe({ type: "ticker", orderbook_ids: [orderbookId] });
  ws.subscribe({
    type: "price_history",
    orderbook_id: orderbookId,
    resolution,
    include_ohlcv: false,
  });

  let eventCount = 0;
  const maxEvents = 20;

  ws.on((event: WsEvent) => {
    if (event.type === "Message") {
      const msg = event.message;

      if (msg.type === "ticker") {
        const ticker = msg.data;
        console.log(
          `[ticker] bid=${ticker.best_bid ?? "-"} ask=${ticker.best_ask ?? "-"} mid=${ticker.mid ?? "-"}`
        );
      } else if (msg.type === "price_history") {
        const data = msg.data;

        if (data.event_type === "snapshot") {
          const prices: LineData[] = data.prices.map((c) => ({
            time: c.t,
            value: c.m ? parseFloat(c.m) : 0,
          }));
          history.applySnapshot(
            asOrderBookId(data.orderbook_id),
            data.resolution as Resolution,
            prices
          );
          console.log(`[price_history] snapshot: ${data.prices.length} candles`);
        } else if (data.event_type === "update") {
          const point: LineData = {
            time: data.t,
            value: data.m ? parseFloat(data.m) : 0,
          };
          history.applyUpdate(
            asOrderBookId(data.orderbook_id),
            data.resolution as Resolution,
            point
          );
          console.log(`[price_history] update: t=${data.t} mid=${data.m ?? "-"}`);
        } else if (data.event_type === "heartbeat") {
          console.log(`[price_history] heartbeat: server_time=${data.server_time}`);
        }

        const candles = history.get(orderbookId, resolution);
        if (candles) {
          console.log(`  total candles: ${candles.length}`);
        }
      }
    } else if (event.type === "Error") {
      console.error("ws error:", event.error);
    }

    eventCount++;
    if (eventCount >= maxEvents) {
      console.log("received", maxEvents, "events, disconnecting");
      ws.disconnect();
    }
  });

  // Keep alive for 30 seconds
  await new Promise((resolve) => setTimeout(resolve, 30_000));
  await ws.disconnect();
}

main().catch(console.error);
