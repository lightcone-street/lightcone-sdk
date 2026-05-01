import { Transaction } from "@solana/web3.js";
import { generateSalt } from "../src/program";
import {
  confirmTransactionOrThrow,
  freshOrderNonce,
  getKeypair,
  login,
  marketAndOrderbook,
  quoteDepositMint,
  rpcClient,
  runExample,
  waitForGlobalBalance,
} from "./common";

// Quote needed for the bid below (price * size, scaled to the deposit asset's
// decimals). Must stay in sync with the same constant in `cancel_order.ts`,
// which withdraws this amount back out of the global pool after cancelling.
const ORDER_QUOTE_AMOUNT = 1_100_000n; // 0.55 * 2 USDC, 6 decimals

async function main() {
  const keypair = getKeypair();
  const client = rpcClient();
  client.setSigningStrategy({ type: "native", keypair });
  await login(client, keypair);

  const [market, orderbook] = await marketAndOrderbook(client);
  const mint = quoteDepositMint(orderbook);
  const connection = client.rpc().inner();

  // 1. Deposit collateral into the global pool.
  //
  // submit_order uses the client's default deposit source (Global), so the
  // global pool must cover `price * size` in the deposit asset's base units
  // before the order can be placed. The companion `cancel_order` example
  // cancels this order and withdraws the same amount back to the user's
  // token account, keeping the deposit/submit/cancel/withdraw cycle
  // net-neutral across CI runs.
  const depositIx = client
    .positions()
    .depositToGlobal()
    .user(keypair.publicKey)
    .mint(mint)
    .amount(ORDER_QUOTE_AMOUNT)
    .buildIx();
  {
    const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
    const tx = new Transaction({
      feePayer: keypair.publicKey,
      blockhash,
      lastValidBlockHeight,
    }).add(depositIx);
    tx.sign(keypair);
    const sig = await connection.sendRawTransaction(tx.serialize());
    await confirmTransactionOrThrow(connection, sig, { blockhash, lastValidBlockHeight });
    console.log(`deposit_to_global: confirmed ${sig}`);
  }

  await waitForGlobalBalance(client, mint, 1.1);

  // 2. Submit the limit order. Fetch and cache the on-chain nonce once —
  //    subsequent orders that omit `.nonce()` use this cached value.
  const nonce = await freshOrderNonce(client, keypair.publicKey);
  client.setOrderNonce(nonce);

  const response = await client
    .orders()
    .limitOrder()
    .maker(keypair.publicKey)
    .bid()
    .price("0.55")
    .size("2")
    .salt(generateSalt())
    .submit(client, orderbook);
  console.log(
    `submitted: ${response.order_hash} filled=${response.filled} remaining=${response.remaining} fills=${response.fills.length}`
  );
}

void runExample(main);
