import { restClient, marketAndOrderbook, runExample } from "./common";
import { orderbookDecimals } from "../src/domain/orderbook";

async function main() {
  const client = restClient();
  const [market, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  const depth = await client.orderbooks().get(orderbookId, 10);
  const decimals = orderbookDecimals(orderbook);
  console.log("market:", market.slug);
  console.log("orderbook:", orderbookId);
  console.log(`best bid: ${depth.best_bid}, best ask: ${depth.best_ask}`);
  console.log(`levels: ${depth.bids.length} bids / ${depth.asks.length} asks`);
  console.log(
    `decimals: price=${decimals.priceDecimals}, base=${decimals.baseDecimals}, quote=${decimals.quoteDecimals}`
  );
}

void runExample(main);
