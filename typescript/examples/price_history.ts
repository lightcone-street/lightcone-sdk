import { Resolution } from "../src/shared/types";
import { restClient, marketAndOrderbook } from "./common";

async function main() {
  const client = restClient();
  const [, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;

  // Fetch 7-day history at 1-hour resolution
  const now = Math.floor(Date.now() / 1000);
  const sevenDaysAgo = now - 7 * 24 * 60 * 60;

  const history = await client
    .priceHistory()
    .get(orderbookId, Resolution.Hour1, sevenDaysAgo, now);

  console.log(JSON.stringify(history, null, 2));
}

main().catch(console.error);
