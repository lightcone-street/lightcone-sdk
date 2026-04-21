import { Transaction } from "@solana/web3.js";
import { asPubkeyStr } from "../src";
import {
  cancelBodySigned,
  cancelAllBodySigned,
} from "../src/domain/order/client";
import { generateCancelAllSalt } from "../src/program";
import {
  confirmTransactionOrThrow,
  depositMint,
  getKeypair,
  login,
  market,
  restClient,
  runExample,
  unixTimestamp,
} from "./common";

// Mirrors the constant in `submit_order.ts`. When we cancel the order that
// example left open, we withdraw the same quote amount back from the global
// pool so the deposit/submit/cancel/withdraw cycle is net-neutral.
const ORDER_QUOTE_AMOUNT = 1_100_000n; // 0.55 * 2 USDC, 6 decimals

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  await login(client, keypair);
  const pubkey = keypair.publicKey.toBase58();

  const snapshot = await client.orders().getUserOrders(pubkey, 50);
  const limitOrder = snapshot.orders.find((o) => o.order_type === "limit");

  if (!limitOrder) {
    console.log("No open limit orders to cancel.");
    return;
  }

  const orderHash = limitOrder.order_hash;
  const orderbookId = limitOrder.orderbook_id;

  const cancel = cancelBodySigned(orderHash, asPubkeyStr(pubkey), keypair);
  const cancelled = await client.orders().cancel(cancel);
  console.log(`cancelled: ${cancelled.order_hash} remaining=${cancelled.remaining}`);

  const timestamp = unixTimestamp();
  const salt = generateCancelAllSalt();
  const cancelAll = cancelAllBodySigned(
    asPubkeyStr(pubkey),
    orderbookId,
    timestamp,
    salt,
    keypair
  );
  const cleared = await client.orders().cancelAll(cancelAll);
  console.log(`cancel-all removed ${cleared.count} order(s) in ${cleared.orderbook_id}`);

  // Cleanup: cancelling the order released its locked collateral back into
  // the global pool. Withdraw that amount to the user's token account so the
  // companion `submit_order` → `cancel_order` cycle is net-neutral on the
  // wallet's balance and the global pool.
  const m = await market(client);
  const mint = depositMint(m);
  const connection = client.rpc().inner();
  const withdrawIx = client
    .positions()
    .withdrawFromGlobal()
    .user(keypair.publicKey)
    .mint(mint)
    .amount(ORDER_QUOTE_AMOUNT)
    .buildIx();
  const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
  const tx = new Transaction({
    feePayer: keypair.publicKey,
    blockhash,
    lastValidBlockHeight,
  }).add(withdrawIx);
  tx.sign(keypair);
  const sig = await connection.sendRawTransaction(tx.serialize());
  await confirmTransactionOrThrow(connection, sig, { blockhash, lastValidBlockHeight });
  console.log(`withdraw_from_global: confirmed ${sig}`);
}

void runExample(main);
