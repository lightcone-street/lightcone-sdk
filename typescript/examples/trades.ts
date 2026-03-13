import { restClient, marketAndOrderbook } from "./common";

function printTrades(pageLabel: string, trades: Array<{
  tradeId: string;
  timestamp: Date;
  size: string;
  side: string;
  price: string;
}>): void {
  console.log(`${pageLabel}: ${trades.length} trade(s)`);
  for (const trade of trades) {
    console.log(
      `  ${trade.tradeId} ${trade.timestamp.toISOString()} ${trade.size} ${trade.side} @ ${trade.price}`
    );
  }
}

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  const page1 = await client.trades().get(orderbookId, 10);
  printTrades("page 1", page1.trades);
  const latest = page1.trades[0];
  if (latest) {
    console.log(`latest: ${latest.size} ${latest.side} @ ${latest.price}`);
  }

  if (page1.nextCursor !== undefined) {
    const page2 = await client.trades().get(orderbookId, 10, page1.nextCursor);
    printTrades("page 2", page2.trades);
  }
}

main().catch(console.error);
