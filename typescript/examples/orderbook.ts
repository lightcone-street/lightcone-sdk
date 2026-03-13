import { restClient, marketAndOrderbook } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  // 1. Fetch orderbook depth
  const depth = await client.orderbooks().get(orderbookId, 10);
  console.log("orderbook:", depth.orderbook_id);
  console.log("best bid:", depth.best_bid ?? "none");
  console.log("best ask:", depth.best_ask ?? "none");
  console.log("bids:", depth.bids.length);
  console.log("asks:", depth.asks.length);

  // 2. Fetch decimal precision metadata
  const decimals = await client.orderbooks().decimals(orderbookId);
  console.log("price decimals:", decimals.price_decimals);
  console.log("base decimals:", decimals.base_decimals);
  console.log("quote decimals:", decimals.quote_decimals);
}

main().catch(console.error);
