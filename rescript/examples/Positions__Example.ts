// TypeScript example — a user's positions (portfolio-wide + per-market) via the
// gentype facade. Authenticates with a wallet keypair to resolve the trading
// wallet, then queries the public path-based position endpoints.
import * as fs from "node:fs";
import * as os from "node:os";
import {
  makeForEnv,
  useNativeSigner,
  AuthClient,
  MarketClient,
  PositionClient,
} from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

function walletSecretKey(): Uint8Array {
  const path = process.env.LIGHTCONE_WALLET_PATH ?? "~/.config/solana/id.json";
  const resolved = path.startsWith("~") ? path.replace("~", os.homedir()) : path;
  return Uint8Array.from(JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[]);
}

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  await useNativeSigner(client, walletSecretKey());
  const session = await AuthClient.login(client, undefined);
  const identity = session.user.identity;
  if (identity.TAG !== "Wallet") {
    throw new Error(`expected a wallet identity, got ${identity.TAG}`);
  }
  const wallet = identity.address;

  const page = await MarketClient.get(client, undefined, 1);
  const market = page.markets[0];
  if (!market) {
    console.log("no markets found");
    return;
  }

  const all = await PositionClient.forUser(client, wallet);
  const perMarket = await PositionClient.forMarket(client, wallet, market.pubkey);

  console.log(`wallet: ${wallet}`);
  console.log(`markets with positions: ${all.totalMarkets}`);
  console.log(`positions in ${market.slug}: ${perMarket.positions.length}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
