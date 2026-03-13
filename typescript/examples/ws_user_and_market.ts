import { asPubkeyStr, type WsEvent } from "../src";
import { login, market, restClient, wallet, withTimeout } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();
  await login(client, keypair);
  const m = await market(client);
  const ws = client.ws();
  let sawAuth = false;
  let sawUser = false;
  let sawMarket = false;

  let resolveDone!: () => void;
  const done = new Promise<void>((resolve) => {
    resolveDone = resolve;
  });

  const unsubscribe = ws.on((event: WsEvent) => {
    if (event.type === "Message" && event.message.type === "auth") {
      console.log("auth:", event.message.data);
      sawAuth = true;
    } else if (event.type === "Message" && event.message.type === "user") {
      console.log("user:", event.message.data);
      sawUser = true;
    } else if (event.type === "Message" && event.message.type === "market") {
      console.log("market:", event.message.data);
      sawMarket = true;
    } else if (event.type === "Error") {
      console.error("ws error:", event.error);
    }

    if (sawAuth && sawUser) {
      resolveDone();
    }
  });

  try {
    await ws.connect();
    ws.subscribe({
      type: "user",
      wallet_address: asPubkeyStr(keypair.publicKey.toBase58()),
    });
    ws.subscribe({
      type: "market",
      market_pubkey: m.pubkey,
    });
    await withTimeout(done, 15_000, "timed out waiting for websocket data");
  } finally {
    unsubscribe();
    await ws.disconnect();
  }

  console.log(`market event received: ${sawMarket}`);
}

main().catch(console.error);
