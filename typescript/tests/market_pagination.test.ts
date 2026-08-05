import { once } from "node:events";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import assert from "node:assert/strict";
import { test } from "node:test";
import { LightconeClientBuilder } from "../src/client";

test("markets get preserves backend pagination metadata", async () => {
  let requestedUrl: string | undefined;
  const server = createServer((request, response) => {
    requestedUrl = request.url;
    response.writeHead(200, { "content-type": "application/json" });
    response.end(
      JSON.stringify({
        status: "success",
        body: {
          markets: [],
          next_cursor: 42,
          has_more: true,
        },
      })
    );
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address() as AddressInfo;
  const client = new LightconeClientBuilder()
    .baseUrl(`http://127.0.0.1:${address.port}`)
    .build();

  try {
    const page = await client.markets().get(7, 50);

    assert.deepEqual(page, {
      markets: [],
      validationErrors: [],
      nextCursor: 42,
      hasMore: true,
    });
    assert.equal(requestedUrl, "/api/markets?cursor=7&limit=50");
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve()))
    );
  }
});
