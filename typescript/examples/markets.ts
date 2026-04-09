import { market, restClient, runExample } from "./common";

async function main() {
  const client = restClient();

  const featured = await client.markets().featured();
  console.log("featured markets:", featured.length);
  const firstFeatured = featured[0];
  if (firstFeatured) {
    console.log(`featured: ${firstFeatured.market_name} (${firstFeatured.slug})`);
  }

  const page = await client.markets().get(undefined, 5);
  console.log(
    `paginated listing: ${page.markets.length} markets, ${page.validationErrors.length} validation errors`
  );

  const selectedMarket = await market(client);
  console.log(`by slug: ${selectedMarket.slug} -> ${selectedMarket.pubkey}`);
  console.log(
    "by pubkey:",
    (await client.markets().getByPubkey(selectedMarket.pubkey)).name
  );

  const query =
    selectedMarket.name.split(/\s+/).find((word) => word.length > 3) ?? "market";
  const results = await client.markets().search(query, 5);
  console.log(`search '${query}': ${results.length} result(s)`);
  for (const result of results) {
    console.log(`  - ${result.slug}`);
  }
}

void runExample(main);
