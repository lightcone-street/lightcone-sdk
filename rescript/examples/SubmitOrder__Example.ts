// TypeScript example — submit a signed limit order (bid) on the first market's
// first orderbook, entirely through the gentype facade. Mirrors
// SubmitOrder__Example.res: authenticate, deposit the order's quote collateral
// into the global pool, then sign + POST the order. Unlike the ReScript core,
// the facade sources the native signer internally, so no @solana/kit
// keypair/address helpers are needed: market and mint pubkeys are plain strings.
// Run with: npx tsx examples/SubmitOrder__Example.ts
import * as fs from "node:fs";
import * as os from "node:os";
import {
  makeForEnv,
  useNativeSigner,
  signerAddress,
  setOrderNonce,
  AuthClient,
  MarketClient,
  OrderClient,
  PositionClient,
  RpcClient,
} from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

// Quote needed for the bid below (price * size, scaled to the deposit asset's
// 6 decimals). Must stay in sync with the same constant in CancelOrder__Example.ts,
// which withdraws this amount back out of the global pool after cancelling —
// keeping the deposit/submit/cancel/withdraw cycle net-neutral across runs.
const ORDER_QUOTE_AMOUNT = 1_100_000n; // 0.55 * 2 USDC

function walletSecretKey(): Uint8Array {
  const path = process.env.LIGHTCONE_WALLET_PATH ?? "~/.config/solana/id.json";
  const resolved = path.startsWith("~") ? path.replace("~", os.homedir()) : path;
  return Uint8Array.from(JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[]);
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

// Poll the tracked deposit-token balances until `mint` shows at least `minimum`
// idle — the deposit must land before the order can rest (15 × 2s cap).
async function waitForGlobalBalance(
  client: ReturnType<typeof makeForEnv>,
  mint: string,
  minimum: number
): Promise<void> {
  console.log(`waiting for global balance: mint=${mint} required=${minimum}`);
  for (let attempt = 1; attempt <= 15; attempt++) {
    const balances = await PositionClient.depositTokenBalances(client);
    const idle = balances[mint]?.idle ?? "0";
    if (Number(idle) >= minimum) {
      console.log(`global balance ready: idle=${idle} (attempt ${attempt})`);
      return;
    }
    console.log(`global balance not ready: idle=${idle}/${minimum} (attempt ${attempt})`);
    await sleep(2000);
  }
  throw new Error(`global balance for ${mint} did not reach ${minimum} within 30s`);
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

  // 1. Fund the global pool: submit uses the client's default Global deposit
  //    source, so the pool must cover `price * size` in the quote deposit asset
  //    before the order can be placed.
  const mint = pair.quote.depositAsset;
  const depositSignature = await PositionClient.depositToGlobal(client, mint, ORDER_QUOTE_AMOUNT);
  console.log(`deposit_to_global: confirmed ${depositSignature}`);
  await waitForGlobalBalance(client, mint, 1.1);

  // 2. Sign with the maker's current on-chain nonce — fetch and cache it once;
  //    subsequent submits that omit a nonce reuse the cached value.
  const maker = signerAddress(client);
  if (maker !== undefined) {
    const nonce = await RpcClient.nonce(client, maker);
    setOrderNonce(client, BigInt(nonce));
  }

  // 3. Derive the scaling decimals from the pair's token metadata (no REST call),
  //    mirroring OrderBookPair::decimals(): price_decimals = max(0, 6 + quote - base).
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
    "0.55",
    "2",
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
