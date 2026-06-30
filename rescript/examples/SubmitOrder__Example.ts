// TypeScript example — submit a signed limit order (bid) on the first market's
// first orderbook, entirely through the gentype facade. Mirrors
// SubmitOrder.example.res. Unlike the ReScript core, the facade sources the native
// signer internally, so no @solana/kit keypair/address helpers are needed: market
// and mint pubkeys are plain strings and `submitLimitOrder` signs + POSTs the order.
//
// The global pool must already hold `price * size` of the quote deposit asset as
// collateral before the order can rest — see GlobalDepositWithdrawal.example.ts.
// Run with: npx tsx examples/SubmitOrder.example.ts
import * as fs from "node:fs";
import * as os from "node:os";
import { makeForEnv, useNativeSigner, AuthClient, MarketClient, OrderClient } from "../src/TypeScriptApi.gen.ts";
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

  // Derive the scaling decimals from the pair's token metadata (no REST call),
  // mirroring OrderBookPair::decimals(): price_decimals = max(0, 6 + quote - base).
  const baseDecimals = Math.trunc(pair.base.decimals);
  const quoteDecimals = Math.trunc(pair.quote.decimals);
  const priceDecimals = Math.max(0, 6 + quoteDecimals - baseDecimals);
  const tickSize = Math.max(0, pair.tickSize);

  // All-positional facade call; side 0 = bid, 1 = ask. timeInForce defaults (undefined).
  const response = await OrderClient.submitLimit(
    client,
    market.pubkey,
    pair.base.mint,
    pair.quote.mint,
    0,
    "0.50",
    "10",
    baseDecimals,
    quoteDecimals,
    priceDecimals,
    tickSize,
    pair.orderbookId,
    undefined,
  );

  console.log(`submitted: ${response.orderHash} status=${response.status}`);
  console.log(`filled=${response.filled} remaining=${response.remaining} fills=${response.fills.length}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
