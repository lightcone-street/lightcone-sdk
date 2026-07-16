import { once } from "node:events";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import assert from "node:assert/strict";
import { test } from "node:test";
import { LightconeClientBuilder } from "../src/client";

test("favorite markets methods use the contracted verbs, paths, cookies, and bodies", async () => {
  const requests: Array<{ method?: string; url?: string; cookie?: string }> = [];
  const server = createServer((request, response) => {
    requests.push({ method: request.method, url: request.url, cookie: request.headers.cookie });
    const favorited = request.method === "POST";
    const body = request.method === "GET"
      ? { market_pubkeys: ["market-a"], next_cursor: 12, has_more: true }
      : { market_pubkey: "market%2Fone", favorited };
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify({ status: "success", body }));
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address() as AddressInfo;
  const client = new LightconeClientBuilder().baseUrl(`http://127.0.0.1:${address.port}`).build();

  try {
    assert.deepEqual(await client.markets().favoriteMarketsWithCookies("lightcone-token=test", 50, 7), { market_pubkeys: ["market-a"], next_cursor: 12, has_more: true });
    assert.equal((await client.markets().addFavoriteMarketWithCookies("market/one", "lightcone-token=test")).favorited, true);
    assert.equal((await client.markets().removeFavoriteMarketWithCookies("market/one", "lightcone-token=test")).favorited, false);
    assert.deepEqual(requests, [
      { method: "GET", url: "/api/users/favorite-markets?limit=50&cursor=7", cookie: "lightcone-token=test" },
      { method: "POST", url: "/api/users/favorite-markets/market%2Fone", cookie: "lightcone-token=test" },
      { method: "DELETE", url: "/api/users/favorite-markets/market%2Fone", cookie: "lightcone-token=test" },
    ]);
  } finally {
    await new Promise<void>((resolve, reject) => server.close((error) => error ? reject(error) : resolve()));
  }
});
