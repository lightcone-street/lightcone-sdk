// TypeScript example — WebSocket (authenticated): a user's private stream (auth
// handshake, balance, order, deposit, and nonce events) alongside a market's lifecycle
// stream. The user channel requires login first — the cookie set by `login` authenticates
// the socket. Driven entirely through the gentype facade — this proves the WS layer is
// reachable from TypeScript without importing any `.res.mjs` or `@solana/kit`.
// Note: connects to a live WS server; push messages arrive via the onMessage callback.
import * as fs from "node:fs";
import * as os from "node:os";
import {
  makeForEnv,
  MarketClient,
  useNativeSigner,
  AuthClient,
  signerAddress,
  WsClient,
} from "../src/TypeScriptApi.gen.ts";
import type { WsClient_wsMessage as wsMessage, WsClient_wsSubscription as wsSubscription } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

function walletSecretKey(): Uint8Array {
  const path = process.env.LIGHTCONE_WALLET_PATH ?? "~/.config/solana/id.json";
  const resolved = path.startsWith("~") ? path.replace("~", os.homedir()) : path;
  return Uint8Array.from(JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[]);
}

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  // Authenticate first: the user channel is gated by the session cookie that login sets.
  await useNativeSigner(client, walletSecretKey());
  await AuthClient.login(client, undefined);
  const wallet = signerAddress(client);

  const page = await MarketClient.get(client, undefined, 1);
  const market = page.markets[0];
  if (!market) {
    console.log("no markets found");
    return;
  }

  let eventCount = 0;
  const connection = WsClient.connect(
    client,
    (msg: wsMessage) => {
      const kind = msg.kind;
      if (typeof kind === "string") {
        // "Pong" heartbeat — no payload.
        return;
      }
      switch (kind.TAG) {
        case "Auth":
          eventCount += 1;
          console.log("Auth received");
          break;
        case "User":
          eventCount += 1;
          console.log("User received");
          break;
        case "Market":
          eventCount += 1;
          console.log("Market received");
          break;
        default:
          break;
      }
    },
    undefined,
    undefined,
  );

  // The user's private stream (requires the auth cookie) plus the market lifecycle stream.
  if (wallet) {
    WsClient.subscribe(connection, { TAG: "User", _0: wallet } as wsSubscription);
  } else {
    console.log("no signer configured — skipping user subscription");
  }
  WsClient.subscribe(connection, { TAG: "Market", _0: market.pubkey } as wsSubscription);

  console.log(`subscribed to user + market streams for ${market.pubkey}`);
  await new Promise((r) => setTimeout(r, 15000));
  WsClient.disconnect(connection);
  if (eventCount === 0) {
    console.log("received no websocket events — connection may be unreachable");
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
