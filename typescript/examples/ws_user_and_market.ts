import { asPubkeyStr } from "../src/shared/types";
import type { WsEvent } from "../src/ws";
import { restClient, wallet, login, marketAndOrderbook } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();

  // Login required for user stream
  await login(client, keypair);
  console.log("logged in:", keypair.publicKey.toBase58());

  const [m] = await marketAndOrderbook(client);

  // Connect WebSocket (auth token passed automatically)
  const ws = client.ws();
  await ws.connect();
  console.log("connected");

  // Subscribe to user events and market events
  ws.subscribe({
    type: "user",
    wallet_address: asPubkeyStr(keypair.publicKey.toBase58()),
  });
  ws.subscribe({
    type: "market",
    market_pubkey: m.pubkey,
  });

  let eventCount = 0;
  const maxEvents = 15;

  ws.on((event: WsEvent) => {
    if (event.type === "Message") {
      const msg = event.message;

      if (msg.type === "auth") {
        console.log("[auth]", JSON.stringify(msg.data));
      } else if (msg.type === "user") {
        console.log("[user]", JSON.stringify(msg.data).slice(0, 500));
      } else if (msg.type === "market") {
        console.log("[market]", JSON.stringify(msg.data).slice(0, 500));
      }
    } else if (event.type === "Error") {
      console.error("ws error:", event.error);
    } else if (event.type === "Disconnected") {
      console.log("disconnected:", event.reason);
    }

    eventCount++;
    if (eventCount >= maxEvents) {
      console.log("received", maxEvents, "events, disconnecting");
      ws.disconnect();
    }
  });

  // Keep alive for 30 seconds
  await new Promise((resolve) => setTimeout(resolve, 30_000));
  await ws.disconnect();
}

main().catch(console.error);
