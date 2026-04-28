import type { WsEvent } from "../src";
import { restClient, runExample, withTimeout } from "./common";

async function main() {
  const client = restClient();

  // REST: snapshot of current prices for every active mint in `global_deposit_tokens`.
  const snapshot = await client.priceHistory().getDepositAssetPricesSnapshot();
  const entries = Object.entries(snapshot.prices);
  console.log(
    `REST /api/deposit-asset-prices-snapshot (${entries.length} entries):`
  );
  for (const [mint, price] of entries.slice(0, 10)) {
    console.log(`  ${mint} -> ${price}`);
  }

  // Pick the first asset and subscribe via WS for live updates.
  const firstEntry = entries[0];
  if (!firstEntry) {
    console.log("snapshot has no entries — backend has no priced assets");
    return;
  }
  const [depositAsset] = firstEntry;

  const ws = client.ws();
  let hits = 0;

  let resolveDone!: () => void;
  const done = new Promise<void>((resolve) => {
    resolveDone = resolve;
  });

  const unsubscribe = ws.on((event: WsEvent) => {
    if (
      event.type === "Message" &&
      event.message.type === "deposit_asset_price"
    ) {
      const payload = event.message.data;
      if (payload.event_type === "snapshot") {
        console.log(`WS snapshot: ${payload.deposit_asset} -> ${payload.price}`);
        hits += 1;
      } else if (payload.event_type === "price") {
        console.log(
          `WS tick: ${payload.deposit_asset} -> ${payload.price} @ ${payload.event_time}`
        );
        hits += 1;
      }
    } else if (event.type === "Error") {
      console.error("ws error:", event.error);
    }

    if (hits >= 2) {
      resolveDone();
    }
  });

  try {
    await ws.connect();
    ws.subscribe({ type: "deposit_asset_price", deposit_asset: depositAsset });
    await withTimeout(done, 30_000, "timed out waiting for websocket data");
  } catch {
    console.log("no more websocket data (timeout or stream ended)");
  } finally {
    unsubscribe();
    ws.unsubscribe({ type: "deposit_asset_price", deposit_asset: depositAsset });
    await ws.disconnect();
  }

  if (hits === 0) {
    throw new Error("received no websocket events — connection may be broken");
  }
}

void runExample(main);
