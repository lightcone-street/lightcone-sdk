import { restClient } from "./common";

async function main() {
  const client = restClient();

  // 1. Featured markets
  const featured = await client.markets().featured();
  console.log("featured markets:", featured.length);

  // 2. Paginated listing
  const page = await client.markets().get(undefined, 5);
  console.log("markets:", page.markets.length);
  if (page.validationErrors.length > 0) {
    console.log("validation errors:", page.validationErrors);
  }

  // 3. Lookup by pubkey
  const first = page.markets[0];
  if (first) {
    const byPubkey = await client.markets().getByPubkey(first.pubkey);
    console.log("by pubkey:", byPubkey.name);
  }

  // 4. Search by slug
  const query = first?.slug ?? "market";
  const results = await client.markets().search(query, 5);
  console.log("search results:", results.length);
  for (const result of results) {
    console.log(" -", result.slug);
  }
}

main().catch(console.error);
