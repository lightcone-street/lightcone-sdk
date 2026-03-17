import { Resolution } from "../src";
import { restClient, marketAndOrderbook, unixTimestampMs } from "./common";

async function main() {
  const client = restClient();
  const [market, orderbook] = await marketAndOrderbook(client);
  const orderbookId = orderbook.orderbookId;
  const depositAsset = market.depositAssets[0];
  if (!depositAsset) {
    throw new Error("Market has no deposit assets");
  }

  const now = unixTimestampMs();
  const sevenDaysAgo = now - 7 * 24 * 60 * 60 * 1000;

  const orderbookHistory = await client
    .priceHistory()
    .get(orderbookId, {
      resolution: Resolution.Hour1,
      from: sevenDaysAgo,
      to: now,
      includeOhlcv: true,
      limit: 10,
    });

  const depositHistory = await client
    .priceHistory()
    .getDepositAsset(depositAsset.pubkey, {
      resolution: Resolution.Hour1,
      from: sevenDaysAgo,
      to: now,
      limit: 10,
    });

  console.log("orderbook:");
  console.log(JSON.stringify(orderbookHistory, null, 2));
  console.log("deposit asset:");
  console.log(JSON.stringify(depositHistory, null, 2));
}

main().catch(console.error);
