import { restClient, runExample } from "./common";

async function main() {
  const client = restClient();
  const depositAsset = process.argv[2];

  const response = await client.metrics().orderbookTickers(depositAsset);

  console.log(
    `orderbooks with tickers: ${response.tickers.length}${response.has_more ? ` (truncated; resume at ${response.next_cursor})` : ""}`,
  );
  for (const entry of response.tickers.slice(0, 10)) {
    const mid = entry.midpoint ?? "—";
    console.log(
      `  ${entry.orderbook_id} (market ${entry.market_pubkey}, outcome ${entry.outcome_index}) mid=${mid}`,
    );
  }
}

void runExample(main);
