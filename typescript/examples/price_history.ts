import { Resolution } from "../src";
import { restClient, marketAndOrderbook, unixTimestampMs } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  const now = unixTimestampMs();
  const sevenDaysAgo = now - 7 * 24 * 60 * 60 * 1000;

  const history = await client
    .priceHistory()
    .get(orderbookId, Resolution.Hour1, sevenDaysAgo, now);

  console.log(JSON.stringify(history, null, 2));
}

main().catch(console.error);
