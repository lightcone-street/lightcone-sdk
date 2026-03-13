import { restClient, marketAndOrderbook } from "./common";

async function main() {
  const client = restClient();
  const [market, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  const depth = await client.orderbooks().get(orderbookId, 10);
  const decimals = await client.orderbooks().decimals(orderbookId);
  console.log("market:", market.slug);
  console.log("orderbook:", orderbookId);
  console.log(`best bid: ${depth.best_bid}, best ask: ${depth.best_ask}`);
  console.log(`levels: ${depth.bids.length} bids / ${depth.asks.length} asks`);
  console.log(
    `decimals: price=${decimals.price_decimals}, base=${decimals.base_decimals}, quote=${decimals.quote_decimals}`
  );
}

main().catch(console.error);
