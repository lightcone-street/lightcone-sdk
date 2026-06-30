// TypeScript example — on-chain position flow against the first market's first
// orderbook, entirely through the gentype facade. Mirrors
// GlobalDepositWithdrawal.example.res: deposit collateral into the global pool, move
// some into a market (minting a complete conditional set), withdraw from the global
// pool, then merge the conditional set back to collateral — a net-neutral cycle.
//
// Unlike the ReScript core, the facade signs + broadcasts each transaction with the
// client's native signer internally, so no @solana/kit `address` values or keypairs
// are needed; each call returns its transaction signature.
// Run with: npx tsx examples/GlobalDepositWithdrawal.example.ts
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
  await AuthClient.login(client, undefined);

  const page = await MarketClient.get(client, undefined, 1);
  const market = page.markets[0];
  if (!market) {
    console.log("no markets found");
    return;
  }

  const pair = market.orderbookPairs.find((p) => p.active) ?? market.orderbookPairs[0];
  if (!pair) {
    console.log("selected market has no orderbooks");
    return;
  }

  const marketPubkey = market.pubkey;
  const mint = pair.quote.depositAsset;
  const numOutcomes = market.outcomes.length;
  const amount = 1000000n; // 1 unit at 6 decimals
  const depositAmount = 2000000n; // deposit extra so global has funds after the market transfer

  // 1. Fund the global pool with collateral.
  const depositSignature = await PositionClient.depositToGlobal(client, mint, depositAmount);
  console.log(`deposit_to_global: confirmed ${depositSignature}`);

  // 2. Move capital into the market (mints a complete conditional set).
  const marketDepositSignature = await PositionClient.globalToMarketDeposit(client, marketPubkey, mint, amount, numOutcomes);
  console.log(`global_to_market_deposit: confirmed ${marketDepositSignature}`);

  // 3. Pull collateral back out of the global pool.
  const withdrawSignature = await PositionClient.withdrawFromGlobal(client, mint, amount);
  console.log(`withdraw_from_global: confirmed ${withdrawSignature}`);

  // 4. Burn the conditional set, releasing collateral (closes the position).
  const mergeSignature = await PositionClient.merge(client, marketPubkey, mint, amount, numOutcomes);
  console.log(`merge: confirmed ${mergeSignature}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
