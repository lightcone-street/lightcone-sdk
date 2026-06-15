import {
  aggregationFromFrame,
  aggregationKeySuffix,
  aggregationsEqual,
  OrderbookState,
  subscribeBooks,
  TradeHistory,
  unsubscribeBooks,
  validateAggregation,
  type BookAggregation,
  type Trade,
  type WsEvent,
} from "../src";
import { marketAndOrderbook, restClient, runExample, withTimeout } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  // One connection can hold multiple aggregation views of the same book.
  // Key the local state by the frame's aggregation: full precision for
  // pricing, a grouped view (5 sig figs, mantissa 2) for display.
  const groupedAggregation: BookAggregation = validateAggregation({ nSigFigs: 5, mantissa: 2 });
  const fullBook = new OrderbookState(orderbookId);
  const groupedBook = new OrderbookState(orderbookId);
  const trades = new TradeHistory(orderbookId, 20);
  const ws = client.ws();
  let hits = 0;

  let resolveDone!: () => void;
  const done = new Promise<void>((resolve) => {
    resolveDone = resolve;
  });

  const unsubscribe = ws.on((event: WsEvent) => {
    if (event.type === "Message" && event.message.type === "book_update") {
      const update = event.message.data;
      // Untagged frames are the full-precision view; frames from the grouped
      // subscription carry n_sig_figs/mantissa.
      const aggregation = aggregationFromFrame(update.n_sig_figs, update.mantissa);
      if (update.resync) {
        // Refresh exactly the affected view: re-subscribe with the SAME
        // aggregation. The fresh snapshot arrives with seq 0 and replaces
        // the book (last-write-wins).
        ws.send(unsubscribeBooks([update.orderbook_id], aggregation));
        ws.send(subscribeBooks([update.orderbook_id], aggregation));
        return;
      }
      const book = aggregationsEqual(aggregation, groupedAggregation) ? groupedBook : fullBook;
      book.apply(update);
      console.log(
        `book[${aggregationKeySuffix(aggregation)}]: seq=${book.seq} bid=${book.bestBid()} ask=${book.bestAsk()}`
      );
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
    ws.subscribe({
      type: "book_update",
      orderbook_ids: [orderbookId],
      nSigFigs: groupedAggregation.nSigFigs,
      mantissa: groupedAggregation.mantissa,
    });
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
