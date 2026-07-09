// TypeScript example — build + sign + broadcast a short sequence of on-chain
// transactions through the gentype facade, reporting each transaction signature.
// Mirrors OnchainTransactions.example.res: prove the RPC path with a recent
// blockhash, then send a net-neutral deposit + withdraw against the global pool.
//
// Unlike the ReScript core, the facade signs + broadcasts with the client's native
// signer internally, so no @solana/kit `address` values or keypairs are needed; the
// deposit mint is read as a plain string from the first market's first orderbook.
// Run with: npx tsx examples/OnchainTransactions.example.ts
import * as fs from "node:fs";
import * as os from "node:os";
import {
  makeForEnv,
  useNativeSigner,
  AuthClient,
  MarketClient,
  RpcClient,
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
  await AuthClient.login(client, undefined);

  // A recent blockhash proves the @solana/kit RPC path before we send.
  const blockhash = await RpcClient.latestBlockhash(client);
  console.log(`latest blockhash: ${blockhash}`);

  const page = await MarketClient.get(client, undefined, 1);
  const market = page.markets[0];
  if (!market) {
    console.log("no markets found");
    return;
  }

  const pair = market.orderbookPairs.find((p) => p.active) ?? market.orderbookPairs[0];
  if (!pair) {
    console.log("no orderbook found");
    return;
  }

  const mint = pair.quote.depositAsset;
  const amount = 1000000n; // 1 unit at 6 decimals

  // Send a net-neutral deposit then withdraw, reporting each tx signature.
  const depositSignature = await PositionClient.depositToGlobal(client, mint, amount);
  console.log(`deposit_to_global: confirmed ${depositSignature}`);

  const withdrawSignature = await PositionClient.withdrawFromGlobal(client, mint, amount);
  console.log(`withdraw_from_global: confirmed ${withdrawSignature}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
