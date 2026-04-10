import { OrderbookSnapshot, TradeHistory, type Trade, type WsEvent } from "../src";
import { marketAndOrderbook, restClient, runExample, withTimeout } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  const book = new OrderbookSnapshot(orderbookId);
  const trades = new TradeHistory(orderbookId, 20);
  const ws = client.ws();
  let hits = 0;

  let resolveDone!: () => void;
  const done = new Promise<void>((resolve) => {
    resolveDone = resolve;
  });

  const unsubscribe = ws.on((event: WsEvent) => {
    if (event.type === "Message" && event.message.type === "book_update") {
      book.apply(event.message.data);
      console.log(`book: seq=${book.seq} bid=${book.bestBid()} ask=${book.bestAsk()}`);
      hits += 1;
    } else if (event.type === "Message" && event.message.type === "trades") {
      const trade: Trade = {
        orderbookId: event.message.data.orderbook_id,
        tradeId: event.message.data.trade_id,
        timestamp: new Date(event.message.data.timestamp),
        price: event.message.data.price,
        size: event.message.data.size,
        side: event.message.data.side,
        sequence: event.message.data.sequence,
      };
      console.log(`trade: ${trade.size} ${trade.side} @ ${trade.price} seq=${trade.sequence}`);
      trades.push(trade);
      hits += 1;
    } else if (event.type === "Error") {
      console.error("ws error:", event.error);
    }

    if (hits >= 4) {
      resolveDone();
    }
  });

  try {
    await ws.connect();
    ws.subscribe({ type: "book_update", orderbook_ids: [orderbookId] });
    ws.subscribe({ type: "trades", orderbook_ids: [orderbookId] });
    await withTimeout(done, 30_000, "timed out waiting for websocket data");
  } catch {
    console.log("no more websocket data (timeout or stream ended)");
  } finally {
    unsubscribe();
    await ws.disconnect();
  }

  if (hits === 0) {
    throw new Error("received no websocket events — connection may be broken");
  }
  console.log(`buffered trades: ${trades.len()}`);
}

void runExample(main);
