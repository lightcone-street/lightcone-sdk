// TypeScript example — read on-chain account state (Exchange / Market / Orderbooks
// / nonce / position) and derive the related PDAs, entirely via the gentype facade
// (no imports from the compiled .res.mjs). Run with:
//   npx tsx examples/ReadOnchain.ts
import * as fs from "node:fs";
import * as os from "node:os";
import { createKeyPairFromBytes, getAddressFromPublicKey } from "@solana/kit";
import {
  makeForEnv,
  MarketClient,
  RpcClient,
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

  // The user pubkey for the nonce + position reads (no signing needed).
  const keypair = await createKeyPairFromBytes(walletSecretKey());
  const user = await getAddressFromPublicKey(keypair.publicKey);

  // REST market metadata locates the on-chain accounts to read.
  const page = await MarketClient.get(client, undefined, 1);
  const market = page.markets[0];
  if (!market) {
    console.log("no markets found");
    return;
  }
  const pair = market.orderbookPairs.find((candidate) => candidate.active) ?? market.orderbookPairs[0];
  if (!pair) {
    console.log("selected market has no orderbooks");
    return;
  }

  const exchange = await RpcClient.exchange(client);
  const onchainMkt = await RpcClient.market(client, market.pubkey);
  const onchainOb = await RpcClient.orderbook(client, pair.base.mint, pair.quote.mint);
  const nonce = await RpcClient.nonce(client, user);
  const position = await RpcClient.position(client, user, market.pubkey);

  console.log(`exchange: authority=${exchange.authority} operator=${exchange.operator} paused=${exchange.paused}`);
  console.log(`market: id=${onchainMkt.marketId} outcomes=${onchainMkt.numOutcomes} status=${onchainMkt.status}`);
  console.log(
    `orderbook: lookup_table=${onchainOb.lookupTable} base_index=${onchainOb.baseIndex} bump=${onchainOb.bump}`,
  );
  console.log(`user nonce: ${nonce}`);
  console.log(`position exists: ${position !== undefined}`);

  const exPda = await RpcClient.exchangePda(client);
  const mPda = await RpcClient.marketPda(client, onchainMkt.marketId);
  const pPda = await RpcClient.positionPda(client, user, market.pubkey);
  const gdPda = await RpcClient.globalDepositTokenPda(client, pair.quote.depositAsset);
  console.log(`pdas: exchange=${exPda} market=${mPda} position=${pPda} global_deposit=${gdPda}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
