import { restClient, marketAndOrderbook } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  // 1. First page of trades
  const page1 = await client.trades().get(orderbookId, 10);
  console.log("trades (page 1):", page1.trades.length);
  for (const trade of page1.trades) {
    console.log(`  ${trade.side} ${trade.size} @ ${trade.price}`);
  }

  // 2. Next page using cursor
  if (page1.nextCursor !== undefined) {
    const page2 = await client.trades().get(orderbookId, 10, page1.nextCursor);
    console.log("trades (page 2):", page2.trades.length);
  } else {
    console.log("no more pages");
  }
}

main().catch(console.error);
