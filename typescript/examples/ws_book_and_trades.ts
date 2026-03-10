import { OrderbookSnapshot } from "../src/domain/orderbook/state";
import { TradeHistory } from "../src/domain/trade/state";
import { tradeFromWs } from "../src/domain/trade/convert";
import type { WsEvent } from "../src/ws";
import { restClient, marketAndOrderbook } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  // State trackers
  const book = new OrderbookSnapshot(orderbookId);
  const trades = new TradeHistory(orderbookId, 20);

  // Connect WebSocket
  const ws = client.ws();
  await ws.connect();
  console.log("connected");

  // Subscribe to book updates and trades
  ws.subscribe({ type: "book_update", orderbook_ids: [orderbookId] });
  ws.subscribe({ type: "trades", orderbook_ids: [orderbookId] });

  let eventCount = 0;
  const maxEvents = 20;

  ws.on((event: WsEvent) => {
    if (event.type === "Message") {
      const msg = event.message;

      if (msg.type === "book_update") {
        book.apply(msg.data);
        console.log(
          `[book] seq=${book.seq} best_bid=${book.bestBid() ?? "-"} best_ask=${book.bestAsk() ?? "-"}`
        );
      } else if (msg.type === "trades") {
        const wsTrade = msg.data;
        trades.push(tradeFromWs(wsTrade));
        console.log(
          `[trade] ${wsTrade.side} ${wsTrade.size} @ ${wsTrade.price}`
        );
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
